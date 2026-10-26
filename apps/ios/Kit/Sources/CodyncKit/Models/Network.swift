import Foundation

/// The computer's network of the user's computers (`networkView`, and the `network` event): read-only
/// on phones, since who is in is decided at a computer. Hosts before it answer "unknown method".
public struct NetworkView: Codable, Hashable, Sendable {
    public var joined: Bool
    /// This computer's name in the network.
    public var computer: String?
    public var role: Role?
    /// The user's cloud assistant may message this computer's bots.
    public var assistants: Bool?
    /// Cloud assistants can reach this computer at all: it's in a Codync account.
    public var assistantsAvailable: Bool?
    public var computers: [Computer]?

    public enum Role: String, Codable, Hashable, Sendable {
        case owner, admin, member
        /// Any role a newer host adds reads as a plain member.
        public init(from decoder: any Decoder) throws {
            self = Role(rawValue: try decoder.singleValueContainer().decode(String.self)) ?? .member
        }

        public var title: String {
            switch self {
            case .owner: "Owner"
            case .admin: "Admin"
            case .member: "Member"
            }
        }
    }

    public struct Computer: Codable, Hashable, Sendable, Identifiable {
        public var name: String
        public var role: Role
        public var me: Bool
        public var online: Bool
        /// The bots it offers, by address name and display name.
        public var agents: [Agent]?

        public var id: String { name }
    }

    public struct Agent: Codable, Hashable, Sendable, Identifiable {
        public var id: String
        public var name: String
    }

    /// Worth showing: in a network, or cloud assistants can reach this computer's bots.
    public var hasContent: Bool { joined || (assistantsAvailable == true && assistants == true) }
}
