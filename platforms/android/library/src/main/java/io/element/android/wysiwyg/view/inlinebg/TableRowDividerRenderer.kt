/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.view.inlinebg

import android.graphics.Canvas
import android.graphics.Paint
import android.text.Layout
import android.text.Spanned
import androidx.annotation.ColorInt
import androidx.annotation.Px
import io.element.android.wysiwyg.view.spans.TableRowSpan
import io.element.android.wysiwyg.view.spans.TableSpan

/**
 * Draws a horizontal divider line between consecutive rows of each table (skipping the line
 * after the last row of each table). Not routed through [SpanBackgroundHelper]/[BlockRenderer] -
 * those work per independent same-type span, but this needs the parent/child relationship
 * between a [TableSpan] and its nested [TableRowSpan]s to know which row is last.
 */
internal class TableRowDividerRenderer(
    @Px private val rowDividerWidth: Int,
    @ColorInt private val rowDividerColor: Int,
) {
    private val paint = Paint().apply {
        color = rowDividerColor
        strokeWidth = rowDividerWidth.toFloat()
    }

    fun draw(canvas: Canvas, text: Spanned, layout: Layout) {
        val tableSpans = text.getSpans(0, text.length, TableSpan::class.java)
        for (tableSpan in tableSpans) {
            val tableStart = text.getSpanStart(tableSpan)
            val tableEnd = text.getSpanEnd(tableSpan)
            val rowSpans = text.getSpans(tableStart, tableEnd, TableRowSpan::class.java)
                .filter { text.getSpanStart(it) >= tableStart && text.getSpanEnd(it) <= tableEnd }
                .sortedBy { text.getSpanStart(it) }
            // Draw a line after every row except the last one.
            for (rowSpan in rowSpans.dropLast(1)) {
                val rowEnd = text.getSpanEnd(rowSpan)
                val line = layout.getLineForOffset(rowEnd).coerceIn(0, layout.lineCount - 1)
                val y = layout.getLineBottom(line).toFloat()
                canvas.drawLine(0f, y, layout.width.toFloat(), y, paint)
            }
        }
    }
}
