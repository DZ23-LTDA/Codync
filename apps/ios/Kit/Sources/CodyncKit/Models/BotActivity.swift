import ActivityKit
import Foundation

/// Live Activity for a bot working on something the user just delegated.
/// Content state keys must match what codync-host pushes (host/src/remote/push.rs).
public struct BotActivityAttributes: ActivityAttributes {
    public struct ContentState: Codable, Hashable, Sendable {
        /// idle | working | needsInput | error
        public var status: String
        /// Local app updates may show the current step; remote pushes leave it empty (§6.7).
        public var activity: String
        public var startedAt: Date?
        /// A voice call with the bot this phone is on: connecting | listening | speaking | paused. Set by the app only.
        public var call: String?
        /// When that call started, for its length.
        public var callStartedAt: Date?

        public init(status: String, activity: String, startedAt: Date?, call: String? = nil, callStartedAt: Date? = nil) {
            self.status = status
            self.activity = activity
            self.startedAt = startedAt
            self.call = call
            self.callStartedAt = callStartedAt
        }
    }

    public var link: URL?
    public var computerId: ComputerID
    public var botId: String
    public var name: String
    public var avatarShape: String
    public var avatarColor: String

    public init(bot: Bot, computerId: ComputerID, link: URL? = nil) {
        self.link = link
        self.computerId = computerId
        botId = bot.id
        name = bot.name
        avatarShape = bot.avatarShape
        avatarColor = bot.avatarColor
    }
}
