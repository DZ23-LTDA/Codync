import Foundation
import Testing
@testable import CodyncKit
@testable import CodyncUI

@Test func callKitStaysOffWhereTheAppStoreIsMainlandChinas() {
    #expect(!SystemCallPolicy.allowed(storefront: "CHN", region: "US"))
    #expect(SystemCallPolicy.allowed(storefront: "TWN", region: "CN"))
    // Without a storefront (no App Store account), the region decides.
    #expect(!SystemCallPolicy.allowed(storefront: nil, region: "CN"))
    #expect(SystemCallPolicy.allowed(storefront: nil, region: "HK"))
    #expect(SystemCallPolicy.allowed(storefront: nil, region: nil))
}

@MainActor @Test func theLiveActivityFollowsTheCallsPhase() {
    #expect(CallActivity.state(.starting) == .connecting)
    #expect(CallActivity.state(.reconnecting) == .connecting)
    #expect(CallActivity.state(.listening) == .listening)
    #expect(CallActivity.state(.speaking) == .speaking)
    #expect(CallActivity.state(.interrupted) == .paused)
    #expect(CallActivity.state(.failed("x", .connection)) == nil)
}

@MainActor @Test func anActivityShowsTheCallOnlyWhileThisPhoneIsOnIt() throws {
    var bot = try #require(Bot.widgetPreview.first)
    bot.id = "call-activity-test"
    // The computer's push carries no call; one left on the activity by a call that's over goes.
    let pushed = BotActivityAttributes.ContentState(status: "working", activity: "", startedAt: nil)
    let leftover = CallActivity.merge(pushed, call: .listening, since: .now)
    #expect(CallActivity.withCall(leftover, botId: bot.id) == pushed)

    let older = CallActivity(bot: bot, computerId: "computer", link: nil)
    older.phase(.speaking)
    let shown = CallActivity.withCall(pushed, botId: bot.id)
    #expect(shown.call == .speaking && shown.callStartedAt != nil && shown.status == "working")
    // A newer call with the bot outlives the older one ending.
    let newer = CallActivity(bot: bot, computerId: "computer", link: nil)
    newer.phase(.listening)
    older.end()
    #expect(CallActivity.isLive(botId: bot.id))
    #expect(CallActivity.withCall(pushed, botId: bot.id).call == .listening)
    newer.end()
    #expect(!CallActivity.isLive(botId: bot.id))
    #expect(CallActivity.withCall(leftover, botId: bot.id) == pushed)
}

@MainActor @Test func callKitEndingTheCallHangsUpOnlyOnceTheCallWasLive() {
    var hungUp = 0
    func systemEnds(after setUp: (CallKitCall) -> Void) {
        let call = CallKitCall(id: UUID())
        call.onEnd = { hungUp += 1 }
        setUp(call)
        call.systemEnded()
    }
    // Ended before it got its audio or connected: CallKit dropped it (the Simulator), and the call goes on.
    systemEnds { _ in }
    #expect(hungUp == 0)
    systemEnds { $0.connected() }
    #expect(hungUp == 1)
    systemEnds { $0.audioActivated() }
    #expect(hungUp == 2)
    // The app hung up, or CallKit refused the call: nothing more to end.
    systemEnds { $0.connected(); $0.end() }
    systemEnds { $0.connected(); $0.declined() }
    #expect(hungUp == 2)
}

@MainActor @Test func theCallWaitsForCallKitsAudioAtMostAMoment() async throws {
    let timeout = CallKitCall.audioTimeout
    defer { CallKitCall.audioTimeout = timeout }
    let clock = ContinuousClock()

    CallKitCall.audioTimeout = .seconds(5)
    let call = CallKitCall(id: UUID())
    var start = clock.now
    let ready = Task { await call.audioReady() }
    try await Task.sleep(for: .milliseconds(50))
    call.audioActivated()
    await ready.value
    #expect(clock.now - start < .seconds(2))
    // Active already: no wait.
    start = clock.now
    await call.audioReady()
    #expect(clock.now - start < .seconds(1))
    // It never comes: the call goes on after the timeout.
    CallKitCall.audioTimeout = .milliseconds(50)
    start = clock.now
    await CallKitCall(id: UUID()).audioReady()
    #expect(clock.now - start < .seconds(2))
}

@MainActor @Test func callKitsHoldPausesTheCallAndItResumesOnceTheAudioIsBack() async {
    var heard: [String] = []
    let events = AudioSessionEvents(engine: nil, interrupted: { change in
        switch change {
        case .began: heard.append("paused")
        case .ended(let shouldResume): heard.append(shouldResume ? "resumed" : "ended")
        }
    })
    defer { events.stop() }
    func settle() async { try? await Task.sleep(for: .milliseconds(20)) }

    let call = CallKitCall(id: UUID())
    call.audioActivated()
    call.held(true)
    call.audioDeactivated()
    // Off hold before the system gave the audio back: the engine waits for it.
    call.held(false)
    await settle()
    #expect(heard == ["paused"])
    call.audioActivated()
    await settle()
    #expect(heard == ["paused", "resumed"])
    // Held without losing the audio: off hold resumes at once.
    call.held(true)
    call.held(false)
    await settle()
    #expect(heard == ["paused", "resumed", "paused", "resumed"])
}
