import Testing
@testable import CodyncUI

/// A provider's recording goes out in 20 s pieces that cover every byte in order; a tail under
/// half a second (noise a model would invent words for) stays with the piece before.
@Test @MainActor func dictationPiecesCoverTheRecording() {
    let piece = Dictation.pieceBytes, min = Dictation.minBytes
    #expect(Dictation.pieces(of: piece / 2) == [0..<piece / 2])
    #expect(Dictation.pieces(of: piece * 2) == [0..<piece, piece..<piece * 2])
    #expect(Dictation.pieces(of: piece * 2 + min) == [0..<piece, piece..<piece * 2, piece * 2..<piece * 2 + min])
    #expect(Dictation.pieces(of: piece + min - 1) == [0..<piece + min - 1])
    #expect(Dictation.pieces(of: piece * 2 + 2) == [0..<piece, piece..<piece * 2 + 2])
}

@Test @MainActor func dictatedWordsGoAfterTheBox() {
    #expect(Dictation.join("", "hello") == "hello")
    #expect(Dictation.join("Fix the bug", "in the parser") == "Fix the bug in the parser")
    #expect(Dictation.join("Fix the bug\n", "now") == "Fix the bug\nnow")
    #expect(Dictation.join("Fix the bug", "") == "Fix the bug")
}
