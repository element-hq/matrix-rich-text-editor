/*
 * Copyright 2022-2024 New Vector Ltd.
 * Copyright 2018 The Android Open Source Project
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
*/
package io.element.android.wysiwyg.view.inlinebg

import android.graphics.Canvas
import android.graphics.drawable.Drawable
import android.text.Layout
import android.text.Spanned
import io.element.android.wysiwyg.internal.view.getLineBottomWithoutPadding
import io.element.android.wysiwyg.internal.view.getLineTopWithoutPadding
import kotlin.math.max
import kotlin.math.min

/**
 * Base class for single and multi line background renderers.
 *
 * @param horizontalPadding the padding to be applied to left & right of the background
 * @param verticalPadding the padding to be applied to top & bottom of the background
 */
internal abstract class SpanBackgroundRenderer(
        val horizontalPadding: Int,
        val verticalPadding: Int
) {

    /**
     * Draw the background that starts at the {@code startOffset} and ends at {@code endOffset}.
     *
     * @param canvas Canvas to draw onto
     * @param layout Layout that contains the text
     * @param startLine the start line for the background
     * @param endLine the end line for the background
     * @param startOffset the character offset that the background should start at
     * @param endOffset the character offset that the background should end at
     */
    abstract fun draw(
        canvas: Canvas,
        layout: Layout,
        startLine: Int,
        endLine: Int,
        startOffset: Int,
        endOffset: Int,
        leadingMargin: Int,
        text: Spanned,
        spanType: Class<*>,
    )

    /**
     * Get the top offset of the line and add padding into account so that there is a gap between
     * top of the background and top of the text.
     *
     * @param layout Layout object that contains the text
     * @param line line number
     */
    protected fun getLineTop(layout: Layout, line: Int): Int {
        return layout.getLineTopWithoutPadding(line) - verticalPadding
    }

    /**
     * Get the bottom offset of the line and add padding into account so that there is a gap between
     * bottom of the background and bottom of the text.
     *
     * @param layout Layout object that contains the text
     * @param line line number
     */
    protected fun getLineBottom(layout: Layout, line: Int): Int {
        return layout.getLineBottomWithoutPadding(line) + verticalPadding
    }
}

/**
 * Draws the background for text that starts and ends on the same line.
 *
 * @param horizontalPadding the padding to be applied to left & right of the background
 * @param verticalPadding the padding to be applied to top & bottom of the background
 * @param drawable the drawable used to draw the background
 */
internal class SingleLineRenderer(
    horizontalPadding: Int,
    verticalPadding: Int,
    val drawable: Drawable
) : SpanBackgroundRenderer(horizontalPadding, verticalPadding) {

    override fun draw(
        canvas: Canvas,
        layout: Layout,
        startLine: Int,
        endLine: Int,
        startOffset: Int,
        endOffset: Int,
        leadingMargin: Int,
        text: Spanned,
        spanType: Class<*>,
    ) {
        val lineTop = getLineTop(layout, startLine)
        val lineBottom = getLineBottom(layout, startLine)
        // get min of start/end for left, and max of start/end for right since we don't
        // the language direction
        val left = min(startOffset, endOffset)
        val right = max(startOffset, endOffset)
        val width = canvas.width
        drawable.setBounds(max(left, 0), lineTop, min(right, width), lineBottom)
        drawable.draw(canvas)
    }
}

/**
 * Draws the background for text that starts and ends on different lines.
 *
 * @param horizontalPadding the padding to be applied to left & right of the background
 * @param verticalPadding the padding to be applied to top & bottom of the background
 * @param drawableLeft the drawable used to draw left edge of the background
 * @param drawableMid the drawable used to draw for whole line
 * @param drawableRight the drawable used to draw right edge of the background
 */
internal class MultiLineRenderer(
    horizontalPadding: Int,
    verticalPadding: Int,
    val drawableLeft: Drawable,
    val drawableMid: Drawable,
    val drawableRight: Drawable
) : SpanBackgroundRenderer(horizontalPadding, verticalPadding) {

    override fun draw(
        canvas: Canvas,
        layout: Layout,
        startLine: Int,
        endLine: Int,
        startOffset: Int,
        endOffset: Int,
        leadingMargin: Int,
        text: Spanned,
        spanType: Class<*>,
    ) {
        // draw the first line
        val paragDir = layout.getParagraphDirection(startLine)
        val lineEndOffset = if (paragDir == Layout.DIR_RIGHT_TO_LEFT) {
            layout.getLineLeft(startLine) - horizontalPadding
        } else {
            layout.getLineRight(startLine) + horizontalPadding
        }.toInt()

        var lineBottom = getLineBottom(layout, startLine)
        var lineTop = getLineTop(layout, startLine)
        drawStart(canvas, startOffset, lineTop, lineEndOffset, lineBottom)

        // for the lines in the middle draw the mid drawable
        for (line in startLine + 1 until endLine) {
            lineTop = getLineTop(layout, line)
            lineBottom = getLineBottom(layout, line)
            drawableMid.setBounds(
                (layout.getLineLeft(line).toInt() - horizontalPadding),
                lineTop,
                (layout.getLineRight(line).toInt() + horizontalPadding),
                lineBottom
            )
            drawableMid.draw(canvas)
        }

        val lineStartOffset = if (paragDir == Layout.DIR_RIGHT_TO_LEFT) {
            layout.getLineRight(startLine) + horizontalPadding
        } else {
            layout.getLineLeft(startLine) - horizontalPadding
        }.toInt()

        // draw the last line
        lineBottom = getLineBottom(layout, endLine)
        lineTop = getLineTop(layout, endLine)

        drawEnd(canvas, lineStartOffset, lineTop, endOffset, lineBottom)
    }

    /**
     * Draw the first line of a multiline annotation. Handles LTR/RTL.
     *
     * @param canvas Canvas to draw onto
     * @param start start coordinate for the background
     * @param top top coordinate for the background
     * @param end end coordinate for the background
     * @param bottom bottom coordinate for the background
     */
    private fun drawStart(canvas: Canvas, start: Int, top: Int, end: Int, bottom: Int) {
        val width = canvas.width
        if (start > end) {
            drawableRight.setBounds(max(end, 0), top, min(start, width), bottom)
            drawableRight.draw(canvas)
        } else {
            drawableLeft.setBounds(max(start, 0), top, min(end, width), bottom)
            drawableLeft.draw(canvas)
        }
    }

    /**
     * Draw the last line of a multiline annotation. Handles LTR/RTL.
     *
     * @param canvas Canvas to draw onto
     * @param start start coordinate for the background
     * @param top top position for the background
     * @param end end coordinate for the background
     * @param bottom bottom coordinate for the background
     */
    private fun drawEnd(canvas: Canvas, start: Int, top: Int, end: Int, bottom: Int) {
        val width = canvas.width
        if (start > end) {
            drawableLeft.setBounds(max(end, 0), top, min(start, width), bottom)
            drawableLeft.draw(canvas)
        } else {
            drawableRight.setBounds(max(start, 0), top, min(end, width), bottom)
            drawableRight.draw(canvas)
        }
    }
}
