import CodyncKit
import CodyncUI
import SwiftUI

/// A computer this iPhone can use: its connection, how it's reached, and whether it's in the account.
struct ComputerRow: View {
    let store: BotStore
    let inAccount: CloudComputer?
    let openScreen: () -> Void
    /// Agents, connectors and skills installed on this computer.
    let openMarketplace: () -> Void
    let ask: (() -> Void)?
    /// Set when this iPhone's access to the computer can be revoked.
    let revoke: (() -> Void)?
    let remove: () -> Void
    @Environment(AccountStore.self) private var accounts
    @State private var coloring = false
    @State private var voiceSettings = false

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            summary
            ForEach(actions) { action in
                Hairline().padding(.top, 14)
                actionControl(action)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(.top, 14)
            }
        }
        .padding(.vertical, 2)
    }

    /// Name, connection, and an access request when this iPhone can't reach the computer.
    private var summary: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 10) {
                ComputerBadge(store.computer, size: 32)
                Text(store.hostName).font(.body.weight(.semibold)).foregroundStyle(Palette.text)
                    .frame(maxWidth: .infinity, alignment: .leading)
                if inAccount != nil {
                    Image(systemName: "person.crop.circle.badge.checkmark")
                        .foregroundStyle(Palette.tertiary)
                        .accessibilityLabel(L10n.tr("In your account"))
                }
            }
            connectionDetail
            if store.isOffline, let ask {
                Button(accounts.pendingAccess[store.computer.id] == nil ? "Ask for access" : "Show code", action: ask)
                    .buttonStyle(.secondary)
            }
        }
        .contentShape(Rectangle())
        .contextActions {
            var items = [
                MenuItem("Color", icon: "paintpalette") { coloring = true },
            ]
            if store.connection == .online {
                items.append(MenuItem("Extensions", icon: "square.grid.2x2", action: openMarketplace))
                items.append(MenuItem("Voice chat", icon: "waveform") { voiceSettings = true })
            }
            if store.screen != nil, store.connection == .online {
                items.append(MenuItem("Screen", icon: "display", action: openScreen))
            }
            if let revoke {
                items.append(MenuItem("Revoke this iPhone's access", icon: "lock.slash", destructive: true, divider: true, action: revoke))
            }
            if inAccount == nil {
                items.append(MenuItem("Remove", icon: "trash", destructive: true, divider: revoke == nil, action: remove))
            }
            return items
        }
        .codyncSheet(isPresented: $voiceSettings) { VoiceChatSettingsView().environment(store) }
        .codyncOverlay(isPresented: $coloring) { close in
            ZStack {
                Color.black.opacity(0.35).ignoresSafeArea().onTapGesture(perform: close)
                SwatchPanel(selected: store.computer.color) { id in
                    close()
                    accounts.setColor(store.computer.id, id)
                }
                .background(Palette.bubbleAgent, in: RoundedRectangle(cornerRadius: 22, style: .continuous))
                .shadow(color: .black.opacity(0.25), radius: 24, y: 10)
            }
            .accessibilityAddTraits(.isModal)
        }
    }

    private enum Action: String, Identifiable {
        case extensions, screen, connection, remove
        var id: String { rawValue }
    }

    /// Extensions, Screen and Connection each take their own row under the computer.
    private var actions: [Action] {
        var rows: [Action] = []
        if store.connection == .online {
            rows.append(.extensions)
            if store.screen != nil { rows.append(.screen) }
        }
        rows.append(.connection)
        if stale, inAccount == nil { rows.append(.remove) }
        return rows
    }

    @ViewBuilder private func actionControl(_ action: Action) -> some View {
        switch action {
        case .extensions:
            Button(action: openMarketplace) {
                RowLabel("Extensions", systemImage: "square.grid.2x2", chevron: true)
            }
            .buttonStyle(.plain)
        case .screen:
            Button(action: openScreen) {
                RowLabel("Screen", systemImage: "display", chevron: true)
            }
            .buttonStyle(.plain)
        case .connection:
            DropdownMenu {
                ConnectionRoute.allCases.map { route in
                    MenuItem(RelayCopy.routeTitle(route, cloud: store.computer.cloud), icon: route.icon, selected: route == preference) {
                        accounts.setRoute(store.computer.id, route)
                    }
                }
            } label: {
                HStack(spacing: 12) {
                    Image(systemName: preference.icon).font(.system(size: 17)).frame(width: 26)
                    Text(L10n.tr("Connection"))
                    Spacer(minLength: 8)
                    Image(systemName: "chevron.up.chevron.down").font(.caption.weight(.semibold)).foregroundStyle(Palette.tertiary)
                }
                .foregroundStyle(Palette.text)
                .contentShape(Rectangle())
            }
        case .remove:
            Button(action: remove) {
                RowLabel("Remove", systemImage: "trash", tint: Palette.danger)
            }
            .buttonStyle(.plain)
        }
    }

    private var connectionDetail: some View {
        Text(L10n.tr(detail)).font(.footnote)
            .foregroundStyle(store.isOffline ? Palette.warning : Palette.secondary)
            .fixedSize(horizontal: false, vertical: true)
    }

    private var preference: ConnectionRoute { store.computer.route ?? .automatic }

    private var stale: Bool { accounts.isStale(store.computer.id) }

    private var detail: String {
        guard store.connection == .online else { return accounts.statusText(store) }
        let count = store.roster.count
        let route = RelayCopy.detailSuffix(route: store.hostRoute, cloud: store.computer.cloud)
        return "\(count) bot\(count == 1 ? "" : "s")\(route)"
    }
}

/// A computer in the account that this iPhone can't use yet.
struct AccountComputerRow: View {
    let computer: CloudComputer
    let ticket: AccessTicket?
    let ask: () -> Void
    let revoke: () -> Void

    var body: some View {
        HStack(spacing: 12) {
            ComputerBadge(Computer(id: computer.computerId, name: computer.name, signKey: computer.signKey, device: computer.device), size: 40)
                .opacity(0.5)
            VStack(alignment: .leading, spacing: 2) {
                Text(computer.name).font(.body.weight(.semibold)).foregroundStyle(Palette.text)
                Text(L10n.tr(detail)).font(.subheadline).foregroundStyle(Palette.secondary).lineLimit(2)
            }
            Spacer()
            if ticket != nil {
                Button(L10n.tr("Code"), action: ask).buttonStyle(.secondary)
            } else if computer.isGranted {
                Button(L10n.tr("Revoke"), role: .destructive, action: revoke).buttonStyle(.secondary)
            } else {
                Button(L10n.tr("Ask for access"), action: ask).buttonStyle(.secondary)
            }
        }
        .padding(.vertical, 2)
    }

    private var detail: String {
        let online = computer.isOnline ? "Online" : "Offline"
        if let ticket { return "Waiting for your OK on the computer · \(ticket.code)" }
        return switch computer.access {
        case "granted": "Approved for an earlier install of Codync. Revoke it, then ask again."
        case "pending": "\(online) · A request is waiting"
        default: "\(online) · In your account"
        }
    }
}

/// An older account identity stays visible; only account management can remove it.
struct OlderCopyRow: View {
    let computer: CloudComputer

    var body: some View {
        HStack(spacing: 12) {
            ComputerBadge(Computer(id: computer.computerId, name: computer.name, signKey: computer.signKey, device: computer.device), size: 40)
                .opacity(0.35)
            VStack(alignment: .leading, spacing: 2) {
                Text(computer.name).font(.body.weight(.semibold)).foregroundStyle(Palette.secondary)
                Text(L10n.tr(detail)).font(.subheadline).foregroundStyle(Palette.tertiary).lineLimit(2)
            }
            Spacer()
        }
        .padding(.vertical, 2)
    }

    private var detail: String {
        let seen = computer.lastSeenAt.map { " · last seen \(RelativeTime.day(Date(milliseconds: $0)))" } ?? ""
        return "Older copy of this computer\(seen)"
    }
}

/// A navigation row label with a trailing chevron.
struct LinkRow<Content: View>: View {
    @ViewBuilder let content: Content

    var body: some View {
        HStack {
            content.foregroundStyle(Palette.text)
            Spacer(minLength: 8)
            Image(systemName: "chevron.right").font(.caption.weight(.semibold)).foregroundStyle(Palette.tertiary)
        }
        .contentShape(Rectangle())
    }
}

/// The computer color choices: a grid of colored dots, the current one checked.
private struct SwatchPanel: View {
    let selected: String?
    let pick: (String) -> Void

    var body: some View {
        LazyVGrid(columns: Array(repeating: GridItem(.fixed(36), spacing: 10), count: 6), spacing: 10) {
            ForEach(AvatarPalette.colors) { swatch in
                Button { pick(swatch.id) } label: {
                    Circle().fill(swatch.color)
                        .frame(width: 32, height: 32)
                        .overlay {
                            if selected == swatch.id {
                                Image(systemName: "checkmark").font(.caption.weight(.bold)).foregroundStyle(.white)
                            }
                        }
                }
                .buttonStyle(.plain)
                .accessibilityLabel(swatch.label)
                .accessibilityAddTraits(selected == swatch.id ? .isSelected : [])
            }
        }
        .padding(14)
    }
}

private extension ConnectionRoute {
    var icon: String {
        switch self {
        case .automatic: "arrow.triangle.branch"
        case .cloudflareFirst: "cloud"
        case .directOnly: "wifi"
        }
    }
}
