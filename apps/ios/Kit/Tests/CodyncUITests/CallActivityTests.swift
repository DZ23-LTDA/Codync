import Foundation
import Testing
@testable import CodyncKit
@testable import CodyncUI

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
