/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next.internal.document

import androidx.compose.runtime.Immutable

/**
 * Semantic representation of the Rust composer's content.
 *
 * [text] is the *logical* text: every index in it is exactly the UTF-16 code unit index the Rust
 * `ComposerModel` uses, so it can be stored as is in the `TextFieldState` and no index mapping
 * is needed when talking to the composer. This means:
 *
 * - Sibling block nodes are separated by a single `\n`.
 * - A mention is a single [MENTION_PLACEHOLDER] character.
 * - The `&nbsp;` placeholders Rust adds to empty blocks are not part of the text.
 * - Non-breaking spaces outside code blocks are normalised to plain spaces, as that's what the
 *   user typed (Rust only emits them to keep the HTML rendering consistent).
 *
 * Anything that only exists for display purposes (list bullets, mention display names, etc.) is
 * described by the other properties and applied later as an `OutputTransformation`.
 */
@Immutable
internal data class RichTextDocument(
    val text: String,
    val lines: List<DocumentLine>,
    val blocks: List<DocumentBlock>,
    val inlineRanges: List<InlineRange>,
    val mentions: List<MentionRange>,
) {
    companion object {
        val Empty = RichTextDocument(
            text = "",
            lines = listOf(DocumentLine(start = 0, end = 0, indents = emptyList(), marker = null, isCode = false)),
            blocks = emptyList(),
            inlineRanges = emptyList(),
            mentions = emptyList(),
        )
    }
}

/**
 * The character used in the logical text to represent a mention, which Rust counts as a single
 * character.
 */
internal const val MENTION_PLACEHOLDER = '￼'

internal const val NBSP = ' '

/**
 * A line of the logical text, delimited by `\n` characters, which are not included in it.
 */
@Immutable
internal data class DocumentLine(
    val start: Int,
    val end: Int,
    /** The nested containers affecting the indentation of this line, outermost first. */
    val indents: List<IndentKind>,
    /** A list item marker to draw at the start of the line, if it's the first line of an item. */
    val marker: ListMarker?,
    val isCode: Boolean,
)

internal enum class IndentKind { List, Quote, CodeBlock }

@Immutable
internal sealed interface ListMarker {
    data class Bullet(val depth: Int) : ListMarker
    data class Number(val value: Int) : ListMarker
}

/**
 * A block that needs some decoration drawn for its lines, like a quote bar or a code background.
 */
@Immutable
internal data class DocumentBlock(
    val kind: BlockKind,
    /** The indentation of the containers wrapping this block. */
    val outerIndents: List<IndentKind>,
    val firstLine: Int,
    val lastLine: Int,
)

internal enum class BlockKind { Quote, CodeBlock }

@Immutable
internal sealed interface InlineFormatting {
    data object Bold : InlineFormatting
    data object Italic : InlineFormatting
    data object Underline : InlineFormatting
    data object StrikeThrough : InlineFormatting
    data object InlineCode : InlineFormatting
    data class Link(val url: String) : InlineFormatting
    data class Heading(val level: Int) : InlineFormatting
}

@Immutable
internal data class InlineRange(
    val format: InlineFormatting,
    val start: Int,
    val end: Int,
)

@Immutable
internal data class MentionRange(
    /** The position of the [MENTION_PLACEHOLDER] in the logical text. */
    val offset: Int,
    val url: String,
    val text: String,
    val isAtRoom: Boolean,
)
