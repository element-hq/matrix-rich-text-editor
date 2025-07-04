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
import android.text.style.ReplacementSpan

/**
 * Wrapper for a [ReplacementSpan] which does nothing except delegate to an
 * underlying span.
 * It is used to allow reuse of the same underlying span across multiple ranges
 * of a spanned text.
 */
class CustomMentionSpan(
    val providedSpan: ReplacementSpan,
    val url: String? = null,
) : ReplacementSpan() {
    override fun draw(
        canvas: Canvas,
        text: CharSequence?,
        start: Int,
        end: Int,
        x: Float,
        top: Int,
        y: Int,
        bottom: Int,
        paint: Paint
    ) = providedSpan.draw(
        canvas, text, start, end, x, top, y, bottom, paint
    )

    override fun getSize(
        paint: Paint,
        text: CharSequence?,
        start: Int,
        end: Int,
        fm: Paint.FontMetricsInt?
    ): Int = providedSpan.getSize(
        paint, text, start, end, fm
    )
}
