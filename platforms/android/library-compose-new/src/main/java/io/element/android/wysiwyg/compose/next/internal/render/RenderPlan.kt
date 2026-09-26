/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next.internal.render

import androidx.compose.runtime.Immutable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.ParagraphStyle
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextIndent
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.em
import io.element.android.wysiwyg.compose.next.Mention
import io.element.android.wysiwyg.compose.next.MentionDisplay
import io.element.android.wysiwyg.compose.next.RichTextEditorStyle
import io.element.android.wysiwyg.compose.next.internal.TextEdit
import io.element.android.wysiwyg.compose.next.internal.document.BlockKind
import io.element.android.wysiwyg.compose.next.internal.document.IndentKind
import io.element.android.wysiwyg.compose.next.internal.document.InlineFormatting
import io.element.android.wysiwyg.compose.next.internal.document.ListMarker
import io.element.android.wysiwyg.compose.next.internal.document.RichTextDocument

/**
 * Zero width space, used to replace line breaks in the displayed text.
 */
internal const val ZWSP = "​"

/**
 * Describes how to display a [RichTextDocument]: how its logical text must be transformed into
 * the *visual* text and which styles and decorations apply to it.
 *
 * Every line of the document is displayed as its own paragraph (so it can have its own
 * indentation). Compose already starts a new line for each paragraph and a line break at the end
 * of a paragraph would add an extra empty line, so line breaks are replaced by [ZWSP] characters.
 * Since this replacement has the same length, it doesn't affect the indexes: only mentions and
 * empty last lines change the length of the visual text.
 *
 * All the ranges in [paragraphStyles], [spanStyles] and [decorations] use visual text indexes.
 */
@Immutable
internal class RenderPlan(
    val logicalText: String,
    /** Edits to apply to the logical text to get the visual one, sorted by position. */
    val edits: List<TextEdit>,
    val visualLength: Int,
    val paragraphStyles: List<AnnotatedString.Range<ParagraphStyle>>,
    val spanStyles: List<AnnotatedString.Range<SpanStyle>>,
    val decorations: List<Decoration>,
)

@Immutable
internal sealed interface Decoration {
    /** A list item marker, drawn before [endX] in the first visual line containing [offset]. */
    data class Marker(val offset: Int, val marker: ListMarker, val endX: Float, val color: Color) : Decoration

    /** A vertical bar for quotes. */
    data class Bar(val start: Int, val end: Int, val x: Float, val width: Float, val color: Color) : Decoration

    /** A background covering all the lines in the range, from [x] to the end of the text field. */
    data class BlockBackground(
        val start: Int,
        val end: Int,
        val x: Float,
        val color: Color,
        val cornerRadius: Float,
    ) : Decoration

    /** A background covering only the text in the range, like a pill for a mention. */
    data class InlineBackground(
        val start: Int,
        val end: Int,
        val color: Color,
        val cornerRadius: Float,
        val horizontalPadding: Float,
    ) : Decoration
}

internal class RenderPlanBuilder(
    private val style: RichTextEditorStyle,
    private val density: Density,
    private val resolveMentionDisplay: (Mention) -> MentionDisplay,
) {
    private class Styled(val start: Int, val end: Int, val style: SpanStyle)

    fun build(document: RichTextDocument): RenderPlan {
        val text = document.text
        val edits = mutableListOf<TextEdit>()
        val decorations = mutableListOf<Decoration>()
        val styled = mutableListOf<Styled>()

        // Replace line breaks
        text.forEachIndexed { index, char ->
            if (char == '\n') edits += TextEdit(index, index + 1, ZWSP)
        }
        // Replace mention placeholders with their display text
        val mentionDisplays = document.mentions.map { mention ->
            val display = resolveMentionDisplay(Mention(mention.url, mention.text, mention.isAtRoom))
            edits += TextEdit(mention.offset, mention.offset + 1, display.text.ifEmpty { mention.text.ifEmpty { "@" } })
            mention to display
        }
        // An empty last line needs some content to be styled and to have the cursor placed in it
        val lastLine = document.lines.last()
        if (lastLine.start == lastLine.end) {
            edits += TextEdit(text.length, text.length, ZWSP)
        }
        edits.sortBy { it.start }

        val mapper = OffsetMapper(edits)
        val visualLength = mapper.mapEnd(text.length)

        // Paragraphs, one per line
        val paragraphRanges = document.lines.mapIndexed { index, line ->
            val start = mapper.mapStart(line.start)
            // A paragraph ends where the next one starts, right after the line break
            val end = if (index == document.lines.lastIndex) visualLength else mapper.mapStart(line.end + 1)
            start to end
        }
        val paragraphStyles = document.lines.mapIndexed { index, line ->
            val (start, end) = paragraphRanges[index]
            val indentPx = indentPx(line.indents)
            val paragraphStyle = if (indentPx > 0f) {
                val indent = with(density) { indentPx.toSp() }
                ParagraphStyle(textIndent = TextIndent(firstLine = indent, restLine = indent))
            } else {
                ParagraphStyle()
            }
            line.marker?.let { marker ->
                val endX = indentPx - with(density) { style.list.markerGap.toPx() }
                decorations += Decoration.Marker(start, marker, endX, style.list.markerColor)
            }
            if (line.isCode && line.end > line.start) {
                styled += Styled(line.start, line.end, style.codeBlock.spanStyle)
            }
            AnnotatedString.Range(paragraphStyle, start, end)
        }

        // Blocks
        for (block in document.blocks) {
            val start = paragraphRanges[block.firstLine].first
            val end = paragraphRanges[block.lastLine].second
            val x = indentPx(block.outerIndents)
            decorations += when (block.kind) {
                BlockKind.Quote -> Decoration.Bar(
                    start = start,
                    end = end,
                    x = x,
                    width = with(density) { style.quote.barWidth.toPx() },
                    color = style.quote.barColor,
                )
                BlockKind.CodeBlock -> Decoration.BlockBackground(
                    start = start,
                    end = end,
                    x = x,
                    color = style.codeBlock.background,
                    cornerRadius = with(density) { style.codeBlock.cornerRadius.toPx() },
                )
            }
        }

        // Inline formatting
        for (range in document.inlineRanges) {
            styled += Styled(range.start, range.end, range.format.toSpanStyle())
            if (range.format == InlineFormatting.InlineCode) {
                decorations += Decoration.InlineBackground(
                    start = mapper.mapStart(range.start),
                    end = mapper.mapEnd(range.end),
                    color = style.inlineCode.background,
                    cornerRadius = with(density) { style.inlineCode.cornerRadius.toPx() },
                    horizontalPadding = with(density) { style.inlineCode.horizontalPadding.toPx() },
                )
            }
        }
        for ((mention, display) in mentionDisplays) {
            styled += Styled(mention.offset, mention.offset + 1, style.mention.spanStyle)
            if (display is MentionDisplay.Pill) {
                decorations += Decoration.InlineBackground(
                    start = mapper.mapStart(mention.offset),
                    end = mapper.mapEnd(mention.offset + 1),
                    color = style.mention.pillBackground,
                    cornerRadius = with(density) { style.mention.cornerRadius.toPx() },
                    horizontalPadding = with(density) { style.mention.horizontalPadding.toPx() },
                )
            }
        }

        return RenderPlan(
            logicalText = text,
            edits = edits,
            visualLength = visualLength,
            paragraphStyles = paragraphStyles,
            spanStyles = mergeSpanStyles(styled, mapper),
            decorations = decorations,
        )
    }

    private fun indentPx(indents: List<IndentKind>): Float = with(density) {
        indents.sumOf { kind ->
            when (kind) {
                IndentKind.List -> style.list.indent
                IndentKind.Quote -> style.quote.indent
                IndentKind.CodeBlock -> style.codeBlock.padding
            }.toPx().toDouble()
        }.toFloat()
    }

    private fun InlineFormatting.toSpanStyle(): SpanStyle = when (this) {
        InlineFormatting.Bold -> SpanStyle(fontWeight = FontWeight.Bold)
        InlineFormatting.Italic -> SpanStyle(fontStyle = FontStyle.Italic)
        InlineFormatting.Underline -> SpanStyle(textDecoration = TextDecoration.Underline)
        InlineFormatting.StrikeThrough -> SpanStyle(textDecoration = TextDecoration.LineThrough)
        InlineFormatting.InlineCode -> style.inlineCode.spanStyle
        is InlineFormatting.Link -> style.link
        is InlineFormatting.Heading -> SpanStyle(
            fontWeight = FontWeight.Bold,
            fontSize = when (level) {
                1 -> 1.5.em
                2 -> 1.35.em
                3 -> 1.2.em
                else -> 1.1.em
            },
        )
    }

    /**
     * Overlapping span styles override each other's text decorations, so split the styled ranges
     * into non-overlapping segments where the styles are merged and decorations combined.
     */
    private fun mergeSpanStyles(styled: List<Styled>, mapper: OffsetMapper): List<AnnotatedString.Range<SpanStyle>> {
        if (styled.isEmpty()) return emptyList()
        val boundaries = styled.flatMap { listOf(it.start, it.end) }.distinct().sorted()
        return boundaries.zipWithNext().mapNotNull { (start, end) ->
            val active = styled.filter { it.start <= start && it.end >= end }
            if (active.isEmpty()) return@mapNotNull null
            val merged = active.fold(SpanStyle()) { acc, item ->
                val decorations = listOfNotNull(acc.textDecoration, item.style.textDecoration)
                acc.merge(item.style).copy(
                    textDecoration = decorations.takeIf { it.isNotEmpty() }?.let { TextDecoration.combine(it) }
                )
            }
            AnnotatedString.Range(merged, mapper.mapStart(start), mapper.mapEnd(end))
        }
    }
}

/**
 * Maps logical text offsets to visual text ones given the [edits] applied to the logical text.
 */
internal class OffsetMapper(edits: List<TextEdit>) {
    private val lengthChangingEdits = edits.filter { it.replacement.length != it.end - it.start }

    /** Maps an offset where a range starts, so it doesn't include text inserted at [offset]. */
    fun mapStart(offset: Int): Int = offset + lengthChangingEdits.sumOf { edit ->
        val isBefore = edit.end < offset || (edit.end == offset && edit.start < edit.end)
        if (isBefore) edit.delta else 0
    }

    /** Maps an offset where a range ends, so it includes any text inserted at [offset]. */
    fun mapEnd(offset: Int): Int = offset + lengthChangingEdits.sumOf { edit ->
        if (edit.end <= offset) edit.delta else 0
    }

    private val TextEdit.delta get() = replacement.length - (end - start)
}
