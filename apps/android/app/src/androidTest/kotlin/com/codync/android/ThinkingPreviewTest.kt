package com.codync.android

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.Text
import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.test.*
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.unit.dp
import com.codync.android.core.Bot
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Rule
import org.junit.Test

class ThinkingPreviewTest {
    @get:Rule val compose = createComposeRule()

    @Test fun thinkingExpansionAndStreamingFollowTheBottomButLeaveHistoryInPlace() {
        // The live orb runs a frame loop; advance motion explicitly instead of waiting for it to stop.
        compose.mainClock.autoAdvance = false
        val thought = mutableStateOf("First thinking line")
        lateinit var list: LazyListState
        compose.setContent {
            Hosted {
                list = rememberLazyListState()
                rememberFollowing(list, "message", 1, working = true, bottom = 0.dp)
                LazyColumn(Modifier.size(320.dp).testTag("conversation"), state = list, contentPadding = PaddingValues(0.dp)) {
                    items(30) { Box(Modifier.height(48.dp)) { Text("Message $it") } }
                    item("working") { WorkingIndicator(Bot("bot", "Bot", status = "working"), thought.value) }
                }
            }
        }
        compose.mainClock.advanceTimeBy(1000)
        compose.waitForIdle()
        compose.onNodeWithText("Working…").performClick()
        compose.mainClock.advanceTimeBy(1000)
        compose.onNodeWithText("First thinking line").assertIsDisplayed()
        compose.runOnIdle { assertFalse(list.canScrollForward) }
        compose.runOnIdle { thought.value = (1..12).joinToString("\n") { "Thinking line $it" } }
        compose.mainClock.advanceTimeBy(1000)
        compose.waitForIdle()
        compose.runOnIdle { assertFalse(list.canScrollForward) }
        compose.onNodeWithTag("conversation").performTouchInput { swipeDown() }
        compose.mainClock.advanceTimeBy(1000)
        compose.waitForIdle()
        var place = 0 to 0
        compose.runOnIdle {
            place = list.firstVisibleItemIndex to list.firstVisibleItemScrollOffset
            thought.value += "\nAnother thinking line"
        }
        compose.mainClock.advanceTimeBy(1000)
        compose.waitForIdle()
        compose.runOnIdle { assertEquals(place, list.firstVisibleItemIndex to list.firstVisibleItemScrollOffset) }
    }
}
