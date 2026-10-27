package com.codync.android

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.material3.Text
import androidx.compose.runtime.setValue
import androidx.compose.ui.unit.dp
import com.codync.android.core.Bot
import com.codync.android.core.BotDraft
import com.codync.android.core.LinkState
import com.codync.android.design.*
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put

/** One skill or connector the agent keeps for itself (`agentGear`). */
internal data class GearItem(val name: String, val description: String, val detail: String)

internal data class AgentGear(val skills: List<GearItem>, val connectors: List<GearItem>)

/** What `backend` already has in `cwd`. Codync's own skills and connectors are not in this list. */
internal suspend fun AppStore.agentGear(backend: String, cwd: String): AgentGear {
    val body = hostCall("agentGear", buildJsonObject {
        put("backend", backend)
        put("cwd", cwd)
    })
    val obj = body as? JsonObject ?: return AgentGear(emptyList(), emptyList())
    return AgentGear(obj.gearItems("skills"), obj.gearItems("connectors"))
}

private fun JsonObject.gearItems(key: String): List<GearItem> {
    val rows = this[key] as? JsonArray ?: return emptyList()
    return rows.mapNotNull { row ->
        val item = row as? JsonObject ?: return@mapNotNull null
        val name = item["name"]?.jsonPrimitive?.contentOrNull?.trim().orEmpty()
        if (name.isEmpty()) return@mapNotNull null
        GearItem(name, item["description"]?.jsonPrimitive?.contentOrNull.orEmpty(), item["detail"]?.jsonPrimitive?.contentOrNull.orEmpty())
    }
}

/**
 * The Skills or Connectors page: what the bot's agent already brings, then the Codync ones passed on top.
 * `skills` picks the skills page; otherwise the page is connectors.
 */
@Composable internal fun BotGearPage(bot: Bot, state: AppState, store: AppStore, draft: BotDraft, change: (BotDraft) -> Unit, skills: Boolean) {
    val online = state.actualConnection is LinkState.Ready
    val cwd = draft.cwd.ifEmpty { bot.cwd }
    val agent = backendName(state, draft.backend)
    var gear by remember(bot.id) { mutableStateOf<AgentGear?>(null) }
    var failed by remember(bot.id) { mutableStateOf(false) }
    var viewingSkill by remember { mutableStateOf<SkillRef?>(null) }
    LaunchedEffect(draft.backend, cwd, online) {
        gear = null
        failed = false
        if (!online) {
            failed = true
            return@LaunchedEffect
        }
        try {
            gear = store.agentGear(draft.backend, cwd)
        } catch (e: Exception) {
            if (e is kotlinx.coroutines.CancellationException) throw e
            failed = true
        }
    }
    LaunchedEffect(online) { if (online) store.refreshPlugins() }
    val items = if (skills) gear?.skills.orEmpty() else gear?.connectors.orEmpty()
    Column(verticalArrangement = Arrangement.spacedBy(28.dp)) {
        Text(tr("{agent} brings these. Codync adds its own on top, and those are what get passed to the agent.").replace("{agent}", agent),
            style = Type.footnote, color = Palette.secondary)
        CardSection(tr("From {agent}").replace("{agent}", agent)) {
            when {
                gear == null && !failed -> row { Text(tr("Loading…"), style = Type.body, color = Palette.secondary) }
                failed -> row { Text(tr("Couldn't load what this agent brings."), style = Type.body, color = Palette.danger) }
                items.isEmpty() -> row { Text(tr("None from this agent."), style = Type.body, color = Palette.secondary) }
                else -> for (item in items) row {
                    val note = if (skills) item.description else item.detail
                    Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                        Text(item.name, style = Type.body, color = Palette.text)
                        if (note.isNotEmpty()) Text(note, style = Type.caption, color = Palette.secondary)
                    }
                }
            }
        }
        val plugins = state.plugins
        if (skills) {
            PluginToggles("From Codync", "No skills yet. Get some from Extensions, or write your own.",
                plugins?.skills.orEmpty().map { Triple(it.id, it.name, it.description) }, draft.skills.orEmpty(),
                { change(draft.copy(skills = it)) }) { id ->
                plugins?.skills?.firstOrNull { it.id == id }?.let { viewingSkill = SkillRef(it.id, it.name, it.description) }
            }
        } else {
            PluginToggles("From Codync", "No connectors yet. Add GitHub, Linear, Notion and more from Extensions.",
                plugins?.connectors.orEmpty().map { Triple(it.id, it.name, it.command ?: it.url.orEmpty()) },
                draft.connectors ?: plugins?.connectors.orEmpty().map { it.id }, { change(draft.copy(connectors = it)) })
        }
    }
    CodyncSheet(viewingSkill, { viewingSkill = null }) { SkillSheet(it, state, store) }
}
