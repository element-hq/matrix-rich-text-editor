/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next

import androidx.compose.ui.text.TextRange
import io.element.android.wysiwyg.compose.next.internal.document.InlineFormatting
import io.element.android.wysiwyg.compose.next.internal.document.InlineRange
import io.element.android.wysiwyg.compose.next.internal.document.ListMarker
import io.element.android.wysiwyg.compose.next.internal.document.MENTION_PLACEHOLDER
import io.element.android.wysiwyg.view.models.InlineFormat
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction
import uniffi.wysiwyg_composer.MenuAction

private const val NBSP = ' '
private const val ALICE_URL = "https://matrix.to/#/@alice:matrix.org"

class RichTextEditorStateTest : RustTest() {
    @Test
    fun `typing plain text`() = editorTest {
        type("hello")
        assertEquals("hello", text)
        assertEquals("hello", html)
        assertEquals(TextRange(5), selection)
    }

    @Test
    fun `typing spaces keeps them as plain spaces in the text field`() = editorTest {
        type("a  b ")
        assertEquals("a  b ", text)
        assertEquals("a$NBSP${NBSP}b$NBSP", html)
    }

    @Test
    fun `IME replacing its composing text`() = editorTest {
        replace(0, 0, "helo")
        replace(0, 4, "hello ")
        assertEquals("hello ", text)
        assertEquals(TextRange(6), selection)
    }

    @Test
    fun `formatting applies to the text typed after it`() = editorTest {
        type("a")
        state.toggleInlineFormat(InlineFormat.Bold)
        assertEquals(ActionState.REVERSED, state.actions[ComposerAction.BOLD])
        type("b")
        assertEquals("a<strong>b</strong>", html)
        assertEquals(listOf(InlineRange(InlineFormatting.Bold, 1, 2)), state.document.inlineRanges)
    }

    @Test
    fun `formatting a selection`() = editorTest {
        type("hello world")
        select(6, 11)
        state.toggleInlineFormat(InlineFormat.Italic)
        assertEquals("hello <em>world</em>", html)
        assertEquals(TextRange(6, 11), selection)
    }

    @Test
    fun `enter creates new paragraphs`() = editorTest {
        type("ab")
        enter()
        enter()
        type("c")
        assertEquals("ab\n\nc", text)
        assertEquals("<p>ab</p><p>$NBSP</p><p>c</p>", html)
        assertEquals(TextRange(5), selection)
    }

    @Test
    fun `a paragraph with only a space is not an empty paragraph`() = editorTest {
        type("ab")
        enter()
        type(" ")
        assertEquals("ab\n ", text)
        assertEquals(TextRange(4), selection)
        type("x")
        assertEquals("ab\n x", text)
        assertEquals(TextRange(5), selection)
    }

    @Test
    fun `pasting text with line breaks`() = editorTest {
        commit("a\nb\nc")
        assertEquals("a\nb\nc", text)
        assertEquals("<p>a</p><p>b</p><p>c</p>", html)
        assertEquals(TextRange(5), selection)
    }

    @Test
    fun `backspace removes characters and paragraphs`() = editorTest {
        type("ab")
        enter()
        type("c")
        backspace()
        backspace()
        assertEquals("ab", text)
        backspace()
        assertEquals("a", text)
        assertEquals(TextRange(1), selection)
    }

    @Test
    fun `backspace removes a whole emoji`() = editorTest {
        type("a😀")
        assertEquals(TextRange(3), selection)
        backspace(count = 2)
        assertEquals("a", text)
    }

    @Test
    fun `creating and leaving a list`() = editorTest {
        state.toggleList(ordered = false)
        type("a")
        enter()
        type("b")
        enter()
        assertEquals("a\nb\n", text)
        assertEquals(listOf(ListMarker.Bullet(0), ListMarker.Bullet(0), ListMarker.Bullet(0)), state.document.lines.map { it.marker })

        // Enter in an empty list item removes it: the composer modifies what the user typed
        enter()
        assertEquals("<ul><li>a</li><li>b</li></ul><p>$NBSP</p>", html)
        assertEquals("a\nb\n", text)
        assertEquals(listOf(ListMarker.Bullet(0), ListMarker.Bullet(0), null), state.document.lines.map { it.marker })

        type("c")
        assertEquals("a\nb\nc", text)
    }

    @Test
    fun `ordered lists are numbered`() = editorTest {
        state.toggleList(ordered = true)
        type("a")
        enter()
        type("b")
        assertEquals(listOf(ListMarker.Number(1), ListMarker.Number(2)), state.document.lines.map { it.marker })
    }

    @Test
    fun `code blocks`() = editorTest {
        state.toggleCodeBlock()
        type("x")
        enter()
        type("y")
        assertEquals("x\ny", text)
        assertEquals("<pre><code>x\ny</code></pre>", html)
        assertTrue(state.document.lines.all { it.isCode })
        // Enter twice at the end exits the code block
        enter()
        enter()
        type("z")
        assertEquals("<pre><code>x\ny</code></pre><p>z</p>", html)
        assertEquals("x\ny\nz", text)
    }

    @Test
    fun `quotes`() = editorTest {
        type("x")
        state.toggleQuote()
        enter()
        type("y")
        assertEquals("<blockquote><p>x</p><p>y</p></blockquote>", html)
        assertEquals("x\ny", text)
    }

    @Test
    fun `mentions are a single character`() = editorTest {
        type("hi ")
        state.insertMention(text = "Alice", url = ALICE_URL)
        // A space is added after the mention
        assertEquals("hi $MENTION_PLACEHOLDER ", text)
        assertEquals(TextRange(5), selection)
        assertEquals(listOf(ALICE_URL), state.mentionsState?.userIds?.map { "https://matrix.to/#/$it" })

        type("!")
        assertEquals("hi $MENTION_PLACEHOLDER !", text)

        backspace()
        backspace()
        backspace()
        assertEquals("hi ", text)
        assertTrue(state.mentionsState?.userIds.orEmpty().isEmpty())
    }

    @Test
    fun `mention at suggestion`() = editorTest {
        type("hello @ali")
        val menuAction = state.menuAction
        assertTrue(menuAction is MenuAction.Suggestion)
        state.insertMentionAtSuggestion(text = "Alice", url = ALICE_URL)
        assertEquals("hello $MENTION_PLACEHOLDER ", text)
        assertEquals(TextRange(8), selection)
        assertEquals(MenuAction.None, state.menuAction)
    }

    @Test
    fun `at room mention at suggestion`() = editorTest {
        type("@ro")
        state.insertAtRoomMentionAtSuggestion()
        assertEquals("$MENTION_PLACEHOLDER ", text)
        assertEquals(true, state.document.mentions.single().isAtRoom)
    }

    @Test
    fun `links`() = editorTest {
        state.insertLink(url = "https://element.io", text = "Element")
        assertEquals("Element", text)
        assertEquals("<a href=\"https://element.io\">Element</a>", html)
        select(0, 7)
        state.removeLink()
        assertEquals("Element", html)
    }

    @Test
    fun `setting html`() = editorTest {
        state.setHtml("<ol><li><p>a</p><ul><li>b</li></ul></li><li>c</li></ol><p>d</p>")
        assertEquals("a\nb\nc\nd", text)
        assertEquals(TextRange(7), selection)
        assertConsistent()
        // Typing in the middle of the content
        select(3)
        type("x")
        assertEquals("a\nbx\nc\nd", text)
        assertEquals("<ol><li><p>a</p><ul><li>bx</li></ul></li><li>c</li></ol><p>d</p>", html)
    }

    @Test
    fun `initial html`() = editorTest(initialHtml = "<strong>hi</strong>") {
        assertEquals("hi", text)
        assertEquals("<strong>hi</strong>", html)
    }

    @Test
    fun `undo and redo`() = editorTest {
        type("ab")
        state.toggleInlineFormat(InlineFormat.Bold)
        type("c")
        state.undo()
        assertConsistent()
        assertEquals("ab", text)
        state.redo()
        assertConsistent()
        assertEquals("abc", text)
        assertEquals("ab<strong>c</strong>", html)
    }

    @Test
    fun `clear`() = editorTest {
        type("hello")
        enter()
        state.clear()
        assertEquals("", text)
        assertEquals(TextRange(0), selection)
    }

    @Test
    fun `message html`() = editorTest {
        type("a")
        enter()
        type("b")
        assertEquals("a<br />b", state.getMessageHtml())
        assertEquals("a\nb", state.getMessageMarkdown())
    }
}
