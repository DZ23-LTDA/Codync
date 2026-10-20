import Foundation
import os
import Testing
@testable import CodyncKit
@testable import CodyncUI

/// The account lists an offline "Desk" last seen at 200 and an online "Desk" claimed at `claimedAt`.
private final class OlderCopyCloudStub: URLProtocol {
    static let claimedAt = OSAllocatedUnfairLock(initialState: 0)

    override class func canInit(with request: URLRequest) -> Bool { true }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }
    override func stopLoading() {}

    override func startLoading() {
        let claimed = Self.claimedAt.withLock { $0 }
        let json = request.url?.path.hasSuffix("/v1/computers") == true
            ? #"{"computers":[{"computerId":"old","name":"Desk","signKey":"a","online":false,"lastSeenAt":200},"#
                + #"{"computerId":"new","name":"Desk","signKey":"b","online":true,"claimedAt":\#(claimed)}]}"#
            : "{}"
        let response = HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!
        client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
        client?.urlProtocol(self, didLoad: Data(json.utf8))
        client?.urlProtocolDidFinishLoading(self)
    }
}

/// A same-named offline computer is an older copy only if it was last seen before the online one
/// joined the account; seen after, it ran beside it and is another machine.
@MainActor @Test func olderCopyNeedsToBeLastSeenBeforeTheCurrentOneJoined() async throws {
    let suite = "OlderCopy.\(UUID().uuidString)"
    defer { UserDefaults(suiteName: suite)?.removePersistentDomain(forName: suite) }
    let config = URLSessionConfiguration.ephemeral
    config.protocolClasses = [OlderCopyCloudStub.self]
    let cloud = CloudClient(baseURL: URL(string: "https://cloud.test")!, identity: DeviceIdentity(signing: .init(), push: .init()),
                            session: URLSession(configuration: config)) { "token" }
    let accounts = AccountStore(storage: SharedStore.Context(accountID: "account", suite: suite), cloud: cloud)
    defer { accounts.retire() }

    OlderCopyCloudStub.claimedAt.withLock { $0 = 100 }
    await accounts.refreshCloud()
    let old = try #require(accounts.cloudComputers.first { $0.computerId == "old" })
    #expect(!accounts.isOlderCopy(old))

    OlderCopyCloudStub.claimedAt.withLock { $0 = 300 }
    await accounts.refreshCloud()
    #expect(accounts.isOlderCopy(old))
}
