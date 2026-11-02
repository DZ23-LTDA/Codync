package com.codync.android

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.*
import androidx.compose.ui.test.*
import androidx.compose.ui.test.junit4.createComposeRule
import com.codync.android.core.*
import kotlinx.coroutines.*
import org.junit.Assert.*
import org.junit.Rule
import org.junit.Test
import java.util.concurrent.atomic.AtomicBoolean

class AgentSettingsScreenTest {
    @get:Rule val compose = createComposeRule()
    private val online = AppState(loading = false, computers = listOf(Computer("fixture", "Fixture computer", "fixture")),
        actualConnection = LinkState.Ready(HostRoute.Direct))
    private fun model(name: String) = hasText(name, substring = true) and hasClickAction()

    @Composable private fun Picker(state: AppState, backend: String, selected: String?, choose: (String?) -> Unit, load: suspend (String) -> AgentModels) =
        AgentModelPicker(state, load, backend, selected, emptyMap(), false, null, { id, _ -> choose(id) }, {})

    @Test fun hostModelsRetainAnUnknownOverrideAndFoldTheRecommendedDefault() {
        var selected by mutableStateOf<String?>("old-model")
        compose.setContent { Hosted { Picker(online, "agent", selected, { selected = it }) {
            AgentModels(listOf(AgentModel("auto", "Default (recommended)"), AgentModel("host-model", "Host model", "From this agent")))
        } } }
        compose.waitUntil(5_000) { compose.onAllNodes(model("old-model")).fetchSemanticsNodes().size == 1 }
        compose.onNode(model("old-model")).performClick()
        compose.onAllNodesWithText("Default (recommended)").assertCountEquals(1)
        compose.onNodeWithText("Default (recommended)").performClick()
        assertNull(selected)
        compose.waitForIdle()
        // The picker row and the menu item share the word: the row is the one further down.
        val reopened = compose.onAllNodes(hasText("Default", substring = true) and hasClickAction())
        val nodes = reopened.fetchSemanticsNodes()
        reopened[nodes.indices.maxBy { nodes[it].boundsInRoot.top }].performClick()
        compose.onNodeWithText("Host model").performClick()
        assertEquals("host-model", selected)
        // The chosen model is what the row shows now (the refresh button lives inside the open list).
        compose.waitForIdle()
        // The list stays open after choosing (its options belong to the model): the name is in
        // the row and in the open list.
        compose.onAllNodes(model("Host model")).assertCountEquals(2)
    }

    @Test fun aLateCancelledModelDiscoveryCannotReplaceAnotherAgentsCatalog() {
        var backend by mutableStateOf("old")
        val started = AtomicBoolean(false); val returned = AtomicBoolean(false)
        compose.setContent { Hosted { Column {
            TextButton(onClick = { backend = "new" }) { Text("Switch agent") }
            Picker(online, backend, null, {}) { id ->
                if (id == "old") withContext(NonCancellable) { started.set(true); delay(600); returned.set(true) }
                AgentModels(listOf(AgentModel(id, "$id advertised model")))
            }
        } } }
        compose.waitUntil(5_000) { started.get() }
        compose.onNodeWithText("Switch agent").performClick()
        compose.waitUntil(5_000) { compose.onAllNodes(model("new")).fetchSemanticsNodes().size == 1 }
        compose.waitUntil(5_000) { returned.get() }
        compose.onNode(model("new")).performClick()
        compose.onNodeWithText("new advertised model").assertIsDisplayed()
        compose.onNodeWithText("old advertised model").assertDoesNotExist()
    }

    @Test fun anOfflineModelOverrideRemainsEditableWithoutInventingAProviderList() {
        var selected by mutableStateOf<String?>("my-model")
        compose.setContent { Hosted { Picker(online.copy(actualConnection = LinkState.Connecting), "agent", selected, { selected = it }) {
            fail("Offline must not query models"); AgentModels(emptyList())
        } } }
        // The reason lives inside the list it is about: opening the picker is what shows it.
        compose.onNode(model("my-model")).performClick()
        compose.onNodeWithText("Connect to this computer to load its models. You can still enter a model ID.").assertIsDisplayed()
        compose.onNodeWithContentDescription("Refresh model list").assertIsDisplayed()
        compose.onNode(hasText("agent · Default", substring = true)).assertDoesNotExist()
        compose.onNode(hasSetTextAction() and hasText("my-model")).performTextReplacement("saved-model")
        assertEquals("saved-model", selected)
    }

    @Test fun hostFolderBrowsingFiltersLocallyDrillsInAndSelectsTheAuthoritativeDirectory() {
        var chosen: String? = null
        val requests = mutableListOf<String?>()
        compose.setContent { Hosted { FolderPicker("/home", true, "Fixture computer", load = { path ->
            requests.add(path)
            if (path == "/home/Repo") DirListing("/home/Repo", "/home", true, emptyList(), recent = emptyList())
            else DirListing("/home", "/", false, listOf(DirItem("Repo", "/home/Repo", true), DirItem("Other", "/home/Other")), recent = emptyList())
        }, select = { it }) { chosen = it } } }
        compose.waitUntil(5_000) { compose.onAllNodesWithText("Browse other folders").fetchSemanticsNodes().size == 1 }
        compose.onNodeWithText("Search recent workspaces").assertDoesNotExist()
        compose.onNodeWithText("Browse other folders").performClick()
        compose.waitUntil(5_000) { compose.onAllNodesWithText("Repo").fetchSemanticsNodes().size == 1 }
        compose.onNode(hasSetTextAction()).performTextInput("repo")
        compose.onNodeWithText("Other").assertDoesNotExist()
        assertEquals(listOf(null, "/home"), requests)
        compose.onNodeWithText("Repo").performClick()
        compose.waitUntil(5_000) { requests.lastOrNull() == "/home/Repo" }
        compose.onNodeWithText("Use this folder").performClick()
        compose.waitUntil(5_000) { chosen == "/home/Repo" }
        assertEquals(listOf(null, "/home", "/home/Repo"), requests)
        compose.onNodeWithContentDescription("Parent folder").performClick()
        compose.waitUntil(5_000) { compose.onAllNodesWithText("Other").fetchSemanticsNodes().size == 1 }
        compose.onNodeWithText("Use this folder").performClick()
        compose.waitUntil(5_000) { chosen == "/home" }
    }

    @Test fun anUnavailableOrOfflineFolderShowsWhyWithoutOfferingIt() {
        val online = mutableStateOf(true)
        var loads = 0
        compose.setContent { Hosted { FolderPicker("/missing", online.value, "Fixture computer", load = { loads++; throw java.io.IOException("This folder no longer exists.") }, select = { fail("Unavailable folder must not be selected"); it }) {
            fail("A folder that failed to load can't be chosen")
        } } }
        compose.waitUntil(5_000) { compose.onAllNodesWithText("This folder no longer exists.").fetchSemanticsNodes().size == 1 }
        compose.onNodeWithText("Use this folder").assertIsNotEnabled()
        compose.runOnUiThread { online.value = false }
        compose.onNodeWithText("Connect to this computer to browse its folders.").assertIsDisplayed()
        assertEquals(1, loads)
    }
}
