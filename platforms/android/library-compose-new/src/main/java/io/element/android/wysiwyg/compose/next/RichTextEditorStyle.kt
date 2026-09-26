/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp

/**
 * Defines how the contents of a [RichTextEditor] are displayed.
 */
@Immutable
data class RichTextEditorStyle(
    val textStyle: TextStyle,
    val cursorBrush: Brush,
    val placeholderColor: Color,
    val link: SpanStyle,
    val inlineCode: InlineCodeStyle,
    val codeBlock: CodeBlockStyle,
    val quote: QuoteStyle,
    val list: ListStyle,
    val mention: MentionStyle,
)

@Immutable
data class InlineCodeStyle(
    val spanStyle: SpanStyle,
    val background: Color,
    val cornerRadius: Dp,
    val horizontalPadding: Dp,
)

@Immutable
data class CodeBlockStyle(
    val spanStyle: SpanStyle,
    val background: Color,
    val cornerRadius: Dp,
    /** Space between the start of the code block background and its text. */
    val padding: Dp,
)

@Immutable
data class QuoteStyle(
    val barColor: Color,
    val barWidth: Dp,
    /** Indentation of the quote text, including the bar. */
    val indent: Dp,
)

@Immutable
data class ListStyle(
    /** Indentation added by each list level, the list marker is drawn inside it. */
    val indent: Dp,
    /** Space between the list marker and the text of the list item. */
    val markerGap: Dp,
    val bulletRadius: Dp,
    val markerColor: Color,
)

@Immutable
data class MentionStyle(
    val spanStyle: SpanStyle,
    val pillBackground: Color,
    val cornerRadius: Dp,
    val horizontalPadding: Dp,
)

object RichTextEditorDefaults {
    @Composable
    fun style(
        textStyle: TextStyle = defaultTextStyle(),
        cursorBrush: Brush = SolidColor(textStyle.color),
        placeholderColor: Color = textStyle.color.copy(alpha = 0.5f),
        link: SpanStyle = SpanStyle(color = linkColor(), textDecoration = TextDecoration.Underline),
        inlineCode: InlineCodeStyle = inlineCodeStyle(),
        codeBlock: CodeBlockStyle = codeBlockStyle(),
        quote: QuoteStyle = quoteStyle(),
        list: ListStyle = listStyle(textStyle),
        mention: MentionStyle = mentionStyle(),
    ) = RichTextEditorStyle(
        textStyle = textStyle,
        cursorBrush = cursorBrush,
        placeholderColor = placeholderColor,
        link = link,
        inlineCode = inlineCode,
        codeBlock = codeBlock,
        quote = quote,
        list = list,
        mention = mention,
    )

    @Composable
    fun defaultTextStyle() = TextStyle(
        fontSize = 16.sp,
        color = if (isSystemInDarkTheme()) Color(0xFFEBEEF2) else Color(0xFF1B1D22),
    )

    @Composable
    fun inlineCodeStyle(
        spanStyle: SpanStyle = SpanStyle(fontFamily = FontFamily.Monospace, fontSize = 0.9.em),
        background: Color = subtleBackground(),
        cornerRadius: Dp = 4.dp,
        horizontalPadding: Dp = 2.dp,
    ) = InlineCodeStyle(spanStyle, background, cornerRadius, horizontalPadding)

    @Composable
    fun codeBlockStyle(
        spanStyle: SpanStyle = SpanStyle(fontFamily = FontFamily.Monospace, fontSize = 0.9.em),
        background: Color = subtleBackground(),
        cornerRadius: Dp = 6.dp,
        padding: Dp = 8.dp,
    ) = CodeBlockStyle(spanStyle, background, cornerRadius, padding)

    @Composable
    fun quoteStyle(
        barColor: Color = if (isSystemInDarkTheme()) Color(0xFF656D77) else Color(0xFFC1C6CD),
        barWidth: Dp = 3.dp,
        indent: Dp = 14.dp,
    ) = QuoteStyle(barColor, barWidth, indent)

    @Composable
    fun listStyle(
        textStyle: TextStyle = defaultTextStyle(),
        indent: Dp = 28.dp,
        markerGap: Dp = 6.dp,
        bulletRadius: Dp = 3.dp,
        markerColor: Color = textStyle.color,
    ) = ListStyle(indent, markerGap, bulletRadius, markerColor)

    @Composable
    fun mentionStyle(
        spanStyle: SpanStyle = SpanStyle(color = linkColor()),
        pillBackground: Color = if (isSystemInDarkTheme()) Color(0xFF26313D) else Color(0xFFE3F0FF),
        cornerRadius: Dp = 12.dp,
        horizontalPadding: Dp = 4.dp,
    ) = MentionStyle(spanStyle, pillBackground, cornerRadius, horizontalPadding)

    @Composable
    private fun linkColor() = if (isSystemInDarkTheme()) Color(0xFF7CB2FF) else Color(0xFF0467DD)

    @Composable
    private fun subtleBackground() = if (isSystemInDarkTheme()) Color(0xFF26282D) else Color(0xFFF0F2F5)
}
