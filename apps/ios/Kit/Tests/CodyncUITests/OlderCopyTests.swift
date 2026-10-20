import Foundation
import os
import Testing
@testable import CodyncKit
@testable import CodyncUI

/// Answers `/v1/computers` with the subclass's `computers(for:)` and every other request with `{}`.
private class CloudStub: URLProtocol {
    /// The listed computers' JSON objects, comma-separated; asked for every request.
    class func computers(for request: URLRequest) -> String { "" }

    override class func canInit(with request: URLRequest) -> Bool { true }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }
    override func stopLoading() {}

    override func startLoading() {
        let computers = Self.computers(for: request)
        let json = request.url?.path.hasSuffix("/v1/computers") == true ? #"{"computers":[\#(computers)]}"# : "{}"
        let response = HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!
        client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
        client?.urlProtocol(self, didLoad: Data(json.utf8))
        client?.urlProtocolDidFinishLoading(self)
    }
}

/// A cloud client whose requests `stub` answers.
private func stubbedCloud(_ stub: CloudStub.Type, token: @escaping @Sendable () async throws -> String = { "token" }) -> CloudClient {
    let config = URLSessionConfiguration.ephemeral
    config.protocolClasses = [stub]
    return CloudClient(baseURL: URL(string: "https://cloud.test")!, identity: DeviceIdentity(signing: .init(), push: .init()),
                       session: URLSession(configuration: config), token: token)
}

/// The account lists an offline "Desk" last seen at 200 and an online "Desk" claimed at `claimedAt`.
private final class OlderCopyCloudStub: CloudStub {
    static let claimedAt = OSAllocatedUnfairLock(initialState: 0)

    override class func computers(for request: URLRequest) -> String {
        #"{"computerId":"old","name":"Desk","signKey":"a","online":false,"lastSeenAt":200},"#
            + #"{"computerId":"new","name":"Desk","signKey":"b","online":true,"claimedAt":\#(claimedAt.withLock { $0 })}"#
    }
}

/// A same-named offline computer is an older copy only if it was last seen before the online one
/// joined the account; seen after, it ran beside it and is another machine.
@MainActor @Test func olderCopyNeedsToBeLastSeenBeforeTheCurrentOneJoined() async throws {
    let suite = "OlderCopy.\(UUID().uuidString)"
    defer { UserDefaults(suiteName: suite)?.removePersistentDomain(forName: suite) }
    let accounts = AccountStore(storage: SharedStore.Context(accountID: "account", suite: suite), cloud: stubbedCloud(OlderCopyCloudStub.self))
    defer { accounts.retire() }

    OlderCopyCloudStub.claimedAt.withLock { $0 = 100 }
    await accounts.refreshCloud()
    let old = try #require(accounts.cloudComputers.first { $0.computerId == "old" })
    #expect(!accounts.isOlderCopy(old))

    OlderCopyCloudStub.claimedAt.withLock { $0 = 300 }
    await accounts.refreshCloud()
    #expect(accounts.isOlderCopy(old))
}

private let currentKey = Data(repeating: 5, count: 32)

/// The account lists only the online "Desk", claimed at `claimedAt`.
private final class StaleCopyCloudStub: CloudStub {
    static let claimedAt = OSAllocatedUnfairLock(initialState: 0)

    override class func computers(for request: URLRequest) -> String {
        #"{"computerId":"\#(RelayCrypto.computerId(signKey: currentKey))","name":"Desk","signKey":"\#(currentKey.base64URL)","online":true,"claimedAt":\#(claimedAt.withLock { $0 })}"#
    }
}

/// A computer paired here and gone offline is stale beside an online one with its name only if the relay
/// last saw it before that one joined the account; seen after, it's another machine still to come back.
@MainActor @Test func offlineComputerIsStaleOnlyIfLastSeenBeforeTheCurrentOneJoined() async throws {
    let suite = "StaleCopy.\(UUID().uuidString)"
    defer { UserDefaults(suiteName: suite)?.removePersistentDomain(forName: suite) }
    let storage = SharedStore.Context(accountID: "account", suite: suite)
    let oldKey = Data(repeating: 4, count: 32)
    let old = Computer(id: RelayCrypto.computerId(signKey: oldKey), name: "Desk", signKey: oldKey.base64URL)
    storage.computers = [old]
    let accounts = AccountStore(storage: storage, cloud: stubbedCloud(StaleCopyCloudStub.self)) { computer in
        BotStore(computer: computer, storage: storage) { _ in FakeRemote(.connecting) }
    }
    defer { accounts.retire() }
    accounts.setActive(false)
    let store = try #require(accounts.store(for: old.id))
    store.connection = .computerOffline(lastSeen: Date(milliseconds: 200))

    StaleCopyCloudStub.claimedAt.withLock { $0 = 100 }
    await accounts.refreshCloud()
    #expect(!accounts.isStale(old.id))

    StaleCopyCloudStub.claimedAt.withLock { $0 = 300 }
    await accounts.refreshCloud()
    #expect(accounts.isStale(old.id))
}

private let olderKey = Data(repeating: 6, count: 32)

/// The account lists the older "Desk" beside the current one until a DELETE removes it. A list asked
/// for with the `stale` token is the one from before the removal.
private final class RemoveCopyCloudStub: CloudStub {
    static let state = OSAllocatedUnfairLock(initialState: (removed: false, deletes: 0))

    override class func computers(for request: URLRequest) -> String {
        let removed = request.httpMethod == "DELETE"
            ? state.withLock { $0.removed = true; $0.deletes += 1; return true }
            : state.withLock { $0.removed } && request.value(forHTTPHeaderField: "Authorization") != "Bearer stale"
        let older = #"{"computerId":"\#(RelayCrypto.computerId(signKey: olderKey))","name":"Desk","signKey":"\#(olderKey.base64URL)","online":false,"lastSeenAt":200}"#
        let current = #"{"computerId":"\#(RelayCrypto.computerId(signKey: currentKey))","name":"Desk","signKey":"\#(currentKey.base64URL)","online":true,"claimedAt":300}"#
        return removed ? current : "\(older),\(current)"
    }
}

/// Removing an older copy takes its row away at once: a second tap sends no second DELETE, and a list
/// asked for before the removal that arrives after it doesn't bring the row back.
@MainActor @Test func removedOlderCopyIsRemovedOnceAndStaysGone() async throws {
    let suite = "RemoveCopy.\(UUID().uuidString)"
    defer { UserDefaults(suiteName: suite)?.removePersistentDomain(forName: suite) }
    // The second token (the refresh asked for before the removal) comes late, and gets the earlier list.
    let tokens = OSAllocatedUnfairLock(initialState: 0)
    let cloud = stubbedCloud(RemoveCopyCloudStub.self) {
        guard tokens.withLock({ $0 += 1; return $0 }) == 2 else { return "token" }
        try await Task.sleep(for: .milliseconds(300))
        return "stale"
    }
    let accounts = AccountStore(storage: SharedStore.Context(accountID: "account", suite: suite), cloud: cloud)
    defer { accounts.retire() }
    await accounts.refreshCloud()
    let older = RelayCrypto.computerId(signKey: olderKey), current = RelayCrypto.computerId(signKey: currentKey)
    #expect(accounts.cloudComputers.map(\.computerId) == [older, current])

    let earlier = Task { await accounts.refreshCloud() }
    for _ in 0..<100 where tokens.withLock({ $0 }) < 2 { try await Task.sleep(for: .milliseconds(10)) }
    async let tap: Void = accounts.removeFromAccount(older)
    async let secondTap: Void = accounts.removeFromAccount(older)
    _ = await (tap, secondTap)
    await earlier.value

    #expect(RemoveCopyCloudStub.state.withLock { $0.deletes } == 1)
    #expect(accounts.cloudComputers.map(\.computerId) == [current])
}

/// A list that comes back while the removal is still on its way leaves the older copy out, so its row
/// can't come back to be tapped again.
@MainActor @Test func listDuringRemovalLeavesTheOlderCopyOut() async throws {
    let suite = "RemovingCopy.\(UUID().uuidString)"
    defer { UserDefaults(suiteName: suite)?.removePersistentDomain(forName: suite) }
    // The second request, the DELETE, is slow; the account keeps listing both copies meanwhile.
    let tokens = OSAllocatedUnfairLock(initialState: 0)
    let cloud = stubbedCloud(OlderCopyCloudStub.self) {
        if tokens.withLock({ $0 += 1; return $0 }) == 2 { try await Task.sleep(for: .milliseconds(300)) }
        return "token"
    }
    let accounts = AccountStore(storage: SharedStore.Context(accountID: "account", suite: suite), cloud: cloud)
    defer { accounts.retire() }
    await accounts.refreshCloud()
    #expect(accounts.cloudComputers.map(\.computerId) == ["old", "new"])

    let removal = Task { await accounts.removeFromAccount("old") }
    for _ in 0..<100 where tokens.withLock({ $0 }) < 2 { try await Task.sleep(for: .milliseconds(10)) }
    await accounts.refreshCloud()
    #expect(accounts.cloudComputers.map(\.computerId) == ["new"])
    await removal.value
}
