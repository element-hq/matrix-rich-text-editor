/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next.internal.document

import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * The expected texts and lengths here were taken from the actual output of the Rust model.
 */
class HtmlDocumentParserTest {
    private fun parse(html: String, plainText: String? = null) =
        HtmlDocumentParser.parse(html) { plainText }

    @Test
    fun `plain text`() {
        val document = parse("hello")
        assertEquals("hello", document.text)
        assertEquals(listOf(DocumentLine(0, 5, emptyList(), null, false)), document.lines)
    }

    @Test
    fun `paragraphs are separated by a single line break and placeholders are removed`() {
        // Rust: "ab", enter, enter, "c" -> selection at 5
        val document = parse("<p>ab</p><p> </p><p>c</p>", plainText = "ab\n\nc\n")
        assertEquals("ab\n\nc", document.text)
        assertEquals(listOf(0 to 2, 3 to 3, 4 to 5), document.lines.map { it.start to it.end })
    }

    @Test
    fun `ambiguous paragraph containing a space is resolved with the plain text`() {
        // Rust: "ab", enter, " " -> selection at 4
        val document = parse("<p>ab</p><p> </p>", plainText = "ab\n \n")
        assertEquals("ab\n ", document.text)
    }

    @Test
    fun `several ambiguous paragraphs are resolved`() {
        // Rust: "ab", enter, " ", enter -> selection at 5
        val document = parse("<p>ab</p><p> </p><p> </p>", plainText = "ab\n \n\n")
        assertEquals("ab\n \n", document.text)
    }

    @Test
    fun `ambiguous paragraphs are empty if plain text is not available`() {
        val document = parse("<p>ab</p><p> </p>", plainText = null)
        assertEquals("ab\n", document.text)
    }

    @Test
    fun `nbsp characters in text are normalised to spaces`() {
        val document = parse("a  b ")
        assertEquals("a  b ", document.text)
    }

    @Test
    fun `lists have markers in the first line of each item`() {
        val document = parse("<ul><li>a</li><li>b</li><li></li></ul>")
        assertEquals("a\nb\n", document.text)
        assertEquals(
            listOf(
                DocumentLine(0, 1, listOf(IndentKind.List), ListMarker.Bullet(0), false),
                DocumentLine(2, 3, listOf(IndentKind.List), ListMarker.Bullet(0), false),
                DocumentLine(4, 4, listOf(IndentKind.List), ListMarker.Bullet(0), false),
            ),
            document.lines,
        )
    }

    @Test
    fun `nested lists`() {
        val document = parse("<ol><li><p>a</p><ul><li>b</li></ul></li><li>c</li></ol><p>d</p>")
        assertEquals("a\nb\nc\nd", document.text)
        assertEquals(
            listOf(
                listOf(IndentKind.List) to ListMarker.Number(1),
                listOf(IndentKind.List, IndentKind.List) to ListMarker.Bullet(1),
                listOf(IndentKind.List) to ListMarker.Number(2),
                emptyList<IndentKind>() to null,
            ),
            document.lines.map { it.indents to it.marker },
        )
    }

    @Test
    fun `ordered list with custom start`() {
        val document = parse("<ol start=\"3\"><li>a</li><li>b</li></ol>")
        assertEquals(listOf(ListMarker.Number(3), ListMarker.Number(4)), document.lines.map { it.marker })
    }

    @Test
    fun `code block lines and placeholders`() {
        // Rust: code block, "x", enter -> selection at 2
        val document = parse("<pre><code>x\n </code></pre>")
        assertEquals("x\n", document.text)
        assertEquals(listOf(true, true), document.lines.map { it.isCode })
        assertEquals(
            listOf(DocumentBlock(BlockKind.CodeBlock, emptyList(), 0, 1)),
            document.blocks,
        )
    }

    @Test
    fun `empty code block`() {
        val document = parse("<pre><code> </code></pre>")
        assertEquals("", document.text)
        assertEquals(1, document.lines.size)
    }

    @Test
    fun `code block keeps nbsp characters in the middle of the text`() {
        val document = parse("<pre><code>a b</code></pre>")
        assertEquals("a b", document.text)
    }

    @Test
    fun `quote followed by paragraph`() {
        val document = parse("<blockquote><p>x</p></blockquote><p> </p>", plainText = "x\n\n")
        assertEquals("x\n", document.text)
        assertEquals(listOf(listOf(IndentKind.Quote), emptyList()), document.lines.map { it.indents })
        assertEquals(listOf(DocumentBlock(BlockKind.Quote, emptyList(), 0, 0)), document.blocks)
    }

    @Test
    fun `mentions are a single placeholder character`() {
        val html = "hi <a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\" " +
            "contenteditable=\"false\">Alice</a> "
        val document = parse(html)
        assertEquals("hi $MENTION_PLACEHOLDER ", document.text)
        assertEquals(
            listOf(MentionRange(3, "https://matrix.to/#/@alice:matrix.org", "Alice", isAtRoom = false)),
            document.mentions,
        )
    }

    @Test
    fun `at room mention`() {
        val document = parse("<a data-mention-type=\"at-room\" href=\"#\" contenteditable=\"false\">@room</a> ")
        assertEquals("$MENTION_PLACEHOLDER ", document.text)
        assertEquals(true, document.mentions.single().isAtRoom)
    }

    @Test
    fun `inline formatting ranges`() {
        val document = parse("a<strong>b<em>c</em></strong><code>d</code><a href=\"https://x.org\">e</a>")
        assertEquals("abcde", document.text)
        assertEquals(
            setOf(
                InlineRange(InlineFormatting.Italic, 2, 3),
                InlineRange(InlineFormatting.Bold, 1, 3),
                InlineRange(InlineFormatting.InlineCode, 3, 4),
                InlineRange(InlineFormatting.Link("https://x.org"), 4, 5),
            ),
            document.inlineRanges.toSet(),
        )
    }

    @Test
    fun `html entities are decoded`() {
        assertEquals("a<b>&c", parse("a&lt;b&gt;&amp;c").text)
    }
}
