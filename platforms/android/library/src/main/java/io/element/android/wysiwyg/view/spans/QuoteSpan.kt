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
import android.os.Parcel
import android.text.Layout
import android.text.TextPaint
import android.text.style.LeadingMarginSpan
import android.text.style.MetricAffectingSpan

/**
 * Quote ("> a quote" in Markdown, <blockquote> in HTML) Span that applies margin and an indicator
 * on the start of the paragraph.
 */
class QuoteSpan : MetricAffectingSpan, LeadingMarginSpan {

    private val indicatorColor: Int
    private val indicatorWidth: Int
    private val indicatorPadding: Int
    private val margin: Int

    private val paint = Paint()
    private var rect = Rect()

    constructor(
        indicatorColor: Int,
        indicatorWidth: Int,
        indicatorPadding: Int,
        margin: Int
    ): super() {
        this.margin = margin
        this.indicatorWidth = indicatorWidth
        this.indicatorPadding = indicatorPadding
        this.indicatorColor = indicatorColor
    }

    constructor(parcel: Parcel): super() {
        indicatorColor = parcel.readInt()
        indicatorWidth = parcel.readInt()
        indicatorPadding = parcel.readInt()
        margin = parcel.readInt()
    }

    override fun updateDrawState(tp: TextPaint) {}

    override fun updateMeasureState(textPaint: TextPaint) {}

    override fun getLeadingMargin(first: Boolean): Int {
        return margin + indicatorWidth + indicatorPadding
    }

    override fun drawLeadingMargin(
        c: Canvas,
        p: Paint,
        x: Int,
        dir: Int,
        top: Int,
        baseline: Int,
        bottom: Int,
        text: CharSequence?,
        start: Int,
        end: Int,
        first: Boolean,
        layout: Layout?
    ) {
        paint.style = Paint.Style.FILL
        paint.color = indicatorColor

        val left: Int
        val right: Int

        if (dir > 0) {
            // Left to right: `x` is the leading (left) edge of the line
            left = x + margin
            right = left + indicatorWidth
        } else {
            // Right to left: `x` is the leading (right) edge of the line, so the margin and the
            // indicator are laid out towards the left instead
            right = x - margin
            left = right - indicatorWidth
        }

        rect.set(left, top, right, bottom)
        c.drawRect(rect, paint)
    }
}
