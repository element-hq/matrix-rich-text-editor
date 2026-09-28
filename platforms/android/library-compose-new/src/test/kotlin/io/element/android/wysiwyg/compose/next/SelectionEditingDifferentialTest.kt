/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next

import androidx.compose.ui.text.TextRange
import io.element.android.wysiwyg.compose.next.internal.toJvmString
import org.junit.Assert.fail
import org.junit.Test
import uniffi.wysiwyg_composer.ComposerModel
import uniffi.wysiwyg_composer.newComposerModel

/**
 * For every possible selection in several documents, applies each kind of edit through the editor
 * (as a text field would) and the equivalent operation directly on a separate Rust composer, then
 * checks that both end up with the same content and selection.
 *
 * This verifies the translation of text field edits into composer operations, especially at block
 * boundaries and for selections spanning several blocks.
 */
class SelectionEditingDifferentialTest : RustTest() {
    private enum class Operation {
        /** Typing a character, replacing the selection if any. */
        Type,
        Backspace,
        ForwardDelete,
        Enter,
        /** Pasting text containing a line break, replacing the selection if any. */
        PasteMultiline,
        /** Typing the first selected character over the selection. */
        TypeFirstSelectedChar,
        /** Typing the last selected character over the selection. */
        TypeLastSelectedChar,
        /** The IME deleting the 2 characters before the cursor, i.e. deleting a word. */
        DeleteTwoBefore,
    }

    @Test
    fun `paragraphs`() = checkAllSelections("<p>abc</p><p>def</p>")

    @Test
    fun `empty paragraphs`() = checkAllSelections("<p>a</p><p> </p><p> </p><p>b</p>")

    @Test
    fun `paragraph with a single space`() = checkAllSelections("<p>a</p><p> </p><p>b</p>")

    @Test
    fun `list items`() = checkAllSelections("<ul><li>one</li><li>two</li></ul><p>after</p>")

    @Test
    fun `paragraph before a list`() = checkAllSelections("<p>ab</p><ol><li>cd</li><li>ef</li></ol>")

    @Test
    fun `empty list item`() = checkAllSelections("<ul><li>a</li><li></li><li>b</li></ul>")

    @Test
    fun `nested lists`() = checkAllSelections("<ol><li><p>a</p><ul><li>bc</li></ul></li><li>d</li></ol>")

    @Test
    fun `quote`() = checkAllSelections("<p>a</p><blockquote><p>bc</p><p>d</p></blockquote><p>e</p>")

    @Test
    fun `code block`() = checkAllSelections("<p>a</p><pre><code>bc\nd</code></pre><p>e</p>")

    @Test
    fun `quote followed by code block`() = checkAllSelections("<blockquote><p>ab</p></blockquote><pre><code>cd</code></pre>")

    @Test
    fun `inline formatting`() = checkAllSelections("a<strong>bc</strong><em>d</em>e")

    @Test
    fun `formatting in several paragraphs`() = checkAllSelections("<p>a<strong>b</strong></p><p><strong>c</strong>d</p>")

    @Test
    fun `inline code`() = checkAllSelections("a<code>bc</code>d")

    @Test
    fun `link`() = checkAllSelections("<a href=\"https://element.io\">ab</a> c")

    @Test
    fun `mention`() = checkAllSelections(
        "a <a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a> b"
    )

    @Test
    fun `mentions in paragraphs`() = checkAllSelections(
        "<p><a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a></p>" +
            "<p>b<a data-mention-type=\"at-room\" href=\"#\" contenteditable=\"false\">@room</a></p>"
    )

    @Test
    fun `emoji`() = checkAllSelections("a😀b")

    private fun checkAllSelections(html: String) {
        val length = EditorTester(html).use { it.text.length }
        val text = EditorTester(html).use { it.text }
        val failures = mutableListOf<String>()
        for (start in 0..length) {
            for (end in start..length) {
                if (splitsSurrogatePair(text, start) || splitsSurrogatePair(text, end)) continue
                for (operation in Operation.entries) {
                    if (start == end && operation == Operation.Backspace && start == 0) continue
                    if (start == end && operation == Operation.ForwardDelete && start == length) continue
                    if (operation == Operation.TypeFirstSelectedChar && text.getOrNull(start)?.isLetter() != true) continue
                    if (operation == Operation.TypeLastSelectedChar && (end == start || !text[end - 1].isLetter())) continue
                    if (operation == Operation.TypeFirstSelectedChar && end == start) continue
                    if (operation == Operation.DeleteTwoBefore && (start != end || start < 2 || splitsSurrogatePair(text, start - 2))) continue
                    val failure = runCatching { check(html, text, TextRange(start, end), operation) }
                        .exceptionOrNull()
                    if (failure != null) {
                        failures += "[$start, $end) $operation: ${failure.message}"
                    }
                }
            }
        }
        if (failures.isNotEmpty()) {
            fail("${failures.size} failures for $html:\n" + failures.joinToString("\n"))
        }
    }

    private fun check(html: String, text: String, selection: TextRange, operation: Operation) {
        val typed = when (operation) {
            Operation.Type -> "x"
            Operation.PasteMultiline -> "p\nq"
            Operation.TypeFirstSelectedChar -> text[selection.start].toString()
            Operation.TypeLastSelectedChar -> text[selection.end - 1].toString()
            else -> null
        }
        val (expectedHtml, expectedSelection) = runOnComposer(html, selection, operation, typed)
        EditorTester(html).use { editor ->
            editor.select(selection.start, selection.end)
            when (operation) {
                Operation.Backspace -> editor.backspace(count = codeUnitsBefore(editor.text, selection))
                Operation.ForwardDelete -> editor.forwardDelete(count = codeUnitsAfter(editor.text, selection))
                Operation.Enter -> editor.enter()
                Operation.DeleteTwoBefore -> editor.backspace(count = 2)
                else -> editor.commit(typed!!)
            }
            val actualHtml = editor.state.composerHtml()
            if (actualHtml != expectedHtml || editor.selection != expectedSelection) {
                throw AssertionError(
                    "expected '$expectedHtml' $expectedSelection but was '$actualHtml' ${editor.selection}"
                )
            }
        }
    }

    /**
     * Applies [operation] directly to a Rust composer, returning its HTML and selection.
     */
    private fun runOnComposer(
        html: String,
        selection: TextRange,
        operation: Operation,
        typed: String?,
    ): Pair<String, TextRange> {
        fun ComposerModel.type(value: String) {
            value.split('\n').forEachIndexed { index, segment ->
                if (index > 0) enter()
                if (segment.isNotEmpty()) replaceText(segment)
            }
        }
        fun run(): Pair<String, TextRange> {
            val composer: ComposerModel = newComposerModel()
            try {
                composer.setContentFromHtml(html)
                composer.select(selection.start.toUInt(), selection.end.toUInt())
                when (operation) {
                    Operation.Backspace -> composer.backspace()
                    Operation.ForwardDelete -> composer.delete()
                    Operation.DeleteTwoBefore -> composer.deleteIn((selection.start - 2).toUInt(), selection.start.toUInt())
                    Operation.Enter -> composer.enter()
                    else -> composer.type(typed!!)
                }
                val state = composer.getCurrentDomState()
                return state.html.toJvmString() to TextRange(state.start.toInt(), state.end.toInt())
            } finally {
                composer.destroy()
            }
        }
        return run()
    }

    private fun splitsSurrogatePair(text: String, index: Int) =
        index in 1 until text.length && text[index].isLowSurrogate()

    private fun codeUnitsBefore(text: String, selection: TextRange) =
        if (selection.collapsed && selection.start >= 2 && text[selection.start - 1].isLowSurrogate()) 2 else 1

    private fun codeUnitsAfter(text: String, selection: TextRange) =
        if (selection.collapsed && selection.start < text.length && text[selection.start].isHighSurrogate()) 2 else 1
}
