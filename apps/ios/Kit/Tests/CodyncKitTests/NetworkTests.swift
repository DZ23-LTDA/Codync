import Foundation
import Testing
@testable import CodyncKit

@Test func decodesTheNetworkViewAndItsEvent() throws {
    let view = #"{"joined":true,"computer":"desk","role":"owner","connector":"public","assistants":true,"computers":[{"name":"desk","role":"owner","me":true,"online":true,"bots":["reviewer"],"agents":[{"id":"reviewer","name":"Reviewer"}],"canRemove":false,"canPromote":false},{"name":"laptop","role":"overlord","me":false,"online":false,"bots":[],"agents":[],"canRemove":true,"canPromote":true}]}"#
    let decoded = try JSONDecoder().decode(NetworkView.self, from: Data(view.utf8))
    #expect(decoded.joined && decoded.isAssistantsOn && decoded.role == .owner)
    #expect(decoded.computers?.map(\.name) == ["desk", "laptop"])
    #expect(decoded.computers?.first?.agents?.first?.name == "Reviewer")
    #expect(decoded.computers?.last?.role == .member, "an unknown role reads as a member")

    guard case let .network(event) = HostClient.parseEvent(Data(#"{"type":"network","network":\#(view)}"#.utf8)) else {
        Issue.record("network event"); return
    }
    #expect(event == decoded)
    // Outside a network, from a host that only says whether assistants are on.
    let alone = try JSONDecoder().decode(NetworkView.self, from: Data(#"{"joined":false,"assistants":false}"#.utf8))
    #expect(!alone.hasContent && alone.computers == nil)
    // A network event this app can't read is skipped, never a rewind.
    #expect(HostClient.parseEvent(Data(#"{"type":"network","network":{"joined":"yes"}}"#.utf8)) == nil)
}
