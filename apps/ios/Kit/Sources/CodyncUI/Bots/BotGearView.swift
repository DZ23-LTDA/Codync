import CodyncKit
import SwiftUI

/// What the bot's agent already brings, and the Codync skills or connectors passed on top of that.
struct BotGearPage: View {
    @Binding var draft: BotDraft
    var kind: Kind
    @Environment(BotStore.self) private var model
    @State private var gear: AgentGear?
    @State private var failed = false
    @State private var viewingSkill: SkillRef?

    enum Kind { case skills, connectors }

    private var bot: Bot? { draft.id.flatMap { model.bots[$0] } }

    /// A personal workspace keeps its folder on the saved bot; a draft only has a folder once one is chosen.
    private var cwd: String { draft.cwd.isEmpty ? (bot?.cwd ?? "") : draft.cwd }

    private var agent: String { model.backendName(draft.backend) }

    private var agentItems: [AgentGearItem] {
        switch kind {
        case .skills: gear?.skills ?? []
        case .connectors: gear?.connectors ?? []
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 28) {
            Text(L10n.tr("{agent} brings these. Codync adds its own on top, and those are what get passed to the agent.")
                .replacingOccurrences(of: "{agent}", with: agent))
                .font(.footnote)
                .foregroundStyle(Palette.secondary)
            agentSection
            codyncSection
        }
        .task(id: "\(draft.backend)|\(cwd)") { await load() }
        .codyncSheet(item: $viewingSkill) { SkillSheet(skill: $0) }
    }

    private var agentSection: some View {
        CardSection(L10n.tr("From {agent}").replacingOccurrences(of: "{agent}", with: agent)) {
            if gear == nil, !failed {
                Text(L10n.tr("Loading…")).foregroundStyle(Palette.secondary)
            } else if failed {
                Text(L10n.tr("Couldn't load what this agent brings.")).foregroundStyle(Palette.danger)
            } else if agentItems.isEmpty {
                Text(L10n.tr("None from this agent.")).foregroundStyle(Palette.secondary)
            } else {
                ForEach(agentItems) { item in
                    VStack(alignment: .leading, spacing: 2) {
                        Text(item.name).foregroundStyle(Palette.text)
                        let note = kind == .skills ? item.description : item.detail
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
        }
    }

    private func load() async {
        await model.refreshPlugins()
        failed = false
        guard let client = model.client else {
            failed = true
            return
        }
        do {
            let loaded = try await client.agentGear(backend: draft.backend, cwd: cwd)
            if !Task.isCancelled { gear = loaded }
        } catch {
            if !Task.isCancelled { failed = true }
        }
    }
}
