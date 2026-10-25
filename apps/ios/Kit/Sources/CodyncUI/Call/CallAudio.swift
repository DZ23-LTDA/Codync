import AVFoundation
import Foundation
import os
@preconcurrency import WebRTC

private let log = Logger(subsystem: "com.pokai.Codync", category: "Call")

/// A realtime call's audio on the phone: the voice-processing microphone (16 kHz PCM16 up) and the speaker (24 kHz
/// PCM16 down). Tests use a scripted one.
@MainActor
protocol CallAudio: AnyObject {
    func permission() async -> Bool
    /// Starts the microphone, handing each buffer (base64 PCM16) and its loudness to `mic`, and the speaker.
    func start(mic: @escaping @MainActor (String, Float) -> Void) throws
    /// Plays a piece of the voice; `done` runs once it has been rendered.
    func play(_ pcm: Data, done: @escaping @MainActor () -> Void)
    /// Drops what's queued to play.
    func stopPlayback()
    /// Starts the microphone and speaker again after an interruption.
    func resume() throws
    func stop()
    /// Another app took the audio, or gave it back.
    var interruption: (@MainActor (AudioInterruption) -> Void)? { get set }
}

/// Another app's audio (a phone call, Siri, an alarm) pausing the call, and giving it back.
enum AudioInterruption {
    case began
    /// With whether the system says to pick up again on our own.
    case ended(shouldResume: Bool)
}

/// The call's audio session, set up the same way for every engine. Under CallKit the system activates it (and
/// deactivates it as its call ends or is held), and WebRTC's audio runs only while it's active; otherwise the engines
/// activate and release it.
@MainActor
enum CallAudioSession {
    /// The category options of every engine's audio, WebRTC's too.
    static let options: AVAudioSession.CategoryOptions = [.defaultToSpeaker, .allowBluetoothHFP]

    /// A CallKit call carries the audio (`CallKitCalls`). WebRTC then waits for the system's activation
    /// (`useManualAudio`) instead of activating the session itself, and deactivating it as a connection closes.
    static var systemRuns = false {
        didSet {
            let webRTC = RTCAudioSession.sharedInstance()
            webRTC.isAudioEnabled = false
            webRTC.useManualAudio = systemRuns
        }
    }

    /// Sets the call's category; CallKit activates the session it finds configured.
    static func configure() throws {
        try AVAudioSession.sharedInstance().setCategory(.playAndRecord, mode: .voiceChat, options: options)
    }

    /// Before an engine's audio starts or resumes: the session set up, and active unless the system activates it.
    static func activate() throws {
        try configure()
        guard !systemRuns else { return }
        try AVAudioSession.sharedInstance().setActive(true)
    }

    /// After an engine stops: deactivates the session unless the system runs the call.
    static func release() {
        guard !systemRuns else { return }
        try? AVAudioSession.sharedInstance().setActive(false, options: .notifyOthersOnDeactivation)
    }

    /// CallKit activated the session (the call started, or came off hold): WebRTC's audio may run.
    static func systemActivated(_ session: AVAudioSession) {
        let webRTC = RTCAudioSession.sharedInstance()
        webRTC.audioSessionDidActivate(session)
        webRTC.isAudioEnabled = true
    }

    /// CallKit deactivated the session (held, or ended).
    static func systemDeactivated(_ session: AVAudioSession) {
        let webRTC = RTCAudioSession.sharedInstance()
        webRTC.audioSessionDidDeactivate(session)
        webRTC.isAudioEnabled = false
    }
}

extension Notification.Name {
    /// CallKit put the call on hold (`held` true: a phone call took over) or gave it its audio back after one.
    static let callHeld = Notification.Name("CodyncCallHeld")
}

/// The audio session's events a call follows: interruptions (also CallKit holding the call), and `engine`
/// reconfiguring (a route change such as AirPods, voice processing settling), which stops it.
@MainActor
final class AudioSessionEvents {
    private var observers: [any NSObjectProtocol] = []

    init(engine: AVAudioEngine?, interrupted: @escaping @MainActor (AudioInterruption) -> Void,
         reconfigured: @escaping @MainActor () -> Void = {}) {
        let center = NotificationCenter.default
        observers.append(center.addObserver(forName: AVAudioSession.interruptionNotification, object: nil, queue: .main) { note in
            let type = (note.userInfo?[AVAudioSessionInterruptionTypeKey] as? UInt).flatMap(AVAudioSession.InterruptionType.init)
            let options = AVAudioSession.InterruptionOptions(rawValue: note.userInfo?[AVAudioSessionInterruptionOptionKey] as? UInt ?? 0)
            MainActor.assumeIsolated {
                switch type {
                case .began: interrupted(.began)
                case .ended: interrupted(.ended(shouldResume: options.contains(.shouldResume)))
                default: break
                }
            }
        })
        observers.append(center.addObserver(forName: .callHeld, object: nil, queue: .main) { note in
            let held = note.userInfo?["held"] as? Bool == true
            MainActor.assumeIsolated { interrupted(held ? .began : .ended(shouldResume: true)) }
        })
        if let engine {
            observers.append(center.addObserver(forName: .AVAudioEngineConfigurationChange, object: engine, queue: .main) { _ in
                MainActor.assumeIsolated { reconfigured() }
            })
        }
    }

    func stop() {
        for observer in observers { NotificationCenter.default.removeObserver(observer) }
        observers = []
    }
}

/// The phone's audio for Gemini Live: an echo-cancelled (voice-processing) microphone tap and a player node.
final class VoiceProcessingAudio: CallAudio {
    var interruption: (@MainActor (AudioInterruption) -> Void)?
    private let engine = AVAudioEngine()
    private let player = AVAudioPlayerNode()
    private var started = false
    private var events: AudioSessionEvents?
    private var mic: (@MainActor (String, Float) -> Void)?

    private static let playFormat = AVAudioFormat(commonFormat: .pcmFormatFloat32, sampleRate: 24000, channels: 1, interleaved: false)

    func permission() async -> Bool { await AVAudioApplication.requestRecordPermission() }

    func start(mic: @escaping @MainActor (String, Float) -> Void) throws {
        self.mic = mic
        try CallAudioSession.activate()
        try engine.inputNode.setVoiceProcessingEnabled(true)
        engine.attach(player)
        // The engine stops itself when it reconfigures; without a restart the microphone goes quiet.
        events = AudioSessionEvents(engine: engine, interrupted: { [weak self] in self?.interruption?($0) },
                                    reconfigured: { [weak self] in self?.restart() })
        try run()
        started = true
    }

    /// Wires the speaker and taps the microphone for the current hardware, then starts the engine.
    private func run() throws {
        // Wired again on every start: after the engine reconfigures, the old connections stay in place but
        // play nothing, so the player never finishes its buffers and replies wait forever (#214).
        engine.connect(player, to: engine.mainMixerNode, format: Self.playFormat)
        engine.connect(engine.mainMixerNode, to: engine.outputNode, format: nil)
        let input = engine.inputNode
        input.removeTap(onBus: 0)
        let format = try MicTap.format(of: input)
        input.installTap(onBus: 0, bufferSize: 2048, format: format, block: MicTap.pcm16k(from: format) { [weak self] pcm, level in
            let audio = pcm.base64EncodedString()
            Task { @MainActor in self?.mic?(audio, level) }
        })
        engine.prepare()
        try engine.start()
        player.play()
    }

    private func restart() {
        guard started, !engine.isRunning else { return }
        log.info("call audio engine reconfigured; restarting")
        do { try run() } catch { log.error("call audio restart: \(error.localizedDescription, privacy: .public)") }
    }

    func resume() throws {
        try CallAudioSession.activate()
        if !engine.isRunning { try run() }
    }

    func play(_ pcm: Data, done: @escaping @MainActor () -> Void) {
        guard let format = Self.playFormat, pcm.count >= 2 else { return }
        let frames = pcm.count / 2
        guard let buffer = AVAudioPCMBuffer(pcmFormat: format, frameCapacity: AVAudioFrameCount(frames)),
              let out = buffer.floatChannelData?[0] else { return }
        buffer.frameLength = AVAudioFrameCount(frames)
        pcm.withUnsafeBytes { raw in
            for i in 0..<frames {
                let sample = Int16(littleEndian: raw.loadUnaligned(fromByteOffset: i * 2, as: Int16.self))
                out[i] = Float(sample) / 32768
            }
        }
        player.scheduleBuffer(buffer, completionCallbackType: .dataRendered) { _ in
            Task { @MainActor in done() }
        }
    }

    func stopPlayback() {
        guard started else { return }
        player.stop()
        // A stopped engine (mid-restart, or failing) would throw; restarting plays the player again.
        if engine.isRunning { player.play() }
    }

    func stop() {
        events?.stop()
        events = nil
        if started {
            player.stop()
            engine.inputNode.removeTap(onBus: 0)
            engine.stop()
            started = false
        }
        CallAudioSession.release()
    }
}
