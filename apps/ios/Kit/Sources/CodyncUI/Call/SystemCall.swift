import AVFoundation
import CallKit
import Foundation
import StoreKit
import os

private let log = Logger(subsystem: "com.pokai.Codync", category: "Call")

/// Where CallKit may carry calls: not in apps offered in mainland China's App Store, which doesn't allow CallKit.
enum SystemCallPolicy {
    static func allowed(storefront: String?, region: String?) -> Bool {
        if let storefront { return storefront != "CHN" }
        return region != "CN"
    }

    static func allowed() async -> Bool {
        allowed(storefront: await Storefront.current?.countryCode, region: Locale.current.region?.identifier)
    }
}

/// The app's one CallKit provider; a call starts as an outgoing call to its bot.
@MainActor
final class CallKitCalls: NSObject {
    static let shared = CallKitCalls()

    private let provider: CXProvider
    private let controller = CXCallController()
    private var current: CallKitCall?

    private override init() {
        let config = CXProviderConfiguration()
        config.supportsVideo = false
        config.maximumCallGroups = 1
        config.maximumCallsPerCallGroup = 1
        config.supportedHandleTypes = [.generic]
        config.includesCallsInRecents = false
        provider = CXProvider(configuration: config)
        super.init()
        provider.setDelegate(self, queue: nil)
    }

    func start(id: UUID, name: String) -> CallKitCall {
        current?.systemEnded()
        let call = CallKitCall(id: id, calls: self)
        current = call
        CallAudioSession.systemRuns = true
        let start = CXStartCallAction(call: id, handle: CXHandle(type: .generic, value: name))
        controller.request(CXTransaction(action: start)) { error in
            guard let error else { return }
            Task { @MainActor in
                log.info("CallKit didn't take the call: \(error.localizedDescription, privacy: .public)")
                self.drop(call)
            }
        }
        return call
    }

    fileprivate func connected(_ call: CallKitCall) {
        provider.reportOutgoingCall(with: call.id, connectedAt: nil)
    }

    fileprivate func request(_ action: CXCallAction) {
        controller.request(CXTransaction(action: action)) { error in
            if let error { log.info("CallKit action failed: \(error.localizedDescription, privacy: .public)") }
        }
    }

    /// CallKit refused the call or lost it (the Simulator, CallKit unavailable, its daemon restarted): the call goes
    /// on without the system's.
    private func drop(_ call: CallKitCall) {
        call.declined()
        finished(call)
    }

    /// The call is over for CallKit (ended, or never taken): the engines manage the audio session again.
    private func finished(_ call: CallKitCall) {
        call.systemEnded()
        guard current === call else { return }
        current = nil
        CallAudioSession.systemRuns = false
    }
}

extension CallKitCalls: @preconcurrency CXProviderDelegate {
    func providerDidReset(_ provider: CXProvider) {
        if let current { drop(current) }
    }

    func provider(_ provider: CXProvider, perform action: CXStartCallAction) {
        do { try CallAudioSession.configure() } catch {
            log.error("call audio category: \(error.localizedDescription, privacy: .public)")
        }
        provider.reportOutgoingCall(with: action.callUUID, startedConnectingAt: nil)
        action.fulfill()
    }

    func provider(_ provider: CXProvider, perform action: CXEndCallAction) {
        if let current, current.id == action.callUUID { finished(current) }
        action.fulfill()
    }

    func provider(_ provider: CXProvider, perform action: CXSetMutedCallAction) {
        if let current, current.id == action.callUUID { current.onMute?(action.isMuted) }
        action.fulfill()
    }

    func provider(_ provider: CXProvider, perform action: CXSetHeldCallAction) {
        if let current, current.id == action.callUUID { current.held(action.isOnHold) }
        action.fulfill()
    }

    // Every activation has its deactivation, also after the call is over: WebRTC counts them.
    func provider(_ provider: CXProvider, didActivate audioSession: AVAudioSession) {
        CallAudioSession.systemActivated(audioSession)
        current?.audioActivated()
    }

    func provider(_ provider: CXProvider, didDeactivate audioSession: AVAudioSession) {
        CallAudioSession.systemDeactivated(audioSession)
        current?.audioDeactivated()
    }
}

/// The call as the system shows it (CallKit): on the Lock Screen and in the Dynamic Island, ended or muted from there
/// or from AirPods, held by a phone call, with the audio session the system activates for it.
@MainActor
final class CallKitCall {
    /// The system ended the call (Lock Screen, Dynamic Island, AirPods, another call answered instead).
    var onEnd: (() -> Void)?
    /// The system muted or unmuted the call.
    var onMute: ((Bool) -> Void)?
    let id: UUID
    private weak var calls: CallKitCalls?
    private var over = false
    /// The call got its audio session or connected: from here an end from the system is the user's.
    private var live = false
    private var audio = Audio.awaited
    /// Taken off hold while the system still had its audio: the engine picks up once it's back.
    private var resumesOnAudio = false
    private var waiting: [CheckedContinuation<Void, Never>] = []

    /// How long `audioReady` waits for the system's activation (it never comes on the Simulator).
    static var audioTimeout: Duration = .seconds(1.5)

    /// The call's audio session as the system hands it over.
    private enum Audio { case awaited, active, takenAway }

    /// `calls` is nil only in tests.
    init(id: UUID, calls: CallKitCalls? = nil) {
        self.id = id
        self.calls = calls
    }

    /// The call reached its voice.
    func connected() {
        guard !over else { return }
        live = true
        calls?.connected(self)
    }

    /// The app muted or unmuted the call.
    func setMuted(_ muted: Bool) {
        guard !over else { return }
        calls?.request(CXSetMutedCallAction(call: id, muted: muted))
    }

    /// The app hung up.
    func end() {
        guard !over else { return }
        over = true
        calls?.request(CXEndCallAction(call: id))
        resumeWaiting()
    }

    /// Returns once the system activated the call's audio session, or after `audioTimeout` if it doesn't.
    func audioReady() async {
        guard audio == .awaited, !over else { return }
        Task { [weak self] in
            try? await Task.sleep(for: Self.audioTimeout)
            self?.resumeWaiting()
        }
        await withCheckedContinuation { waiting.append($0) }
    }

    func audioActivated() {
        audio = .active
        live = true
        resumeWaiting()
        if resumesOnAudio {
            resumesOnAudio = false
            Self.post(held: false)
        }
    }

    func audioDeactivated() {
        audio = .takenAway
    }

    /// CallKit held the call (a phone call took over) or took it off hold: the engine pauses at once, and picks up
    /// again once the system has given the audio back, so it doesn't start on a session that isn't active.
    func held(_ onHold: Bool) {
        resumesOnAudio = !onHold && audio == .takenAway
        if !resumesOnAudio { Self.post(held: onHold) }
    }

    /// CallKit refused the call: it goes on without the system's.
    func declined() {
        over = true
        resumeWaiting()
    }

    /// CallKit ended the call. Before the call ever got its audio or connected, that's CallKit dropping it (the
    /// Simulator ends calls right after starting them) and the call goes on without the system's; after, the app
    /// hangs up too unless it started the ending.
    func systemEnded() {
        resumeWaiting()
        guard !over else { return }
        over = true
        if live { onEnd?() } else { log.info("CallKit ended the call before it connected; going on without it") }
    }

    private func resumeWaiting() {
        let continuations = waiting
        waiting = []
        for continuation in continuations { continuation.resume() }
    }

    private static func post(held: Bool) {
        NotificationCenter.default.post(name: .callHeld, object: nil, userInfo: ["held": held])
    }
}
