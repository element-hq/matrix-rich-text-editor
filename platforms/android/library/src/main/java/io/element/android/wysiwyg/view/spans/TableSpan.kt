/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.view.spans

import android.graphics.Canvas
import android.graphics.Paint
import android.text.Layout
import android.text.style.LeadingMarginSpan
import androidx.annotation.Px

/**
 * Table (<table> in HTML) Span marking the whole extent of a table, so [BlockSpan]-aware
 * renderers can draw a bordered box around it.
 */
class TableSpan(
    @Px private val leadingMargin: Int,
) : BlockSpan, LeadingMarginSpan {

    override fun getLeadingMargin(first: Boolean): Int {
        return leadingMargin
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
    ) = Unit
}
