/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.internal.utils

import android.text.Spanned
import android.text.style.ReplacementSpan
import kotlin.math.max
import kotlin.math.min

internal object TextRangeHelper {
    /**
     * Return a new range that covers the given range and extends it to cover
     * any replacement spans at either end.
     *
     * The range is returned as a pair of integers where the first is less than the last
     */
    fun extendRangeToReplacementSpans(
        spanned: Spanned,
        start: Int,
        length: Int,
    ): Pair<Int, Int> {
        require(length > 0)
        val end = start + length
        val spans = spanned.getSpans(start, end, ReplacementSpan::class.java)
        val firstReplacementSpanStart = spans.minOfOrNull { spanned.getSpanStart(it) }
        val lastReplacementSpanEnd = spans.maxOfOrNull { spanned.getSpanEnd(it) }
        val newStart = min(start, firstReplacementSpanStart ?: end)
        val newEnd = max(end, lastReplacementSpanEnd ?: end)
        return newStart to newEnd
    }
}