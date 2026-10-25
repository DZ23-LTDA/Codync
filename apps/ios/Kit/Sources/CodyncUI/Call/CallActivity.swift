import ActivityKit
import CodyncKit
import Foundation

/// The bot's Live Activity while this phone is on a call with it: the call's state and length, and the bot's latest
/// progress from the computer. Uses the bot's activity if one is up (a task it was given), else starts one for the call.
@MainActor
public final class CallActivity {
    /// Bots this phone is on a call with now: an activity's call fields outlive a killed app otherwise.
    private static var live: Set<String> = []

    /// Whether this phone is on a call with the bot (the activity's call fields are current).
    public static func isLive(botId: String) -> Bool { live.contains(botId) }

    private let attributes: BotActivityAttributes
    private let startedAt = Date.now
    /// The activity this call started itself (none was up), ended with the call.
    private var ownID: String?
    private var call: String?
    private var progress: String?

    init(bot: Bot, computerId: ComputerID, link: URL?) {
        attributes = BotActivityAttributes(bot: bot, computerId: computerId, link: link)
        Self.live.insert(bot.id)
    }

    /// `ContentState.call` for a phase: nil once the call can't go on.
    static func state(_ phase: VoicePhase) -> String? {
        switch phase {
        case .starting, .reconnecting: "connecting"
        case .listening: "listening"
        case .speaking: "speaking"
        case .interrupted: "paused"
        case .failed: nil
        }
    }

    func phase(_ phase: VoicePhase) {
        let state = Self.state(phase)
        guard state != call else { return }
        call = state
        show()
    }

    /// What the bot is doing, from the computer's progress cues.
    func progress(_ text: String) {
        progress = text
        show()
    }

    func end() {
        call = nil
        Self.live.remove(attributes.botId)
        let (botId, computerId, own) = (attributes.botId, attributes.computerId, ownID)
        Task.detached {
            guard let activity = Self.find(botId: botId, computerId: computerId) else { return }
            if activity.id == own {
                await activity.end(nil, dismissalPolicy: .immediate)
            } else {
                var state = activity.content.state
                state.call = nil
                state.callStartedAt = nil
                await activity.update(.init(state: state, staleDate: activity.content.staleDate))
            }
        }
    }

    private func show() {
        let (botId, computerId) = (attributes.botId, attributes.computerId)
        let (call, startedAt, progress) = (call, startedAt, progress)
        if Self.find(botId: botId, computerId: computerId) == nil {
            guard call != nil, UserDefaults.standard.object(forKey: "liveActivitiesEnabled") as? Bool ?? true,
                  ActivityAuthorizationInfo().areActivitiesEnabled else { return }
            let state = BotActivityAttributes.ContentState(status: "idle", activity: progress ?? "", startedAt: nil,
                                                           call: call, callStartedAt: startedAt)
            ownID = try? Activity.request(attributes: attributes, content: .init(state: state, staleDate: nil), pushType: nil).id
            return
        }
        Task.detached {
            guard let activity = Self.find(botId: botId, computerId: computerId) else { return }
            var state = activity.content.state
            state.call = call
            state.callStartedAt = startedAt
            if let progress { state.activity = progress }
            await activity.update(.init(state: state, staleDate: nil))
        }
    }

    private nonisolated static func find(botId: String, computerId: ComputerID) -> Activity<BotActivityAttributes>? {
        Activity<BotActivityAttributes>.activities.first { $0.attributes.botId == botId && $0.attributes.computerId == computerId }
    }
}
