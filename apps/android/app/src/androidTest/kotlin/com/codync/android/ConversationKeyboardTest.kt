package com.codync.android

import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.test.*
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.unit.dp
import com.codync.android.design.LinkButton
import org.junit.Rule
import org.junit.Test
import org.junit.Assert.assertEquals

class ConversationKeyboardTest {
    @get:Rule val compose = createAndroidComposeRule<MainActivity>()

    @Test fun aPlainConversationTapDismissesTheComposerButAButtonTapDoesNot() {
        var dismissals = 0
        compose.activity.setContent {
            Hosted {
                Column {
                    val draft = remember { mutableStateOf("Draft") }
                    BasicTextField(draft.value, { draft.value = it }, Modifier.testTag("composer"))
                    // The list tells the composer that a plain tap reached the conversation; that
                    // callback is the contract, and it does not depend on Compose focus semantics in a
                    // test window.
                    ConversationList("message", 1, false, PaddingValues(0.dp), onBackgroundTap = { dismissals++ }) {
                        item { Text("Conversation background") }
                        item { LinkButton("Keep editing", action = {}) }
                    }
                }
            }
        }
        compose.onNodeWithTag("composer").performClick().assertIsFocused()
        compose.onNodeWithText("Keep editing").performTouchInput { click() }
        assertEquals(0, dismissals)
        compose.onNodeWithTag("composer").assertIsFocused()
        compose.onNodeWithText("Conversation background").performTouchInput { click() }
        assertEquals(1, dismissals)
        compose.onNodeWithTag("composer").assertTextEquals("Draft")
    }
}
