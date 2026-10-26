import CodyncKit
import Foundation

/// An update this phone asked of a computer that's behind (`installHostUpdate`).
public enum HostUpdate: Equatable, Sendable {
    case requested(at: Date)
    case failed(String)
}

/// What the update notice offers for a computer that's behind (`.updateHost`).
public enum HostUpdateOffer: Equatable, Sendable {
    /// Asked within `BotStore.hostUpdateWait`: the computer is updating.
    case updating
    /// The computer updates when asked; `last` is a request that didn't (it failed, or the
    /// computer never came back updated).
    case available(last: HostUpdate?)
    /// Not from here: the notice says how to update it on the computer.
    case unavailable
}

/// The Codync row on a computer's card in Computer management: the version it runs, and an update
/// when one is due (a newer release, or a version too old for this app).
public struct HostVersionLine: Equatable, Sendable {
    public var version: String
    /// Sentences, each its own phrase to translate.
    public var detail: [String]
    /// The detail is a problem: the computer is too old for this app, or its update didn't take.
    public var warns = false
    /// The button that asks the computer to update; nil when this phone can't, or it's updating.
    public var action: String?
    public var updating = false

    /// `latest` is the latest release the computer found (nil while it doesn't know: then no verdict).
    public static func make(version: String, latest: String?, mismatch: VersionMismatch?, offer: HostUpdateOffer,
                            host: String, os: String?) -> HostVersionLine {
        if offer == .updating { return HostVersionLine(version: version, detail: [UpdateNeededText.updating(host: host)], updating: true) }
        let minimum: String? = if case let .updateHost(_, minimum) = mismatch { minimum } else { nil }
        let available = latest.flatMap { AppVersion.isBelow(version, $0) ? $0 : nil }
        let due = minimum.map { "This app needs \($0) or newer." } ?? available.map { "\($0) available" }
        guard let due else { return HostVersionLine(version: version, detail: latest == nil ? [] : ["Up to date"]) }
        switch offer {
        case let .available(last?):
            return HostVersionLine(version: version, detail: [UpdateNeededText.notUpdated(last, host: host)], warns: true, action: "Try again")
        case .available(nil):
            return HostVersionLine(version: version, detail: [due], warns: minimum != nil, action: "Update")
        case .unavailable, .updating:
            return HostVersionLine(version: version, detail: [due, UpdateNeededText.how(os: os)], warns: minimum != nil)
        }
    }
}

extension BotStore {
    /// This computer's Codync row; nil until its `hello` arrives.
    public var hostVersionLine: HostVersionLine? {
        guard let hello else { return nil }
        return .make(version: hello.version, latest: hello.latestVersion, mismatch: mismatch,
                     offer: hostUpdateOffer(), host: hostName, os: hello.os)
    }

    /// How long an update may take before the notice offers it again.
    static let hostUpdateWait: TimeInterval = 10 * 60
    /// While it updates, `hello` is asked this often, so the notice goes away soon after.
    static let updatingRecheck: Duration = .seconds(5)

    public func hostUpdateOffer(now: Date = .now) -> HostUpdateOffer {
        if case let .requested(at) = hostUpdate, now.timeIntervalSince(at) < Self.hostUpdateWait { return .updating }
        return hello?.canUpdate == true ? .available(last: hostUpdate) : .unavailable
    }

    /// Asks the computer to update itself; the notice follows along until its version changes.
    public func requestHostUpdate() {
        Motion.animate { hostUpdate = .requested(at: .now) }
        Task {
            do {
                try await withLink { try await $0.installHostUpdate() }
            } catch {
                Motion.animate { hostUpdate = .failed(error.localizedDescription) }
            }
        }
    }
}
