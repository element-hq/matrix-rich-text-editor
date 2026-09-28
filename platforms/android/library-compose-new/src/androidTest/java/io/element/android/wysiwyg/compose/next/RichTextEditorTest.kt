/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next

import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.click
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performKeyInput
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.performTextInputSelection
import androidx.compose.ui.test.performSemanticsAction
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.pressKey
import androidx.compose.ui.text.TextLayoutResult
import androidx.compose.ui.text.TextRange
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

private const val TAG = "editor"
private const val AT_ROOM = "<a data-mention-type=\"at-room\" href=\"#\" contenteditable=\"false\">@room</a>"

/**
 * Runs the editor on a device, with the Android build of the Rust library.
 */
@OptIn(ExperimentalTestApi::class)
@RunWith(AndroidJUnit4::class)
class RichTextEditorTest {
    @get:Rule
    val rule = createComposeRule()

    private lateinit var state: RichTextEditorState

    private fun setUp(html: String = "") {
        rule.setContent {
            state = rememberRichTextEditorState(initialHtml = html).apply {
                onError = { throw AssertionError("Rust composer error", it) }
            }
            RichTextEditor(state = state, modifier = Modifier.testTag(TAG))
        }
        rule.onNodeWithTag(TAG).performClick()
        rule.waitForIdle()
    }

    private val editor get() = rule.onNodeWithTag(TAG)

    private fun select(start: Int, end: Int = start) {
        editor.performTextInputSelection(TextRange(start, end))
        rule.waitForIdle()
    }

    private fun assertHtml(expected: String) {
        rule.waitForIdle()
        assertEquals(expected, state.internalHtml.replace(' ', ' '))
    }

    @Test
    fun typing_and_enter() {
        setUp()
        editor.performTextInput("hello")
        editor.performKeyInput { pressKey(Key.Enter) }
        editor.performTextInput("world")
        assertHtml("<p>hello</p><p>world</p>")
        assertEquals(TextRange(11), state.selection)
    }

    @Test
    fun typing_over_a_selection_across_paragraphs() {
        setUp("<p>abc</p><p>def</p>")
        select(1, 5)
        editor.performTextInput("x")
        assertHtml("<p>axef</p>")
        assertEquals(TextRange(2), state.selection)
    }

    @Test
    fun typing_over_a_selection_across_list_items() {
        // Rust used to lose the typed text
        setUp("<ul><li>one</li><li>two</li></ul><p>after</p>")
        select(0, 4)
        editor.performTextInput("x")
        assertHtml("<ul><li>xtwo</li></ul><p>after</p>")
    }

    @Test
    fun typing_over_a_selection_from_a_list_into_a_paragraph() {
        // Rust used to panic
        setUp("<ul><li>one</li><li>two</li></ul><p>after</p>")
        select(0, 9)
        editor.performTextInput("x")
        assertHtml("<p>xfter</p>")
    }

    @Test
    fun enter_over_a_selection_across_list_items() {
        setUp("<ul><li>one</li><li>two</li></ul>")
        select(1, 6)
        editor.performKeyInput { pressKey(Key.Enter) }
        assertHtml("<ul><li>o</li><li>o</li></ul>")
        assertEquals(TextRange(2), state.selection)
    }

    @Test
    fun enter_at_the_start_of_a_list_item_keeps_the_cursor_with_its_text() {
        setUp("<ol><li>abcd</li><li>ef</li></ol>")
        select(5)
        editor.performKeyInput { pressKey(Key.Enter) }
        assertHtml("<ol><li>abcd</li><li></li><li>ef</li></ol>")
        assertEquals(TextRange(6), state.selection)
        editor.performTextInput("x")
        assertHtml("<ol><li>abcd</li><li></li><li>xef</li></ol>")
    }

    @Test
    fun enter_in_an_empty_list_item_followed_by_other_items_splits_the_list() {
        setUp("<ul><li>a</li><li>b</li></ul>")
        select(1)
        editor.performKeyInput { pressKey(Key.Enter) }
        editor.performKeyInput { pressKey(Key.Enter) }
        assertHtml("<ul><li>a</li></ul><p> </p><ul><li>b</li></ul>")
        editor.performTextInput("x")
        assertHtml("<ul><li>a</li></ul><p>x</p><ul><li>b</li></ul>")
    }

    @Test
    fun deleting_a_selection_across_an_empty_list_item() {
        setUp("<ul><li>ab</li><li></li><li>cd</li></ul>")
        select(1, 5)
        editor.performKeyInput { pressKey(Key.Backspace) }
        assertHtml("<ul><li>ad</li></ul>")
    }

    @Test
    fun backspace_at_the_start_of_a_list_removes_the_first_item() {
        setUp("<ul><li>one</li><li>two</li></ul>")
        select(0)
        editor.performKeyInput { pressKey(Key.Backspace) }
        assertHtml("<p>one</p><ul><li>two</li></ul>")
    }

    @Test
    fun at_room_mentions_survive_setting_the_html() {
        setUp("hi $AT_ROOM ")
        assertHtml("hi $AT_ROOM ")
        assertEquals(1, state.document.mentions.size)
        assertTrue(state.document.mentions.single().isAtRoom)
    }

    @Test
    fun tapping_at_the_start_and_end_of_each_line_places_the_cursor_there() {
        setUp(
            "<p>abc</p><p>def</p><ul><li>one</li><li></li><li>two</li></ul>" +
                "<blockquote><p>quote</p></blockquote><pre><code>code</code></pre><p>end</p>"
        )
        val width = editor.fetchSemanticsNode().size.width.toFloat()
        val results = mutableListOf<TextLayoutResult>()
        editor.performSemanticsAction(SemanticsActions.GetTextLayoutResult) { it(results) }
        val layout = results.single()
        for (line in state.document.lines) {
            // There are no mentions, so logical and visual offsets are the same
            val visualLine = layout.getLineForOffset(line.start)
            val y = (layout.getLineTop(visualLine) + layout.getLineBottom(visualLine)) / 2
            for ((x, expected) in listOf(width - 2f to line.end, 1f to line.start)) {
                editor.performTouchInput { click(Offset(x, y)) }
                // Avoid detecting the next click as a double tap
                rule.mainClock.advanceTimeBy(1_000)
                rule.waitForIdle()
                assertEquals(
                    "Tapping at x=$x in line '${state.textFieldState.text.substring(line.start, line.end)}'",
                    TextRange(expected),
                    state.selection,
                )
            }
        }
    }

    @Test
    fun the_cursor_in_an_empty_line_is_indented() {
        // HTML, index of the empty line, index of a line with text and the same indentation
        val cases = listOf(
            Triple("<ul><li>a</li><li></li></ul>", 1, 0),
            Triple("<ul><li>a</li><li></li><li>b</li></ul>", 1, 0),
            Triple("<blockquote><p>a</p><p>\u00A0</p></blockquote>", 1, 0),
            Triple("<pre><code>a\n\u00A0</code></pre>", 1, 0),
            Triple("<ul><li><p>a</p><ul><li>b</li><li></li></ul></li></ul>", 2, 1),
        )
        setUp()
        for ((html, emptyLine, referenceLine) in cases) {
            rule.runOnIdle { state.setHtml(html) }
            rule.waitForIdle()
            val results = mutableListOf<TextLayoutResult>()
            editor.performSemanticsAction(SemanticsActions.GetTextLayoutResult) { it(results) }
            val layout = results.single()
            val lines = state.document.lines
            // There are no mentions, so logical and visual offsets are the same
            val emptyX = layout.getCursorRect(lines[emptyLine].start).left
            val referenceX = layout.getCursorRect(lines[referenceLine].start).left
            assertTrue("Line with text in $html should be indented", referenceX > 0f)
            assertEquals("Cursor in the empty line of $html", referenceX, emptyX, 1f)
        }
    }

    @Test
    fun mention_at_suggestion_and_deleting_it() {
        setUp()
        editor.performTextInput("hi @ali")
        state.insertMentionAtSuggestion(text = "Alice", url = "https://matrix.to/#/@alice:matrix.org")
        rule.waitForIdle()
        assertEquals("hi ￼ ", state.textFieldState.text.toString())
        editor.performKeyInput { pressKey(Key.Backspace) }
        editor.performKeyInput { pressKey(Key.Backspace) }
        assertHtml("hi ")
    }
}
