package com.codync.android

import android.graphics.Bitmap
import android.os.Build
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.asAndroidBitmap
import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.test.*
import androidx.activity.ComponentActivity
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.lifecycle.ViewModelProvider
import androidx.test.platform.app.InstrumentationRegistry
import com.codync.android.core.*
import kotlinx.serialization.json.*
import org.junit.Assert.*
import org.junit.Assume.assumeTrue
import org.junit.Rule
import org.junit.Test
import java.io.File

class ChatLayoutTest {
    @get:Rule val compose = createAndroidComposeRule<ComponentActivity>()

    @Test fun oneWayBotMessagesShowAttributionWithoutAStatusLine() {
        assumeTrue(Build.PRODUCT.startsWith("sdk"))
        val store = ViewModelProvider(compose.activity)[AppStore::class.java]
        val bot = Bot("handoff-preview", "Dex")
        val body = "Urgent. Investigate the connection error."
        val heading = "Message from Miles: $body"
        val detail = "Recipient stopped. Partial work may have happened."
        val entry = Entry("bot-message", 1, bot.id, rev = 1, kind = "notice", data = buildJsonObject {
            put("text", "$heading\n$detail"); put("heading", heading); put("delegationId", "request"); put("style", "error")
        })
        val state = AppState(loading = false, selectedBot = bot.id, bots = listOf(bot),
            connection = LinkState.Ready(HostRoute.Direct), actualConnection = LinkState.Ready(HostRoute.Direct),
            historyComplete = setOf(bot.id), entries = listOf(entry))
        compose.setContent { Hosted { ChatScreen(state, store, {}) } }
        // One line from the computer, as on iOS: the attribution, the body and the reason travel in
        // the same block, and no delivery status line follows it.
        val notice = compose.onNodeWithText(body, substring = true).assertIsDisplayed().fetchSemanticsNode()
        val text = notice.config.toString()
        assertTrue("attribution missing from the notice", text.contains("Message from Miles"))
        assertTrue("detail missing from the notice", text.contains(detail))
        compose.onNodeWithText(detail).assertDoesNotExist()
        compose.onNodeWithText("Sending").assertDoesNotExist()
        compose.onNodeWithText("Delivered").assertDoesNotExist()
        if (InstrumentationRegistry.getArguments().getString("layoutSnapshots") == "true") {
            val bitmap = compose.onRoot().captureToImage().asAndroidBitmap()
            File(compose.activity.cacheDir, "bot-message-preview.png").outputStream().use { bitmap.compress(Bitmap.CompressFormat.PNG, 100, it) }
        }
    }

    @Test fun botRepliesAppearBelowTheRequestInSeparateBubbles() {
        assumeTrue(Build.PRODUCT.startsWith("sdk"))
        val store = ViewModelProvider(compose.activity)[AppStore::class.java]
        val bot = Bot("reply-preview", "Dex")
        val body = "Investigate the connection error."
        val heading = "Request from Miles: $body"
        val answer = "The caller cancelled while audio was connecting."
        val entry = Entry("bot-reply", 1, bot.id, rev = 1, kind = "notice", data = buildJsonObject {
            put("text", "$heading\nReply from Dex:\n$answer"); put("heading", heading)
            put("delegationId", "request"); put("status", "completed")
        })
        val state = AppState(loading = false, selectedBot = bot.id, bots = listOf(bot),
            connection = LinkState.Ready(HostRoute.Direct), actualConnection = LinkState.Ready(HostRoute.Direct),
            historyComplete = setOf(bot.id), entries = listOf(entry))
        val shown = mutableStateOf(state)
        compose.setContent { Hosted { ChatScreen(shown.value, store, {}) } }
        // The request and the reply are one line from the computer (iOS `ChatRows`), above the
        // composer, with the reply named inside it.
        val notice = compose.onNodeWithText(body, substring = true).assertIsDisplayed().fetchSemanticsNode()
        val noticeBounds = notice.boundsInRoot
        assertTrue("the attribution is inside the notice", notice.config.toString().contains("Reply from Dex"))
        assertTrue("the answer is inside the notice", notice.config.toString().contains(answer))
        val composer = compose.onNodeWithContentDescription("Add files").fetchSemanticsNode().boundsInRoot
        assertTrue("the notice sits above the composer", noticeBounds.bottom <= composer.top)
        val fullReply = Entry("full-reply", 2, bot.id, rev = 2, kind = "agent", data = buildJsonObject {
            put("text", answer); put("final", true)
        })
        // The rule owns the content: a state flip re-renders instead of a second setContent.
        shown.value = state.copy(entries = listOf(entry, fullReply))
        compose.waitForIdle()
        // The finished reply arrives as its own bubble; the line's attribution is not repeated.
        compose.onAllNodesWithText("Reply from Dex", substring = true).assertCountEquals(1)
        compose.onAllNodesWithText(answer, substring = true).assertCountEquals(2)
        compose.onNode(hasText(answer) and hasClickAction()).performTouchInput { longClick(center) }
        compose.onNode(hasText("Reply") and hasClickAction()).assertIsDisplayed()
        compose.onNodeWithText("Show what it did").assertIsDisplayed()
        InstrumentationRegistry.getInstrumentation().sendKeyDownUpSync(android.view.KeyEvent.KEYCODE_BACK)
        if (InstrumentationRegistry.getArguments().getString("layoutSnapshots") == "true") {
            val bitmap = compose.onRoot().captureToImage().asAndroidBitmap()
            File(compose.activity.cacheDir, "bot-reply-preview.png").outputStream().use { bitmap.compress(Bitmap.CompressFormat.PNG, 100, it) }
        }
    }

    @Test fun aShortConversationStaysNearItsComposerAndRetainsContextActions() {
        assumeTrue(Build.PRODUCT.startsWith("sdk"))
        val store = ViewModelProvider(compose.activity)[AppStore::class.java]
        val bot = Bot("layout-preview", "Scout", avatarColor = "green", avatarShape = "cloud")
        val sent = "Add a weekly totals helper and check its tests."
        val reply = "I added the helper and a test. All four tests pass."
        val now = System.currentTimeMillis()
        val state = AppState(loading = false, selectedBot = bot.id, bots = listOf(bot),
            connection = LinkState.Ready(HostRoute.Direct), actualConnection = LinkState.Ready(HostRoute.Direct),
            historyComplete = setOf(bot.id), entries = listOf(
                Entry("user-layout", 1, bot.id, rev = 1, kind = "user", data = buildJsonObject { put("text", sent) }, createdAt = now),
                Entry("agent-layout", 2, bot.id, rev = 1, kind = "agent", data = buildJsonObject { put("text", reply); put("author", bot.id); put("final", true) }, createdAt = now)))
        compose.setContent { Hosted { ChatScreen(state, store, {}) } }
        compose.onNodeWithText("You").assertDoesNotExist()
        compose.onNodeWithText("Message actions").assertDoesNotExist()
        compose.onNodeWithText("Reply").assertDoesNotExist()
        compose.onAllNodesWithText(bot.name).assertCountEquals(1)
        val user = compose.onNode(hasText(sent) and hasClickAction())
        val agent = compose.onNode(hasText(reply) and hasClickAction())
        val userBounds = user.fetchSemanticsNode().boundsInRoot
        val agentBounds = agent.fetchSemanticsNode().boundsInRoot
        val root = compose.onRoot().fetchSemanticsNode().boundsInRoot
        assertTrue(userBounds.left > agentBounds.left)
        assertTrue(userBounds.right > agentBounds.right)
        assertTrue(userBounds.top > root.height / 2)
        compose.onNodeWithContentDescription("Add files").assertIsDisplayed()
        compose.onNodeWithContentDescription("Call").assertIsDisplayed()
        if (InstrumentationRegistry.getArguments().getString("layoutSnapshots") == "true") {
            val bitmap = compose.onRoot().captureToImage().asAndroidBitmap()
            File(compose.activity.cacheDir, "layout-chat-preview.png").outputStream().use { bitmap.compress(Bitmap.CompressFormat.PNG, 100, it) }
        }
        user.performTouchInput { longClick(center) }
        compose.onNode(hasText("Reply") and hasClickAction()).assertIsDisplayed()
        compose.onNodeWithContentDescription("👍").assertIsDisplayed()
        compose.onNodeWithText("Copy").assertIsDisplayed()
        compose.onNodeWithText("Show what it did").assertDoesNotExist()
        InstrumentationRegistry.getInstrumentation().sendKeyDownUpSync(android.view.KeyEvent.KEYCODE_BACK)
        compose.onNodeWithText("Copy").assertDoesNotExist()
    }

    @Test fun theTitleOpensTheBotsDetailsOnDetails() {
        assumeTrue(Build.PRODUCT.startsWith("sdk"))
        val store = ViewModelProvider(compose.activity)[AppStore::class.java]
        val bot = Bot("details-preview", "Scout")
        val state = AppState(loading = false, selectedBot = bot.id, bots = listOf(bot), historyComplete = setOf(bot.id),
            hello = buildJsonObject { put("version", ARTIFACTS_HOST) })
        compose.setContent { Hosted { ChatScreen(state, store, {}) } }
        compose.onNode(chatTitle).performClick()
        // Details first, with the routines; no Memory on Android until the read-only memory screen is ported.
        compose.onNode(hasText("Details") and isSelected()).assertIsDisplayed()
        compose.onNodeWithText("Library").assertIsDisplayed()
        compose.onNodeWithText("Memory").assertDoesNotExist()
        // The sheet merges plain text into itself, so the heading is found unmerged.
        compose.onNodeWithText("Routines", useUnmergedTree = true).assertExists()
        compose.onNodeWithText("Settings").performClick()
        compose.onNodeWithText("Notifications").performScrollTo().assertIsDisplayed()
    }
}
