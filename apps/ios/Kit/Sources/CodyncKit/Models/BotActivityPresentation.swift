import Foundation

/// Presentation only; the ActivityKit payload and host push contract stay unchanged.
public struct BotActivityPresentation: Hashable, Sendable {
    public enum Phase: String, CaseIterable, Sendable {
        /// `sending` and `queued` are this phone's own (a message on its way, or waiting in the
        /// relay mailbox for an offline computer); the computer only pushes the others.
        case sending, queued, working, needsInput, completed, failed, stale, waiting
    }

    public let phase: Phase
    public let activity: String
    /// The voice call with the bot this phone is on (`ContentState.call`), if any.
    public let call: String?

    public init(status: String, activity: String, isStale: Bool = false, call: String? = nil) {
        let phase: Phase = switch status {
        case "sending": .sending
        case "queued": .queued
        case "working": .working
        case "needsInput": .needsInput
        case "idle", "completed": .completed
        case "error": .failed
        default: .waiting
        }
        let shown = isStale && (phase == .working || phase == .needsInput || phase == .sending) ? .stale : phase
        // On a call, a bot that isn't working or waiting on you shows the call instead of "Done".
        self.phase = call != nil && [.completed, .stale, .waiting].contains(shown) ? .waiting : shown
        self.activity = activity
        self.call = call
    }

    /// The call's state in words, while this phone is on one with the bot.
    public var callTitle: String? {
        switch call {
        case "connecting": "Connecting the call"
        case "speaking": "On a call · Speaking"
        case "paused": "Call paused"
        case nil: nil
        default: "On a call · Listening"
        }
    }

    public var title: String {
        if phase == .waiting, let callTitle { return callTitle }
        return switch phase {
        case .sending: "Sending"
        case .queued: "Waiting for your computer"
        case .working: "Working"
        case .needsInput: "Needs you"
        case .completed: "Done"
        case .failed: "Failed"
        case .stale: "Update delayed"
        case .waiting: "Waiting for update"
        }
    }

    /// The one line of text beside the orb: the bot's current step while it's live, else the state.
    public var caption: String {
        if phase == .waiting, let callTitle { return callTitle }
        return switch phase {
        case .working, .needsInput: activity.isEmpty ? title : activity
        case .sending, .queued, .completed, .failed, .stale, .waiting: title
        }
    }

    public var symbol: String {
        if phase == .waiting, call != nil { return "waveform" }
        return switch phase {
        case .completed: "checkmark"
        case .failed: "exclamationmark.triangle.fill"
        case .queued: "desktopcomputer"
        case .sending, .working, .needsInput, .stale, .waiting: "ellipsis"
        }
    }
}
