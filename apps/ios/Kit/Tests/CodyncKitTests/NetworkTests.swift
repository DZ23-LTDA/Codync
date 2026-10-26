import Foundation
import Testing
@testable import CodyncKit

@Test func decodesTheNetworkViewAndItsEvent() throws {
    let view = #"{"joined":true,"computer":"desk","role":"owner","connector":"public","assistants":true,"assistantsAvailable":true,"computers":[{"name":"desk","role":"owner","me":true,"online":true,"agents":[{"id":"reviewer","name":"Reviewer"}],"canRemove":false,"canPromote":false},{"name":"laptop","role":"overlord","me":false,"online":false,"agents":[],"canRemove":true,"canPromote":true}]}"#
    let decoded = try JSONDecoder().decode(NetworkView.self, from: Data(view.utf8))
    #expect(decoded.joined && decoded.hasContent && decoded.role == .owner)
    #expect(decoded.computers?.map(\.name) == ["desk", "laptop"])
    #expect(decoded.computers?.first?.agents?.first?.name == "Reviewer")
    #expect(decoded.computers?.last?.role == .member, "an unknown role reads as a member")

    guard case let .network(event) = HostClient.parseEvent(Data(#"{"type":"network","network":\#(view)}"#.utf8)) else {
        Issue.record("network event"); return
    }
    #expect(event == decoded)
    // Outside a network: shown only while cloud assistants can reach this computer and are on.
    let alone = { (json: String) in try JSONDecoder().decode(NetworkView.self, from: Data(json.utf8)) }
    #expect(try !alone(#"{"joined":false,"assistants":false,"assistantsAvailable":true}"#).hasContent)
    #expect(try !alone(#"{"joined":false,"assistants":true,"assistantsAvailable":false}"#).hasContent, "not in an account")
    #expect(try alone(#"{"joined":false,"assistants":true,"assistantsAvailable":true}"#).hasContent)
    // A network event this app can't read is skipped, never a rewind.
    #expect(HostClient.parseEvent(Data(#"{"type":"network","network":{"joined":"yes"}}"#.utf8)) == nil)
}
