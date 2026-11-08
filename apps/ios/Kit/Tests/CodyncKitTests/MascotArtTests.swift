import Testing
import UIKit
@testable import CodyncKit

/// The mascot art ships as copied files under `Mascots/` in the package bundle, so a missing resource would leave a
/// bot faceless without failing anything else. These pin the lookup and the fallback.
@Suite("Mascot art")
@MainActor
struct MascotArtTests {
    /// The 13 ids in `host/src/agent/avatar.rs`, in its order.
    private let catalog = [
        "tele", "steve-jobs", "nina", "rogerio-ceni", "ed", "jo-soares", "sherlock", "flanelinha", "ju",
        "joel-santana", "pingado", "astro", "byte",
    ]

    @Test("every mascot the host can assign has its picture in the package bundle")
    func everyCatalogMascotLoads() {
        for id in catalog {
            let image = MascotArt.image(id)
            #expect(image != nil, "\(id) has no picture")
            #expect(image?.size.width == 512, "\(id) is not the derived 512 px art")
        }
    }

    @Test("an unknown or unset id has no picture, so the drawn avatar stands")
    func unknownIdsFallBack() {
        #expect(MascotArt.image("") == nil)
        #expect(MascotArt.image("not-a-mascot") == nil)
    }

    @Test("a bot carries the mascot the host sent, and one that sent none is still a bot")
    func botDecodesItsMascot() throws {
        let with = try JSONDecoder().decode(Bot.self, from: Data(#"{"id":"b","avatar":"sherlock"}"#.utf8))
        #expect(with.avatar == "sherlock")
        let without = try JSONDecoder().decode(Bot.self, from: Data(#"{"id":"b"}"#.utf8))
        #expect(without.avatar.isEmpty)
    }
}
