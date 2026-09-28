/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next

import androidx.compose.ui.text.TextRange
import io.element.android.wysiwyg.view.models.InlineFormat
import io.element.android.wysiwyg.view.models.LinkAction
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction
import uniffi.wysiwyg_composer.MenuAction

private const val PARAGRAPHS = "<p>abc</p><p>def</p>"
private const val LIST = "<ul><li>one</li><li>two</li></ul><p>after</p>"
private const val QUOTE = "<p>a</p><blockquote><p>bc</p></blockquote><p>d</p>"
private const val CODE = "<p>a</p><pre><code>bc\nd</code></pre><p>e</p>"
private const val MENTION = "a <a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\" " +
    "contenteditable=\"false\">Alice</a> b"

/**
 * Explicit expectations for editing selections and deleting at the boundaries of blocks. See
 * [SelectionEditingDifferentialTest] for exhaustive checks of every selection.
 */
class SelectionEditingTest : RustTest() {
    private fun EditorTester.assertContent(html: String, text: String, selection: TextRange) {
        // Rust uses non-breaking spaces to keep consecutive spaces in its HTML
        assertEquals(html, state.composerHtml()?.replace('\u00A0', ' '))
        assertEquals(text, this.text)
        assertEquals(selection, this.selection)
    }

    // region Paragraphs

    @Test
    fun `typing over a selection spanning two paragraphs merges them`() = editorTest(PARAGRAPHS) {
        select(1, 5)
        commit("x")
        assertContent("<p>axef</p>", "axef", TextRange(2))
    }

    @Test
    fun `deleting the line break between paragraphs merges them`() = editorTest(PARAGRAPHS) {
        select(3, 4)
        backspace()
        assertContent("<p>abcdef</p>", "abcdef", TextRange(3))
    }

    @Test
    fun `backspace at the start of a paragraph merges it with the previous one`() = editorTest(PARAGRAPHS) {
        select(4)
        backspace()
        assertContent("<p>abcdef</p>", "abcdef", TextRange(3))
    }

    @Test
    fun `delete at the end of a paragraph merges it with the next one`() = editorTest(PARAGRAPHS) {
        select(3)
        forwardDelete()
        assertContent("<p>abcdef</p>", "abcdef", TextRange(3))
    }

    @Test
    fun `deleting everything`() = editorTest(PARAGRAPHS) {
        select(0, 7)
        backspace()
        assertEquals("", text)
        assertEquals(TextRange(0), selection)
    }

    @Test
    fun `typing over everything`() = editorTest(PARAGRAPHS) {
        select(0, 7)
        commit("x")
        assertContent("<p>x</p>", "x", TextRange(1))
    }

    @Test
    fun `typing over a reversed selection`() = editorTest(PARAGRAPHS) {
        select(5, 1)
        commit("x")
        assertContent("<p>axef</p>", "axef", TextRange(2))
    }

    @Test
    fun `enter over a selection spanning two paragraphs`() = editorTest(PARAGRAPHS) {
        select(1, 5)
        enter()
        assertContent("<p>a</p><p>ef</p>", "a\nef", TextRange(2))
    }

    @Test
    fun `pasting several lines over a selection spanning two paragraphs`() = editorTest(PARAGRAPHS) {
        select(1, 5)
        commit("x\ny")
        assertContent("<p>ax</p><p>yef</p>", "ax\nyef", TextRange(4))
    }

    @Test
    fun `IME deleting a word across a paragraph break`() = editorTest(PARAGRAPHS) {
        select(5)
        backspace(count = 3)
        assertContent("<p>abef</p>", "abef", TextRange(2))
    }

    @Test
    fun `IME replacing its composing text at the start of a paragraph`() = editorTest(PARAGRAPHS) {
        select(7)
        replace(4, 7, "deaf")
        assertContent("<p>abc</p><p>deaf</p>", "abc\ndeaf", TextRange(8))
    }

    // endregion

    // region Lists

    @Test
    fun `backspace at the start of a list item merges it with the previous one`() = editorTest(LIST) {
        select(4)
        backspace()
        assertContent("<ul><li>onetwo</li></ul><p>after</p>", "onetwo\nafter", TextRange(3))
    }

    @Test
    fun `backspace at the start of a paragraph after a list moves it into the last item`() = editorTest(LIST) {
        select(8)
        backspace()
        assertContent("<ul><li>one</li><li>twoafter</li></ul>", "one\ntwoafter", TextRange(7))
    }

    @Test
    fun `delete at the end of the last list item`() = editorTest(LIST) {
        select(7)
        forwardDelete()
        assertContent("<ul><li>one</li><li>twoafter</li></ul>", "one\ntwoafter", TextRange(7))
    }

    @Test
    fun `typing over a selection spanning two list items`() = editorTest(LIST) {
        select(2, 5)
        commit("x")
        assertContent("<ul><li>onxwo</li></ul><p>after</p>", "onxwo\nafter", TextRange(3))
    }

    @Test
    fun `typing over a selection from a list item into a paragraph`() = editorTest(LIST) {
        select(5, 10)
        commit("x")
        assertContent("<ul><li>one</li><li>txter</li></ul>", "one\ntxter", TextRange(6))
    }

    @Test
    fun `typing over a selection from the start of a list into a paragraph`() = editorTest(LIST) {
        // Rust used to panic replacing this selection
        select(0, 9)
        commit("x")
        assertContent("<p>xfter</p>", "xfter", TextRange(1))
    }

    @Test
    fun `deleting a whole list`() = editorTest(LIST) {
        select(0, 8)
        backspace()
        assertContent("<p>after</p>", "after", TextRange(0))
    }

    @Test
    fun `deleting the text of a list item keeps the item`() = editorTest(LIST) {
        select(4, 7)
        backspace()
        assertContent("<ul><li>one</li><li></li></ul><p>after</p>", "one\n\nafter", TextRange(4))
    }

    @Test
    fun `deleting a list item with its line break removes the item`() = editorTest(LIST) {
        select(3, 7)
        backspace()
        assertContent("<ul><li>one</li></ul><p>after</p>", "one\nafter", TextRange(3))
    }

    @Test
    fun `backspace at the start of the content removes the first list item`() = editorTest(LIST) {
        select(0)
        assertTrue(state.onBackspaceAtStart())
        assertEquals("<p>one</p><ul><li>two</li></ul><p>after</p>", state.composerHtml())
        assertEquals("one\ntwo\nafter", text)
        assertEquals(null, state.document.lines.first().marker)
        assertConsistent()
    }

    @Test
    fun `backspace at the start of an empty list removes it`() = editorTest {
        state.toggleList(ordered = true)
        assertTrue(state.onBackspaceAtStart())
        assertEquals("", state.composerHtml())
        assertEquals(null, state.document.lines.single().marker)
        assertConsistent()
    }

    @Test
    fun `backspace not at the start of the content is handled by the text field`() = editorTest(LIST) {
        select(1)
        assertFalse(state.onBackspaceAtStart())
        select(0, 2)
        assertFalse(state.onBackspaceAtStart())
    }

    // endregion

    // region Quotes and code blocks

    @Test
    fun `backspace at the start of a quote merges it with the previous paragraph`() = editorTest(QUOTE) {
        select(2)
        backspace()
        assertContent("<p>abc</p><p>d</p>", "abc\nd", TextRange(1))
    }

    @Test
    fun `delete at the end of a quote moves the next paragraph into it`() = editorTest(QUOTE) {
        select(4)
        forwardDelete()
        assertContent("<p>a</p><blockquote><p>bcd</p></blockquote>", "a\nbcd", TextRange(4))
    }

    @Test
    fun `typing over a selection into a quote`() = editorTest(QUOTE) {
        select(0, 3)
        commit("x")
        assertContent("<blockquote><p>xc</p></blockquote><p>d</p>", "xc\nd", TextRange(1))
    }

    @Test
    fun `typing over a selection out of a quote`() = editorTest(QUOTE) {
        select(3, 6)
        commit("x")
        assertContent("<p>a</p><blockquote><p>bx</p></blockquote>", "a\nbx", TextRange(4))
    }

    @Test
    fun `backspace at the start of a code block merges its first line with the previous paragraph`() = editorTest(CODE) {
        select(2)
        backspace()
        assertContent("<p>abc</p><pre><code>d</code></pre><p>e</p>", "abc\nd\ne", TextRange(1))
    }

    @Test
    fun `backspace at the start of a code block line joins the lines`() = editorTest(CODE) {
        select(5)
        backspace()
        assertContent("<p>a</p><pre><code>bcd</code></pre><p>e</p>", "a\nbcd\ne", TextRange(4))
    }

    @Test
    fun `typing over a selection across code block lines`() = editorTest(CODE) {
        select(3, 6)
        commit("x")
        assertContent("<p>a</p><pre><code>bx</code></pre><p>e</p>", "a\nbx\ne", TextRange(4))
    }

    // endregion

    // region Formatting and mentions

    @Test
    fun `typing over a selection uses the formatting at its start`() = editorTest("a<strong>bc</strong>d") {
        // The typed character is the same as the first selected one, but it must still replace
        // the whole selection
        select(0, 3)
        commit("a")
        assertContent("ad", "ad", TextRange(1))
    }

    @Test
    fun `typing over a formatted selection keeps the formatting`() = editorTest("a<strong>bc</strong>d") {
        select(1, 4)
        commit("d")
        assertContent("a<strong>d</strong>", "ad", TextRange(2))
    }

    @Test
    fun `deleting a selected mention`() = editorTest(MENTION) {
        select(2, 3)
        backspace()
        assertContent("a  b", "a  b", TextRange(2))
        assertTrue(state.mentionsState?.userIds.orEmpty().isEmpty())
    }

    @Test
    fun `delete before a mention removes it`() = editorTest(MENTION) {
        select(2)
        forwardDelete()
        assertContent("a  b", "a  b", TextRange(2))
    }

    @Test
    fun `typing over a selection containing a mention`() = editorTest(MENTION) {
        select(1, 4)
        commit("x")
        assertContent("axb", "axb", TextRange(2))
    }

    // endregion

    // region Selection

    @Test
    fun `every selection is synced with the composer`() = editorTest(
        "<p>a<strong>b</strong></p><ul><li>c</li></ul>$MENTION<blockquote><p>d</p></blockquote><pre><code>e\nf</code></pre>"
    ) {
        val length = text.length
        for (start in 0..length) {
            for (end in 0..length) {
                // Also checks reversed selections, assertConsistent verifies the composer's one
                select(start, end)
            }
        }
    }

    @Test
    fun `action states follow the selection`() = editorTest(
        "<p>a<strong>b</strong></p><ul><li>c</li></ul><blockquote><p>d</p></blockquote><pre><code>e</code></pre>"
    ) {
        // Text: "ab\nc\nd\ne"
        select(1, 2)
        assertEquals(ActionState.REVERSED, state.actions[ComposerAction.BOLD])
        assertNotEquals(ActionState.REVERSED, state.actions[ComposerAction.UNORDERED_LIST])
        select(4)
        assertEquals(ActionState.REVERSED, state.actions[ComposerAction.UNORDERED_LIST])
        assertNotEquals(ActionState.REVERSED, state.actions[ComposerAction.BOLD])
        select(6)
        assertEquals(ActionState.REVERSED, state.actions[ComposerAction.QUOTE])
        assertNotEquals(ActionState.REVERSED, state.actions[ComposerAction.UNORDERED_LIST])
        select(8)
        assertEquals(ActionState.REVERSED, state.actions[ComposerAction.CODE_BLOCK])
        assertNotEquals(ActionState.REVERSED, state.actions[ComposerAction.QUOTE])
    }

    @Test
    fun `moving the cursor into and out of a suggestion`() = editorTest {
        type("@al bob")
        assertEquals(MenuAction.None, state.menuAction)
        select(3)
        assertTrue(state.menuAction is MenuAction.Suggestion)
        select(7)
        assertEquals(MenuAction.None, state.menuAction)
    }

    @Test
    fun `link action follows the selection`() = editorTest("<a href=\"https://element.io\">link</a> text") {
        select(2)
        assertEquals(LinkAction.SetLink("https://element.io"), state.linkAction)
        select(7)
        assertEquals(LinkAction.InsertLink, state.linkAction)
        select(6, 9)
        assertEquals(LinkAction.SetLink(null), state.linkAction)
    }

    @Test
    fun `formatting a selection spanning several blocks keeps the selection`() = editorTest(LIST) {
        select(1, 10)
        state.toggleInlineFormat(InlineFormat.Italic)
        assertEquals(
            "<ul><li>o<em>ne</em></li><li><em>two</em></li></ul><p><em>af</em>ter</p>",
            state.composerHtml(),
        )
        assertEquals(TextRange(1, 10), selection)
        assertConsistent()
    }

    @Test
    fun `setting a selection out of bounds clamps it`() = editorTest(PARAGRAPHS) {
        state.setSelection(-3, 100)
        state.onSelectionChanged(selection)
        assertEquals(TextRange(0, 7), selection)
        assertConsistent()
    }

    // endregion
}
