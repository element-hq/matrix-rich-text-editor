/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.assertion.ViewAssertions.matches
import androidx.test.espresso.matcher.ViewMatchers.isDisplayed
import androidx.test.espresso.matcher.ViewMatchers.withText
import io.element.android.wysiwyg.test.rules.createFlakyEmulatorRule
import io.element.android.wysiwyg.utils.NBSP
import io.element.android.wysiwyg.view.models.InlineFormat
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test

class RichTextEditorStateTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    @get:Rule
    val flakyEmulatorRule = createFlakyEmulatorRule(retry = false)

    @Test
    fun testSharingState() = runTest {
        val state = RichTextEditorState()
        val showAlternateEditor = MutableStateFlow(false)
        composeTestRule.setContent {
            MaterialTheme {
                val showAlt by showAlternateEditor.collectAsState()
                Column {
                    if (!showAlt) {
                        Text("Main editor")
                        RichTextEditor(state = state, modifier = Modifier.fillMaxWidth())
                    } else {
                        Text("Alternative editor")
                        RichTextEditor(state = state, modifier = Modifier.fillMaxWidth())
                    }
                }
            }
        }

        state.setHtml("Hello,<br />world")
        state.setSelection(0, 5)
        composeTestRule.awaitIdle()

        composeTestRule.onNodeWithText("Main editor").assertIsDisplayed()
        onView(withText("Hello,\nworld")).check(matches(isDisplayed()))

        showAlternateEditor.emit(true)
        composeTestRule.awaitIdle()

        composeTestRule.onNodeWithText("Alternative editor").assertIsDisplayed()
        onView(withText("Hello,\nworld")).check(matches(isDisplayed()))
        assertEquals(2, state.lineCount)

        state.toggleInlineFormat(InlineFormat.Bold)
        composeTestRule.awaitIdle()

        assertEquals("<strong>Hello</strong>,<br />world", state.messageHtml)
    }

    @Test
    fun testStateRestoration() = runTest {
        val state = RichTextEditorState()
        val hideEditor = MutableStateFlow(false)
        composeTestRule.setContent {
            MaterialTheme {
                val hide by hideEditor.collectAsState()
                Column {
                    if (!hide) {
                        Text("Editor")
                        RichTextEditor(modifier = Modifier.fillMaxWidth(), state = state)
                    }
                }
            }
        }

        state.setHtml("Hello<br/>world")
        state.setSelection(4)
        composeTestRule.awaitIdle()
        // Ensure line count is set
        assertEquals(2, state.lineCount)

        // Hide and show the editor to simulate a configuration change
        hideEditor.emit(true)
        composeTestRule.awaitIdle()
        hideEditor.emit(false)
        composeTestRule.awaitIdle()

        // If the text is found, the state was restored
        onView(withText("Hello\nworld")).check(matches(isDisplayed()))
        assertEquals(state.selection, 4 to 4)
        // Line count is kept
        assertEquals(2, state.lineCount)
    }

    @Test
    fun setHtmlWithTrailingNbspKeepsIt() = runTest {
        val state = RichTextEditorState()
        composeTestRule.setContent {
            MaterialTheme {
                RichTextEditor(state = state)
            }
        }

        state.setHtml("<b>Hey</b>$NBSP")
        composeTestRule.awaitIdle()

        onView(withText("Hey ")).check(matches(isDisplayed()))
        state.setSelection(4)
    }

    @Test
    fun testStateUpdatesDisabled() = runTest {
        val state = RichTextEditorState(
            "Original text"
        )
        composeTestRule.setContent {
            MaterialTheme {
                RichTextEditor(
                    state = state,
                    registerStateUpdates = false
                )
            }
        }

        // `setHtml` waits until the actions can be processed by the UI, but that won't happen
        // with `registerStateUpdates = false`
        val job = launch { state.setHtml("Updated text") }
        composeTestRule.awaitIdle()
        job.cancel()

        onView(withText("Original text")).check(matches(isDisplayed()))
    }

    @Test
    fun testSettingMarkdownText() = runTest {
        val state = RichTextEditorState(
            "Original text"
        )
        composeTestRule.setContent {
            MaterialTheme {
                RichTextEditor(
                    state = state,
                    registerStateUpdates = true
                )
            }
        }

        state.setMarkdown("**Updated text**")
        composeTestRule.awaitIdle()

        onView(withText("Updated text")).check(matches(isDisplayed()))
    }
}
