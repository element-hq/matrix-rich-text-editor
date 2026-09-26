/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next.internal.document

import org.jsoup.Jsoup
import org.jsoup.nodes.Element
import org.jsoup.nodes.TextNode

/**
 * Parses the internal HTML of the Rust composer into a [RichTextDocument].
 *
 * The resulting [RichTextDocument.text] must have exactly the same length as the Rust model's
 * content, so the parsing rules here mirror how Rust computes its text length:
 *
 * - A container adds a separator character between each pair of consecutive block children.
 * - Mentions count as a single character.
 * - Placeholder `&nbsp;` characters added to empty paragraphs are not counted.
 */
internal object HtmlDocumentParser {
    /**
     * @param html the internal HTML of the composer, as returned in a `TextUpdate.ReplaceAll`.
     * @param plainTextProvider provides the composer's plain text. It's only called when the HTML
     * is ambiguous, see [PlaceholderResolver].
     */
    fun parse(html: String, plainTextProvider: () -> String?): RichTextDocument {
        if (html.isEmpty()) return RichTextDocument.Empty
        val body = Jsoup.parseBodyFragment(html).body()
        val spaceParagraphs = PlaceholderResolver.resolveSpaceParagraphs(body, plainTextProvider)
        return DocumentBuilder(spaceParagraphs).build(body)
    }
}

internal fun Element.isBlockElement(): Boolean = tagName() in BLOCK_TAGS

/**
 * Rust writes a single `&nbsp;` inside empty paragraphs, but the same HTML is produced for a
 * paragraph containing a single space, so these paragraphs can't be resolved by looking only at
 * the HTML.
 */
internal fun Element.isAmbiguousPlaceholderParagraph(): Boolean {
    if (tagName() != "p") return false
    val child = childNodes().singleOrNull() as? TextNode ?: return false
    return child.wholeText == NBSP.toString()
}

/**
 * Rust writes a `&nbsp;` in empty lines of code blocks if they're the first or last line.
 * Returns the offsets in [Element.wholeText] of these placeholders.
 */
internal fun Element.codeBlockPlaceholderOffsets(): Set<Int> {
    val text = wholeText()
    val lines = text.split('\n')
    return buildSet {
        if (lines.first() == NBSP.toString()) add(0)
        if (lines.size > 1 && lines.last() == NBSP.toString()) add(text.length - 1)
    }
}

private val BLOCK_TAGS = setOf(
    "p", "ul", "ol", "li", "pre", "blockquote", "div", "details", "summary",
    "h1", "h2", "h3", "h4", "h5", "h6",
)

private class DocumentBuilder(
    private val spaceParagraphs: Set<Element>,
) {
    private sealed interface Context {
        class ListContainer : Context
        class ListItem(val marker: ListMarker, val firstLine: Int) : Context
        class Quote : Context
        class CodeBlock : Context
        class Paragraph : Context
    }

    private class CodeState(val placeholderOffsets: Set<Int>) {
        var position = 0
    }

    private val text = StringBuilder()
    private val lineStarts = mutableListOf(0)
    private val lineContexts = mutableListOf<List<Context>>(emptyList())
    private val stack = ArrayList<Context>()
    private val blocks = mutableListOf<DocumentBlock>()
    private val inlineRanges = mutableListOf<InlineRange>()
    private val mentions = mutableListOf<MentionRange>()
    private var codeState: CodeState? = null

    private val currentLine get() = lineStarts.lastIndex

    fun build(body: Element): RichTextDocument {
        parseChildren(body)
        val lines = lineStarts.mapIndexed { index, start ->
            val end = if (index < lineStarts.lastIndex) lineStarts[index + 1] - 1 else text.length
            val contexts = lineContexts[index]
            val innermostItem = contexts.lastOrNull { it is Context.ListItem } as Context.ListItem?
            DocumentLine(
                start = start,
                end = end,
                indents = contexts.toIndents(),
                marker = innermostItem?.takeIf { it.firstLine == index }?.marker,
                isCode = contexts.any { it is Context.CodeBlock },
            )
        }
        return RichTextDocument(
            text = text.toString(),
            lines = lines,
            blocks = blocks.toList(),
            inlineRanges = inlineRanges.toList(),
            mentions = mentions.toList(),
        )
    }

    private fun parseChildren(element: Element) {
        var seenBlock = false
        for (child in element.childNodes()) {
            when (child) {
                is TextNode -> appendText(child.wholeText)
                is Element -> {
                    if (child.isBlockElement()) {
                        if (seenBlock) appendLineBreak()
                        seenBlock = true
                    }
                    parseElement(child)
                }
            }
        }
    }

    private fun parseElement(element: Element) {
        when (val tag = element.tagName()) {
            "b", "strong" -> inline(InlineFormatting.Bold) { parseChildren(element) }
            "i", "em" -> inline(InlineFormatting.Italic) { parseChildren(element) }
            "u" -> inline(InlineFormatting.Underline) { parseChildren(element) }
            "del", "s", "strike" -> inline(InlineFormatting.StrikeThrough) { parseChildren(element) }
            "code" -> if (codeState != null) {
                parseChildren(element)
            } else {
                inline(InlineFormatting.InlineCode) { parseChildren(element) }
            }
            "a" -> if (element.hasAttr("data-mention-type")) {
                appendMention(element)
            } else {
                inline(InlineFormatting.Link(element.attr("href"))) { parseChildren(element) }
            }
            "br" -> appendLineBreak()
            "ul", "ol" -> inContext(Context.ListContainer()) { parseChildren(element) }
            "li" -> inContext(Context.ListItem(listMarkerFor(element), currentLine)) { parseChildren(element) }
            "blockquote" -> inBlock(BlockKind.Quote, Context.Quote()) { parseChildren(element) }
            "pre" -> inBlock(BlockKind.CodeBlock, Context.CodeBlock()) {
                codeState = CodeState(element.codeBlockPlaceholderOffsets())
                parseChildren(element)
                codeState = null
            }
            "p" -> inContext(Context.Paragraph()) {
                if (element.isAmbiguousPlaceholderParagraph()) {
                    if (element in spaceParagraphs) appendText(" ")
                } else {
                    parseChildren(element)
                }
            }
            "h1", "h2", "h3", "h4", "h5", "h6" -> inContext(Context.Paragraph()) {
                inline(InlineFormatting.Heading(tag.last().digitToInt())) { parseChildren(element) }
            }
            "summary" -> inContext(Context.Paragraph()) {
                inline(InlineFormatting.Bold) { parseChildren(element) }
            }
            "div", "details" -> inContext(Context.Paragraph()) { parseChildren(element) }
            else -> parseChildren(element)
        }
    }

    private fun listMarkerFor(element: Element): ListMarker {
        val parent = element.parent()
        val depth = stack.count { it is Context.ListContainer } - 1
        return if (parent?.tagName() == "ol") {
            val index = parent.children().filter { it.tagName() == "li" }.indexOf(element)
            val start = parent.attr("start").toIntOrNull() ?: 1
            ListMarker.Number(start + index.coerceAtLeast(0))
        } else {
            ListMarker.Bullet(depth.coerceAtLeast(0))
        }
    }

    private fun appendText(value: String) {
        val code = codeState
        for (char in value) {
            if (code != null) {
                val isPlaceholder = code.position in code.placeholderOffsets
                code.position++
                if (isPlaceholder) continue
            }
            when {
                char == '\n' -> appendLineBreak()
                char == NBSP && code == null -> text.append(' ')
                else -> text.append(char)
            }
        }
    }

    private fun appendMention(element: Element) {
        mentions += MentionRange(
            offset = text.length,
            url = element.attr("href"),
            text = element.text(),
            isAtRoom = element.attr("data-mention-type") == "at-room",
        )
        text.append(MENTION_PLACEHOLDER)
    }

    private fun appendLineBreak() {
        text.append('\n')
        lineStarts += text.length
        lineContexts += stack.toList()
    }

    private inline fun inline(format: InlineFormatting, block: () -> Unit) {
        val start = text.length
        block()
        if (text.length > start) {
            inlineRanges += InlineRange(format, start, text.length)
        }
    }

    private inline fun inContext(context: Context, block: () -> Unit) {
        stack += context
        // While a line is empty, its context is the innermost one we've seen: this way the line
        // created by a separator right before a list item is also part of that list item.
        if (text.length == lineStarts.last()) {
            lineContexts[currentLine] = stack.toList()
        }
        block()
        stack.removeAt(stack.lastIndex)
    }

    private inline fun inBlock(kind: BlockKind, context: Context, block: () -> Unit) {
        val outerIndents = stack.toIndents()
        val firstLine = currentLine
        inContext(context, block)
        blocks += DocumentBlock(kind, outerIndents, firstLine, currentLine)
    }

    private fun List<Context>.toIndents(): List<IndentKind> = mapNotNull {
        when (it) {
            is Context.ListContainer -> IndentKind.List
            is Context.Quote -> IndentKind.Quote
            is Context.CodeBlock -> IndentKind.CodeBlock
            else -> null
        }
    }
}
