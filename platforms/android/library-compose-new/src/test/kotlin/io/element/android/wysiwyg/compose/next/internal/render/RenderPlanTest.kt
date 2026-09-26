/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next.internal.render

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.dp
import io.element.android.wysiwyg.compose.next.CodeBlockStyle
import io.element.android.wysiwyg.compose.next.InlineCodeStyle
import io.element.android.wysiwyg.compose.next.ListStyle
import io.element.android.wysiwyg.compose.next.MentionDisplay
import io.element.android.wysiwyg.compose.next.MentionStyle
import io.element.android.wysiwyg.compose.next.QuoteStyle
import io.element.android.wysiwyg.compose.next.RichTextEditorStyle
import io.element.android.wysiwyg.compose.next.internal.TextEdit
import io.element.android.wysiwyg.compose.next.internal.document.HtmlDocumentParser
import io.element.android.wysiwyg.compose.next.internal.document.ListMarker
import org.junit.Assert.assertEquals
import org.junit.Test

class RenderPlanTest {
    private val style = RichTextEditorStyle(
        textStyle = TextStyle(),
        cursorBrush = SolidColor(Color.Black),
        placeholderColor = Color.Gray,
        link = SpanStyle(color = Color.Blue),
        inlineCode = InlineCodeStyle(SpanStyle(), Color.Gray, 0.dp, 0.dp),
        codeBlock = CodeBlockStyle(SpanStyle(), Color.Gray, 0.dp, 10.dp),
        quote = QuoteStyle(Color.Gray, 2.dp, 12.dp),
        list = ListStyle(indent = 20.dp, markerGap = 4.dp, bulletRadius = 2.dp, markerColor = Color.Black),
        mention = MentionStyle(SpanStyle(), Color.Gray, 0.dp, 0.dp),
    )
    private val builder = RenderPlanBuilder(style, Density(1f)) { MentionDisplay.Pill("@${it.text}") }

    private fun plan(html: String) = builder.build(HtmlDocumentParser.parse(html) { null })

    private fun RenderPlan.visualText(): String {
        val result = StringBuilder(logicalText)
        edits.asReversed().forEach { result.replace(it.start, it.end, it.replacement) }
        return result.toString()
    }

    @Test
    fun `line breaks are replaced by zero width spaces keeping the same length`() {
        val plan = plan("<p>ab</p><p>c</p>")
        assertEquals("ab${ZWSP}c", plan.visualText())
        assertEquals(4, plan.visualLength)
        assertEquals(listOf(0 to 3, 3 to 4), plan.paragraphStyles.map { it.start to it.end })
    }

    @Test
    fun `empty last line gets a zero width space`() {
        val plan = plan("<ul><li>a</li><li></li></ul>")
        assertEquals("a$ZWSP$ZWSP", plan.visualText())
        assertEquals(listOf(0 to 2, 2 to 3), plan.paragraphStyles.map { it.start to it.end })
        // Each list item has a marker at the start of its paragraph, before the indentation
        assertEquals(
            listOf(0 to ListMarker.Bullet(0), 2 to ListMarker.Bullet(0)),
            plan.decorations.filterIsInstance<Decoration.Marker>().map { it.offset to it.marker },
        )
        assertEquals(16f, plan.decorations.filterIsInstance<Decoration.Marker>().first().endX)
    }

    @Test
    fun `empty document`() {
        val plan = plan("")
        assertEquals(ZWSP, plan.visualText())
        assertEquals(listOf(0 to 1), plan.paragraphStyles.map { it.start to it.end })
    }

    @Test
    fun `mentions are replaced by their display text and offsets are mapped`() {
        val plan = plan(
            "<strong>hi <a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\">Alice</a>!</strong>"
        )
        assertEquals("hi @Alice!", plan.visualText())
        assertEquals(listOf(TextEdit(3, 4, "@Alice")), plan.edits)
        val pill = plan.decorations.filterIsInstance<Decoration.InlineBackground>().single()
        assertEquals(3 to 9, pill.start to pill.end)
        // Bold covers the whole visual text
        val boldRanges = plan.spanStyles.filter { it.item.fontWeight == FontWeight.Bold }
        assertEquals(0, boldRanges.minOf { it.start })
        assertEquals(10, boldRanges.maxOf { it.end })
    }

    @Test
    fun `overlapping text decorations are combined`() {
        val plan = plan("<u>a<del>b</del></u>")
        val decorations = plan.spanStyles.map { Triple(it.start, it.end, it.item.textDecoration) }
        assertEquals(
            listOf(
                Triple(0, 1, TextDecoration.Underline),
                Triple(1, 2, TextDecoration.combine(listOf(TextDecoration.Underline, TextDecoration.LineThrough))),
            ),
            decorations,
        )
    }

    @Test
    fun `blocks decorations cover their lines`() {
        val plan = plan("<p>a</p><blockquote><p>b</p><p>c</p></blockquote><pre><code>d</code></pre>")
        assertEquals("a${ZWSP}b${ZWSP}c${ZWSP}d", plan.visualText())
        val bar = plan.decorations.filterIsInstance<Decoration.Bar>().single()
        assertEquals(2 to 6, bar.start to bar.end)
        val code = plan.decorations.filterIsInstance<Decoration.BlockBackground>().single()
        assertEquals(6 to 7, code.start to code.end)
    }
}
