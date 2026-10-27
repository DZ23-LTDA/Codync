import CodyncKit
import SwiftUI

/// What the bot's agent already brings, and the Codync skills or connectors passed on top of that.
struct BotGearPage: View {
    @Binding var draft: BotDraft
    var kind: Kind
    var saveError: String?
    @Environment(BotStore.self) private var model
    @State private var viewingSkill: SkillRef?

    enum Kind { case skills, connectors, rules, plugins

        var passesOn: Bool {
            switch self {
            case .skills, .connectors: true
            case .rules, .plugins: false
            }
        }
    }

    private var bot: Bot? { draft.id.flatMap { model.bots[$0] } }

    /// A personal workspace keeps its folder on the saved bot; a draft only has a folder once one is chosen.
    private var cwd: String { draft.cwd.isEmpty ? (bot?.cwd ?? "") : draft.cwd }

    private var agent: String { model.backendName(draft.backend) }

    private var agentItems: [AgentGearItem] {
        switch kind {
        case .skills: model.agentGear?.skills ?? []
        case .connectors: model.agentGear?.connectors ?? []
        case .rules: model.agentGear?.rules ?? []
        case .plugins: model.agentGear?.plugins ?? []
        }
    }

    private var intro: String {
        let key = kind.passesOn
            ? "{agent} brings these. Codync adds its own on top, and those are what get passed to the agent."
            : "{agent} brings these."
        return L10n.tr(key).replacingOccurrences(of: "{agent}", with: agent)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 28) {
            Text(intro)
                .font(.footnote)
                .foregroundStyle(Palette.secondary)
            agentSection
            if kind.passesOn {
                codyncSection.disabled(model.isOffline)
            }
            if let saveError {
                Text(saveError).font(.footnote).foregroundStyle(Palette.danger)
            }
        }
        .task(id: "\(draft.backend)|\(cwd)|\(model.connection == .online)") { await load() }
        .codyncSheet(item: $viewingSkill) { SkillSheet(skill: $0) }
    }

    private var agentSection: some View {
        CardSection(L10n.tr("From {agent}").replacingOccurrences(of: "{agent}", with: agent)) {
            if model.agentGear == nil, !model.agentGearFailed {
                Text(L10n.tr("Loading…")).foregroundStyle(Palette.secondary)
            } else if model.agentGearFailed {
                Text(L10n.tr("Couldn't load what this agent brings.")).foregroundStyle(Palette.danger)
            } else if agentItems.isEmpty {
                Text(L10n.tr("None from this agent.")).foregroundStyle(Palette.secondary)
            } else {
                ForEach(agentItems) { item in
                    VStack(alignment: .leading, spacing: 2) {
                        Text(item.name).foregroundStyle(Palette.text)
                        let note = kind == .connectors ? item.detail : item.description
                        if !note.isEmpty {
                            Text(note).font(.caption).foregroundStyle(Palette.secondary)
                        }
                    }
                }
            }
        }
    }

    @ViewBuilder private var codyncSection: some View {
        switch kind {
        case .skills:
            PluginToggles(
                title: "From Codync",
                empty: "No skills yet. Get some from Extensions, or write your own.",
                items: model.installedSkills.map { ($0.id, $0.name, $0.description) },
                selection: Binding(get: { draft.skills ?? [] }, set: { draft.skills = $0 }),
                info: { id in viewingSkill = model.installedSkills.first { $0.id == id }.map(SkillRef.init) }
            )
        case .connectors:
            PluginToggles(
                title: "From Codync",
                empty: "No connectors yet. Add GitHub, Linear, Notion and more from Extensions.",
                items: model.installedConnectors.map { ($0.id, $0.name, $0.command ?? $0.url ?? "") },
                selection: Binding(get: { draft.connectors ?? [] }, set: { draft.connectors = $0 })
            )
        case .rules, .plugins:
            EmptyView()
        }
    }

    private func load() async {
        await model.refreshPlugins()
        await model.loadAgentGear(backend: draft.backend, cwd: cwd)
    }
}
