import CodyncKit
import CodyncUI
import SwiftUI

/// Every computer this iPhone can use or ask for — paired here and in the account, merged by
/// computer ID — plus settings for the selected account.
struct SettingsView: View {
    /// Pushed on top of another screen rather than opened as its own sheet.
    var pushed = false
    /// The More tab: voice, appearance and the other device settings. The sheet is computers only.
    var embedded = false
    @Environment(AppStore.self) private var app
    @Environment(AccountStore.self) private var accounts
    @Environment(\.dismissModal) private var dismissModal
    @Environment(\.openURL) private var openURL
    @State private var notificationsAllowed: Bool?
    @AppStorage("notificationsEnabled") private var notificationsEnabled = true
    @AppStorage(Analytics.enabledKey) private var analyticsEnabled = true
    @State private var addingComputer = false
    @State private var confirmForget: Computer?
    @State private var confirmRevoke: CloudComputer?
    @State private var access: AccessTarget?
    @State private var confirmStartOver = false
    @State private var voice: VoiceTarget?
    @State private var aboutAnalytics = false

    var body: some View {
        cards
            .refreshable { await accounts.refreshCloud() }
            .task { await accounts.refreshCloud() }
            .task { notificationsAllowed = await PushRegistrar.shared.isAllowed() }
            .codyncSheet(isPresented: $addingComputer) {
                PairingView(inModal: true)
            }
            .codyncSheet(item: $voice) { target in VoiceChatSettingsView().environment(target.store) }
            .codyncSheet(isPresented: $aboutAnalytics) { AnalyticsSheet() }
            .codyncSheet(item: $access) { target in
                AccessRequestView(computer: target.computer, pending: accounts.pendingAccess[target.id])
            }
            .codyncDialog(L10n.tr("Remove {name}?").replacingOccurrences(of: "{name}", with: confirmForget?.name ?? "computer"),
                          isPresented: Binding(get: { confirmForget != nil }, set: { if !$0 { confirmForget = nil } }),
                          message: "Its bots and conversations stay on that computer. You can pair again any time.") {
                [DialogAction("Remove", destructive: true) {
                    if let c = confirmForget { Task { await app.removeComputer(c.id) } }
                }]
            }
            .codyncDialog("Revoke this iPhone's access to \(confirmRevoke?.name ?? "the computer")?",
                          isPresented: Binding(get: { confirmRevoke != nil }, set: { if !$0 { confirmRevoke = nil } }),
                          message: "You can ask for access again; the computer will show a new code to confirm.") {
                [DialogAction("Revoke", destructive: true) {
                    if let c = confirmRevoke { Task { await app.revokeAccess(c.computerId) } }
                }]
            }
            // What failed here (removing, revoking, refreshing) shows over this sheet, not under it at the root,
            // where the same error clears with this one. In place: a failure right after a confirmation comes
            // while that dialog's cover is still fading out, and a second cover can't present over it.
            .codyncDialog("Something went wrong", isPresented: Binding(
                get: { accounts.lastError != nil }, set: { if !$0 { accounts.lastError = nil } }
            ), message: accounts.lastError, cancel: "OK", inPlace: true) { [] }
    }

    private var cards: some View {
        let form = CardForm {
            if embedded {
                Text(L10n.tr("Settings"))
                    .font(.largeTitle.bold())
                    .foregroundStyle(Palette.text)
                    .accessibilityAddTraits(.isHeader)
            } else {
            CardSection(footer: app.account.isSignedIn
                        ? "Computers in your account need your OK on the computer before this iPhone can use them."
                        : "Each bot runs on its own computer. Sign in to see the computers in your account.") {
                ForEach(accounts.computers) { computer in
                    if let store = accounts.store(for: computer.id) {
                        ComputerRow(store: store, inAccount: cloudComputer(computer.id),
                                    openScreen: { openScreen(store) },
                                    openMarketplace: { closeSheets { app.marketplace = store.computer.id } },
                                    ask: cloudComputer(computer.id).map { c in { access = AccessTarget(computer: c) } },
                                    revoke: cloudComputer(computer.id).flatMap { c in c.isGranted ? { confirmRevoke = c } : nil },
                                    remove: { confirmForget = computer })
                    }
                }
                ForEach(accountOnly) { computer in
                    if accounts.isOlderCopy(computer) {
                        OlderCopyRow(computer: computer)
                    } else {
                        AccountComputerRow(computer: computer, ticket: accounts.pendingAccess[computer.computerId],
                                           ask: { access = AccessTarget(computer: computer) },
                                           revoke: { confirmRevoke = computer })
                    }
                }
                Button {
                    addingComputer = true
                } label: {
                    Label(L10n.tr("Pair a computer"), systemImage: "qrcode.viewfinder")
                        .foregroundStyle(Palette.text)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
            }
            }

            if embedded {
            let online = accounts.computers.compactMap { accounts.store(for: $0.id) }.filter { $0.connection == .online }
            if !online.isEmpty {
                CardSection("Voice chat", footer: "Apple speech on this iPhone, or OpenAI and Gemini on your own API key, kept on the computer.") {
                    ForEach(online, id: \.computer.id) { store in
                        Button { voice = VoiceTarget(store: store) } label: {
                            LinkRow {
                                Label(online.count > 1 ? "Voice chat on \(store.hostName)" : L10n.tr("Voice and API keys"), systemImage: "waveform")
                            }
                        }
                        .buttonStyle(.plain)
                    }
                }
            }

            AppearanceSettings()

            LanguageSettings()

            ThinkingDetailsSettings()

            CardSection("Notifications", footer: "Get a result summary, a request for input, or a failure notice. Notification previews follow your iOS settings.") {
                Toggle(isOn: $notificationsEnabled) { Text(L10n.tr("Push notifications")) }
                    .toggleStyle(.codync)
                    .onChange(of: notificationsEnabled) { _, on in
                        PushRegistrar.shared.resync()
                        Analytics.shared.capture(.settingChanged(.push, on: on))
                    }
                notificationsRow
            }

            CardSection(footer: "Which features you use, never your messages, code or files. Helps us spot bugs sooner and make Codync better for you.") {
                HStack(spacing: 4) {
                    Text(L10n.tr("Share usage analytics")).lineLimit(1).layoutPriority(1)
                    IconButton("About usage analytics", systemImage: "info.circle") { aboutAnalytics = true }
                    Toggle(isOn: Binding(get: { analyticsEnabled }, set: { Analytics.shared.setEnabled($0) })) { EmptyView() }
                        .toggleStyle(.codync)
                        .accessibilityLabel(L10n.tr("Share usage analytics"))
                }
            }

            if !hiddenBots.isEmpty {
                CardSection("Hidden bots") {
                    ForEach(hiddenBots) { item in
                        let store = item.store, bot = item.bot
                        HStack {
                            CharacterAvatar(bot: bot, size: 28, animated: false)
                            VStack(alignment: .leading, spacing: 1) {
                                Text(bot.name)
                                Text(store.hostName).font(.caption).foregroundStyle(Palette.tertiary)
                            }
                            Spacer()
                            IconButton("Unhide", systemImage: "eye") {
                                withAnimation(Motion.layout) { store.setHidden(bot, false) }
                            }
                        }
                    }
                }
            }

            CardSection {
                ValueRow("App version", value: Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "")
                Button {
                    Analytics.shared.capture(.linkOpened(.github))
                    openURL(URL(string: "https://github.com/leepokai/Codync")!)
                } label: {
                    Text(L10n.tr("Source code"))
                }
                    .buttonStyle(.plain)
                    .foregroundStyle(Palette.text)
            }

            CardSection {
                Button { confirmStartOver = true } label: {
                    Label(L10n.tr("Start over"), systemImage: "arrow.counterclockwise")
                        .foregroundStyle(Palette.danger)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
            }
            }
        }
        .codyncDialog("Start over?", isPresented: $confirmStartOver,
                      message: "Signs out of every account and forgets every computer on this iPhone, then shows the welcome again. Your computers keep their bots and chats; pair again to use them.") {
            [DialogAction("Start over", destructive: true) { Task { await app.startOver() } }]
        }
        return PresentedSettings(embedded: embedded, pushed: pushed) { form }
    }

    /// More shows device settings. The sheet and a push add the Computers & settings header.
private struct PresentedSettings<Content: View>: View {
    let embedded: Bool
    let pushed: Bool
    @ViewBuilder let content: Content

    var body: some View {
        if embedded {
            content
        } else {
            content
                .page("Computer management", pushed: pushed, icon: "gearshape")
                // A push → settings, the Computers sheet → computers. The More tab tracks itself.
                .tracked(.screenViewed(pushed ? .settings : .computers))
        }
    }
}


    private struct VoiceTarget: Identifiable {
        let store: BotStore
        let id: String

        @MainActor init(store: BotStore) {
            self.store = store
            id = store.computer.id
        }
    }

    private struct AccessTarget: Identifiable {
        let computer: CloudComputer
        var id: ComputerID { computer.computerId }
    }

    private func cloudComputer(_ id: ComputerID) -> CloudComputer? {
        accounts.cloudComputers.first { $0.computerId == id }
    }

    /// Account computers this iPhone isn't set up for yet.
    private var accountOnly: [CloudComputer] {
        accounts.cloudComputers.filter { c in !accounts.computers.contains { $0.id == c.computerId } }
    }

    private struct HiddenBot: Identifiable {
        let store: BotStore
        let bot: Bot
        let id: String
    }

    private var hiddenBots: [HiddenBot] {
        accounts.computers.compactMap { accounts.store(for: $0.id) }.flatMap { store in
            store.hiddenBots.map { HiddenBot(store: store, bot: $0, id: "\(store.computer.id)/\($0.id)") }
        }
    }

    /// The Computers sheet slides away before the marketplace or screen opens over the app. On the More
    /// tab those open immediately.
    private func closeSheets(then open: @escaping @MainActor () -> Void) {
        guard !embedded else { open(); return }
        dismissModal()
        Task {
            try? await Task.sleep(for: .milliseconds(450))
            open()
        }
    }

    private func openScreen(_ store: BotStore) {
        closeSheets { store.screenRequest = ScreenRequest() }
    }

    private var notificationsRow: some View {
        Button {
            if notificationsAllowed != nil {
                UIApplication.shared.open(URL(string: UIApplication.openNotificationSettingsURLString)!)
            } else {
                Task { notificationsAllowed = await PushRegistrar.shared.requestAuthorization(from: .settings) }
            }
        } label: {
            ValueRow("Notifications", value: notificationsAllowed == true ? "On" : notificationsAllowed == false ? "Off in Settings" : "Turn on")
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }
}
