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
 * Zero width space, used to replace line breaks between paragraphs in the displayed text.
 */
internal const val ZWSP = "\u200B"

/**
 * Describes how to display a [RichTextDocument]: how its logical text must be transformed into
 * the *visual* text and which styles and decorations apply to it.
 *
 * Consecutive lines with the same indentation are displayed as a single paragraph, keeping their
 * line breaks. When the indentation changes a new paragraph must start, but Compose already
 * starts a new line for each paragraph and a line break at the end of a paragraph would add an
 * extra empty line, so the line break between paragraphs is replaced by a [ZWSP] character.
 * Since this replacement has the same length, it doesn't affect the indexes: only mentions and
 * an empty last line change the length of the visual text.
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
    /** Logical indexes of the line breaks replaced by [ZWSP] because they end a paragraph. */
    val paragraphBreaks: Set<Int>,
    val mapper: OffsetMapper,
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

        // Group consecutive lines with the same indentation in the same paragraph
        val lineIndents = document.lines.map { indentPx(it.indents) }
        val paragraphs = mutableListOf<IntRange>()
        var firstLine = 0
        for (index in 1..document.lines.size) {
            if (index == document.lines.size || lineIndents[index] != lineIndents[firstLine]) {
                paragraphs += firstLine until index
                firstLine = index
            }
        }
        // Replace the line breaks between paragraphs
        val paragraphBreaks = paragraphs.dropLast(1).map { document.lines[it.last].end }.toSet()
        for (index in paragraphBreaks) {
            edits += TextEdit(index, index + 1, ZWSP)
        }
        // Replace mention placeholders with their display text
        val mentionDisplays = document.mentions.map { mention ->
            val display = resolveMentionDisplay(Mention(mention.url, mention.text, mention.isAtRoom))
            edits += TextEdit(mention.offset, mention.offset + 1, display.text.ifEmpty { mention.text.ifEmpty { "@" } })
            mention to display
        }
        // An empty last line needs some content so its paragraph's indentation applies to it and
        // the cursor is placed after it: Android doesn't apply leading margins to the empty line
        // after a trailing line break. This character only exists in the visual text, the text
        // field maps the cursor and selection around it.
        val lastLine = document.lines.last()
        if (lastLine.start == lastLine.end) {
            edits += TextEdit(text.length, text.length, ZWSP)
        }
        edits.sortBy { it.start }

        val mapper = OffsetMapper(edits)
        val visualLength = mapper.mapEnd(text.length)

        // The visual range of each line, including its line break
        val lineRanges = document.lines.mapIndexed { index, line ->
            val start = mapper.mapStart(line.start)
            // A line ends where the next one starts, right after the line break
            val end = if (index == document.lines.lastIndex) visualLength else mapper.mapStart(line.end + 1)
            start to end
        }
        val paragraphStyles = paragraphs.map { lines ->
            val indentPx = lineIndents[lines.first]
            val paragraphStyle = if (indentPx > 0f) {
                val indent = with(density) { indentPx.toSp() }
                ParagraphStyle(textIndent = TextIndent(firstLine = indent, restLine = indent))
            } else {
                ParagraphStyle()
            }
            AnnotatedString.Range(paragraphStyle, lineRanges[lines.first].first, lineRanges[lines.last].second)
        }
        document.lines.forEachIndexed { index, line ->
            line.marker?.let { marker ->
                val endX = lineIndents[index] - with(density) { style.list.markerGap.toPx() }
                decorations += Decoration.Marker(lineRanges[index].first, marker, endX, style.list.markerColor)
            }
            if (line.isCode && line.end > line.start) {
                styled += Styled(line.start, line.end, style.codeBlock.spanStyle)
            }
        }

        // Blocks
        for (block in document.blocks) {
            val start = lineRanges[block.firstLine].first
            val end = lineRanges[block.lastLine].second
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
            paragraphBreaks = paragraphBreaks,
            mapper = mapper,
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
