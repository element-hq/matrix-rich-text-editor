/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next.internal.document

import org.jsoup.nodes.Element
import org.jsoup.nodes.TextNode
import timber.log.Timber

/**
 * Rust renders both an empty paragraph and a paragraph containing a single space as
 * `<p>&nbsp;</p>`, but the first one has a length of 0 and the second one a length of 1.
 *
 * To tell them apart, we emulate how Rust generates its plain text (`to_plain_text`) from the HTML
 * assuming every ambiguous paragraph is empty, keeping track of where each of those paragraphs
 * would be. Then we compare that with the actual plain text from Rust: if the character at that
 * position is a line break the paragraph was empty, otherwise it contained a space.
 */
internal object PlaceholderResolver {
    /**
     * Returns the ambiguous paragraphs in [body] which actually contain a space.
     */
    fun resolveSpaceParagraphs(body: Element, plainTextProvider: () -> String?): Set<Element> {
        val ambiguous = body.getElementsByTag("p").filter { it.isAmbiguousPlaceholderParagraph() }
        if (ambiguous.isEmpty()) return emptySet()

        val plainText = plainTextProvider() ?: return emptySet()
        val emulator = PlainTextEmulator()
        emulator.emulate(body)

        val result = mutableSetOf<Element>()
        var shift = 0
        for ((element, offset) in emulator.ambiguousOffsets) {
            val char = plainText.getOrNull(offset + shift)
            if (char != null && char != '\n') {
                result.add(element)
                shift++
            }
        }
        if (plainText.length != emulator.length + shift) {
            // Our emulation doesn't match Rust's, so we can't trust the results
            Timber.w("Could not resolve placeholder paragraphs, plain text length mismatch.")
            return emptySet()
        }
        return result
    }
}

/**
 * Mirrors the `ToPlainText` implementations of the Rust DOM nodes.
 */
private class PlainTextEmulator {
    private val builder = StringBuilder()
    val ambiguousOffsets = mutableListOf<Pair<Element, Int>>()
    val length get() = builder.length

    fun emulate(body: Element) = visitChildren(body)

    private fun visitChildren(element: Element) {
        for (node in element.childNodes()) {
            when (node) {
                is TextNode -> builder.append(node.wholeText)
                is Element -> visit(node)
            }
        }
    }

    private fun visit(element: Element) {
        val localStart = builder.length
        fun localEndsWithLineBreak() = builder.length > localStart && builder.last() == '\n'

        when (element.tagName()) {
            "a" -> if (element.hasAttr("data-mention-type")) {
                builder.append(element.text())
            } else {
                visitChildren(element)
            }
            "br" -> builder.append('\n')
            "ul", "ol" -> {
                element.children().forEachIndexed { index, child ->
                    if (index != 0 && !localEndsWithLineBreak()) builder.append('\n')
                    visit(child)
                }
                builder.append('\n')
            }
            "li" -> visitChildren(element)
            "pre" -> {
                val placeholders = element.codeBlockPlaceholderOffsets()
                element.wholeText().forEachIndexed { index, char ->
                    if (index !in placeholders) builder.append(char)
                }
                builder.append('\n')
            }
            else -> {
                if (element.isAmbiguousPlaceholderParagraph()) {
                    ambiguousOffsets += element to builder.length
                } else {
                    visitChildren(element)
                }
                if (element.isBlockElement() && !localEndsWithLineBreak()) {
                    builder.append('\n')
                }
            }
        }
    }
}
