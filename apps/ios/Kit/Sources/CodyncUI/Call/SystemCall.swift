import AVFoundation
import CallKit
import Foundation
import StoreKit
import os

private let log = Logger(subsystem: "com.pokai.Codync", category: "Call")

/// The call as the system shows it (CallKit): on the Lock Screen and in the Dynamic Island, ended or muted from there
/// or from AirPods, held by a phone call, with the audio session the system activates for it.
@MainActor
protocol SystemCall: AnyObject {
    /// The system ended the call (Lock Screen, Dynamic Island, AirPods, another call answered instead).
    var onEnd: (() -> Void)? { get set }
    /// The system muted or unmuted the call.
    var onMute: ((Bool) -> Void)? { get set }
    /// The call reached its voice.
    func connected()
    /// The app muted or unmuted the call.
    func setMuted(_ muted: Bool)
    /// The app hung up.
    func end()
    /// Returns once the system activated the call's audio session, or after a moment if it doesn't (the Simulator).
    func audioReady() async
}

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

    func start(id: UUID, name: String) -> SystemCall {
        current?.systemEnded()
        let call = CallKitCall(id: id, calls: self)
        current = call
        CallAudioSession.systemRuns = true
        let start = CXStartCallAction(call: id, handle: CXHandle(type: .generic, value: name))
        controller.request(CXTransaction(action: start)) { error in
            guard let error else { return }
            Task { @MainActor in
                // The call goes on without the system's (the Simulator, CallKit unavailable).
                log.info("CallKit didn't take the call: \(error.localizedDescription, privacy: .public)")
                call.declined()
                self.finished(call)
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

    /// The call is over for CallKit (ended, or never taken): the engines manage the audio session again.
    fileprivate func finished(_ call: CallKitCall) {
        call.systemEnded()
        guard current === call else { return }
        current = nil
        CallAudioSession.systemRuns = false
    }
}

extension CallKitCalls: @preconcurrency CXProviderDelegate {
    /// CallKit dropped its calls (its daemon restarted, or it isn't available): the call goes on without the system's.
    func providerDidReset(_ provider: CXProvider) {
        guard let current else { return }
        current.declined()
        finished(current)
    }

    func provider(_ provider: CXProvider, perform action: CXStartCallAction) {
        // CallKit activates the session it finds configured.
        try? AVAudioSession.sharedInstance().setCategory(.playAndRecord, mode: .voiceChat,
                                                         options: [.defaultToSpeaker, .allowBluetoothHFP])
        provider.reportOutgoingCall(with: action.callUUID, startedConnectingAt: nil)
        action.fulfill()
    }

    func provider(_ provider: CXProvider, perform action: CXEndCallAction) {
        if let current, current.id == action.callUUID {
            // An end before the call ever got its audio or connected is CallKit dropping it (the Simulator ends
            // calls right after starting them): the call goes on without the system's.
            if !current.live {
                log.info("CallKit ended the call before it connected; going on without it")
                current.declined()
            }
            finished(current)
        }
        action.fulfill()
    }

    func provider(_ provider: CXProvider, perform action: CXSetMutedCallAction) {
        if let current, current.id == action.callUUID { current.onMute?(action.isMuted) }
        action.fulfill()
    }

    func provider(_ provider: CXProvider, perform action: CXSetHeldCallAction) {
        NotificationCenter.default.post(name: .callHeld, object: nil, userInfo: ["held": action.isOnHold])
        action.fulfill()
    }

    func provider(_ provider: CXProvider, didActivate audioSession: AVAudioSession) {
        current?.audioActivated()
    }

    func provider(_ provider: CXProvider, didDeactivate audioSession: AVAudioSession) {}
}

@MainActor
final class CallKitCall: SystemCall {
    var onEnd: (() -> Void)?
    var onMute: ((Bool) -> Void)?
    let id: UUID
    private weak var calls: CallKitCalls?
    private var over = false
    /// The call got its audio session or connected: from here an end from the system is the user's.
    fileprivate private(set) var live = false
    private var audioActive = false
    private var waiting: [CheckedContinuation<Void, Never>] = []

    fileprivate init(id: UUID, calls: CallKitCalls) {
        self.id = id
        self.calls = calls
    }

    func connected() {
        guard !over else { return }
        live = true
        calls?.connected(self)
    }

    func setMuted(_ muted: Bool) {
        guard !over else { return }
        calls?.request(CXSetMutedCallAction(call: id, muted: muted))
    }

    func end() {
        guard !over else { return }
        over = true
        calls?.request(CXEndCallAction(call: id))
        resumeWaiting()
    }

    func audioReady() async {
        guard !audioActive, !over else { return }
        Task { [weak self] in
            try? await Task.sleep(for: .seconds(1.5))
            self?.resumeWaiting()
        }
        await withCheckedContinuation { waiting.append($0) }
    }

    fileprivate func audioActivated() {
        audioActive = true
        live = true
        resumeWaiting()
    }

    /// CallKit refused the call: it goes on without the system's.
    fileprivate func declined() {
        over = true
        resumeWaiting()
    }

    /// CallKit ended the call; the app hangs up too unless it started the ending.
    fileprivate func systemEnded() {
        resumeWaiting()
        guard !over else { return }
        over = true
        onEnd?()
    }

    private func resumeWaiting() {
        let continuations = waiting
        waiting = []
        for continuation in continuations { continuation.resume() }
    }
}
