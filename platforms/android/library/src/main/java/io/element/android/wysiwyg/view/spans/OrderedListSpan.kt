/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.view.spans

import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.Rect
import android.graphics.Typeface
import android.text.Layout
import android.text.Spanned
import android.text.style.LeadingMarginSpan

/**
 * Custom ordered list span (<ol> in HTML), prints a prefix with the current list item number and
 * applies a margin to the item.
 */
class OrderedListSpan(
    typeface: Typeface,
    textSize: Float,
    val order: Int,
    private val gapWidth: Int,
) : LeadingMarginSpan, BlockSpan {

    private val prefix = "$order."
    private val prefixToMeasure = "99."

    private val typefacePaint = Paint().apply {
        this.textSize = textSize
        this.typeface = typeface
        style = Paint.Style.STROKE
    }

    override fun getLeadingMargin(first: Boolean): Int {
        val bounds = Rect()
        typefacePaint.getTextBounds(prefixToMeasure, 0, prefixToMeasure.length, bounds)
        return bounds.width() + gapWidth
    }

    override fun drawLeadingMargin(
        c: Canvas,
        p: Paint,
        x: Int,
        dir: Int,
        top: Int,
        baseline: Int,
        bottom: Int,
        text: CharSequence,
        start: Int,
        end: Int,
        first: Boolean,
        layout: Layout?
    ) {
        val spanStart = (text as? Spanned)?.getSpanStart(this) ?: return
        val startLine = layout?.getLineForOffset(spanStart) ?: 0
        val currentLine = layout?.getLineForOffset(start) ?: 0
        if (currentLine - startLine > 0) return
        val bounds = Rect()
        p.getTextBounds(prefix, 0, prefix.length, bounds)
        val xEnd = x + getLeadingMargin(true) - gapWidth - bounds.width()
        c.drawText(prefix, 0, prefix.length, xEnd.toFloat(), baseline.toFloat(), p)
    }
}
