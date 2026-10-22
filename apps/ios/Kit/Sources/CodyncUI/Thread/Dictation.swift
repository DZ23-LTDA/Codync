import AVFoundation
import CodyncKit
import Speech
import SwiftUI

/// Dictation into the message box. With on-device speech the words appear as they're recognized,
/// after what was already typed, and the recording pill's stop, or the recognizer finishing on a
/// pause, stops it; it uses the call's language setting. With a provider picked beside the mic, it
/// records, and the pill's stop has the computer transcribe all of it with the user's key; the
/// transcript goes after what's in the box by then.
@MainActor @Observable
final class Dictation {
    private(set) var listening = false
    /// Recorded for a provider, waiting for its transcript.
    private(set) var transcribing = false
    /// When it started listening, for the recording pill's time.
    private(set) var startedAt = Date.now
    /// The microphone's level while listening, 0…1.
    private(set) var level: Float = 0
    /// Why the last dictation ended early (no key, no network), until the next.
    var failure: String?
    @ObservationIgnored private let engine = AVAudioEngine()
    @ObservationIgnored private var request: SFSpeechAudioBufferRecognitionRequest?
    @ObservationIgnored private var task: SFSpeechRecognitionTask?
    /// Recording for a provider: what the microphone heard (16 kHz mono PCM16), and whose computer transcribes it.
    @ObservationIgnored private var recording: (provider: VoiceProvider, model: BotStore, pcm: Data)?
    /// Changes the box's text: on-device words replace what follows `before`; a transcript goes after the box as it is then.
    @ObservationIgnored private var edit: (((String) -> String) -> Void)?
    /// The box when dictation started.
    @ObservationIgnored private var before = ""
    /// A recording's transcription; `stop()` cancels it so no more pieces go out on the user's key.
    @ObservationIgnored private var transcription: Task<Void, Never>?
    /// Bumped per start and stop, so a start still waiting (key check, permission) or a late
    /// result from an earlier recognition is dropped.
    @ObservationIgnored private var generation = 0

    /// A transcription request has to fit one channel message once base64-encoded: 20 s of audio
    /// (20.5 s with a short tail still fits).
    static let pieceBytes = 16000 * 2 * 20
    /// Under half a second is a tap of the mic, not speech (and speech-to-text models invent words for silence).
    static let minBytes = 16000 * 2 / 2
    /// Transcription requests in flight at once, under the channel's and the providers' limits.
    private static let inFlight = 3

    /// `said` after what's already in the box, with one space between.
    static func join(_ before: String, _ said: String) -> String {
        if before.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty { return said }
        if said.isEmpty { return before }
        return before.last?.isWhitespace == true ? before + said : before + " " + said
    }

    /// Starts with the engine picked beside the mic, after `draft`; a provider goes through `model`'s
    /// computer. A `stop()` (another chat, a call, a send) or another tap while it waits wins.
    func start(after draft: String, model: BotStore, edit: @escaping ((String) -> String) -> Void) async {
        guard !listening, !transcribing else { return }
        failure = nil
        generation += 1
        before = draft
        if let provider = VoiceSettings.dictationProvider {
            await record(provider, model: model, generation: generation, edit: edit)
        } else {
            await recognize(generation: generation, edit: edit)
        }
    }

    /// The pill's stop: on-device speech has already typed its words; a recording gets transcribed.
    func finish() {
        guard let recording else { return stop() }
        let edit = edit
        stop()
        guard recording.pcm.count >= Self.minBytes else { return }
        let generation = generation
        transcribing = true
        transcription = Task {
            do {
                let said = try await Self.transcribe(recording.pcm, provider: recording.provider, on: recording.model)
                VoiceUsage.add(seconds: recording.pcm.count / (16000 * 2), for: recording.provider)
                guard generation == self.generation else { return }
                edit? { Self.join($0, said) }
            } catch {
                guard generation == self.generation else { return }
                failure = error.localizedDescription
            }
            transcribing = false
        }
    }

    /// Stops now and drops anything still on its way.
    func stop() {
        generation += 1
        transcription?.cancel()
        transcription = nil
        task?.cancel()
        task = nil
        request?.endAudio()
        request = nil
        recording = nil
        transcribing = false
        engine.inputNode.removeTap(onBus: 0)
        if engine.isRunning { engine.stop() }
        edit = nil
        if listening { try? AVAudioSession.sharedInstance().setActive(false, options: .notifyOthersOnDeactivation) }
        listening = false
        level = 0
    }

    private func recognize(generation: Int, edit: @escaping ((String) -> String) -> Void) async {
        guard await AVAudioApplication.requestRecordPermission(), await Self.authorized(), generation == self.generation else { return }
        let recognizer = UserDefaults.standard.string(forKey: OnDeviceEngine.languageKey)
            .flatMap { $0.isEmpty ? nil : SFSpeechRecognizer(locale: Locale(identifier: $0)) } ?? SFSpeechRecognizer()
        guard let recognizer, recognizer.isAvailable else { return }
        let request = SFSpeechAudioBufferRecognitionRequest()
        request.shouldReportPartialResults = true
        request.addsPunctuation = true
        if recognizer.supportsOnDeviceRecognition { request.requiresOnDeviceRecognition = true }
        guard openMic(bufferSize: 1024, tap: { _ in OnDeviceEngine.tap(request) { [weak self] level in
            Task { @MainActor in if self?.listening == true { self?.level = level } }
        } }) else { return }
        self.edit = edit
        self.request = request
        startedAt = .now
        listening = true
        task = recognizer.recognitionTask(with: request, resultHandler: Self.results(self, generation: generation))
    }

    /// Records for `provider` once the computer has its key, so nothing said is lost to a missing one.
    private func record(_ provider: VoiceProvider, model: BotStore, generation: Int, edit: @escaping ((String) -> String) -> Void) async {
        do {
            let status = try await model.ready().voiceStatus()
            VoiceSettings.remember(status)
            guard generation == self.generation else { return }
            guard status.configured(provider) else {
                failure = "Add your \(provider.name) key in Voice chat settings to dictate with it."
                return
            }
        } catch {
            if generation == self.generation { failure = error.localizedDescription }
            return
        }
        let allowed = await AVAudioApplication.requestRecordPermission()
        guard generation == self.generation else { return }
        guard allowed else {
            failure = "Allow microphone access in Settings to dictate."
            return
        }
        guard openMic(bufferSize: 2048, tap: { format in MicTap.pcm16k(from: format) { [weak self] pcm, level in
            Task { @MainActor in self?.recorded(pcm, level: level, generation: generation) }
        } }) else {
            failure = "The microphone isn't available."
            return
        }
        self.edit = edit
        recording = (provider, model, Data())
        startedAt = .now
        listening = true
    }

    private func recorded(_ pcm: Data, level: Float, generation: Int) {
        guard generation == self.generation, recording != nil else { return }
        recording?.pcm.append(pcm)
        self.level = level
    }

    /// The call's audio session (a recording tap on a `.record` session fails on some routes) and
    /// `tap` on the microphone. False, with everything let go, when the microphone can't be used.
    private func openMic(bufferSize: AVAudioFrameCount, tap: (AVAudioFormat) -> AVAudioNodeTapBlock) -> Bool {
        let audio = AVAudioSession.sharedInstance()
        do {
            try audio.setCategory(.playAndRecord, mode: .default, options: [.defaultToSpeaker, .allowBluetoothHFP, .duckOthers])
            try audio.setActive(true)
            let input = engine.inputNode
            let format = try MicTap.format(of: input)
            input.removeTap(onBus: 0)
            input.installTap(onBus: 0, bufferSize: bufferSize, format: format, block: tap(format))
            engine.prepare()
            try engine.start()
            return true
        } catch {
            engine.inputNode.removeTap(onBus: 0)
            if engine.isRunning { engine.stop() }
            try? audio.setActive(false, options: .notifyOthersOnDeactivation)
            return false
        }
    }

    /// Byte ranges of a recording's 20 s pieces; a tail under half a second stays with the piece before.
    static func pieces(of count: Int) -> [Range<Int>] {
        var starts = Array(stride(from: 0, to: count, by: pieceBytes))
        if starts.count > 1, let last = starts.last, count - last < minBytes { starts.removeLast() }
        return zip(starts, starts.dropFirst() + [count]).map { $0..<$1 }
    }

    /// The recording in pieces, a few transcribed at a time and joined in order.
    private static func transcribe(_ pcm: Data, provider: VoiceProvider, on store: BotStore) async throws -> String {
        // Asked for now, not at the start: a reconnect while recording replaces the client.
        let client = try await store.ready()
        let model = VoiceSettings.transcribeModel(provider)
        let wavs = pieces(of: pcm.count).map { MicTap.wav(pcm.subdata(in: $0)) }
        let texts = try await withThrowingTaskGroup(of: (Int, String).self) { group in
            var texts = [String](repeating: "", count: wavs.count)
            func keep(_ result: (Int, String)) { texts[result.0] = result.1.trimmingCharacters(in: .whitespacesAndNewlines) }
            for (i, wav) in wavs.enumerated() {
                if i >= inFlight, let done = try await group.next() { keep(done) }
                try Task.checkCancellation()
                group.addTask { (i, try await client.voiceTranscribe(provider, model: model, wav: wav)) }
            }
            for try await done in group { keep(done) }
            return texts
        }
        return VoiceSettings.inReadersScript(texts.filter { !$0.isEmpty }.reduce("", join))
    }

    private func recognized(_ text: String, final: Bool, generation: Int) {
        guard generation == self.generation else { return }
        let before = before
        edit? { _ in Self.join(before, text) }
        if final { stop() }
    }

    private nonisolated static func results(_ dictation: Dictation, generation: Int) -> @Sendable (SFSpeechRecognitionResult?, (any Error)?) -> Void {
        { result, error in
            let text = result?.bestTranscription.formattedString
            let final = result?.isFinal ?? false
            let failed = error != nil
            Task { @MainActor in
                if let text { dictation.recognized(text, final: final, generation: generation) }
                if failed, generation == dictation.generation { dictation.stop() }
            }
        }
    }

    private nonisolated static func authorized() async -> Bool {
        await withCheckedContinuation { done in
            SFSpeechRecognizer.requestAuthorization { done.resume(returning: $0 == .authorized) }
        }
    }
}
