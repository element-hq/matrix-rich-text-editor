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
import io.mockk.every
import io.mockk.mockk
import io.mockk.slot
import org.hamcrest.MatcherAssert.assertThat
import org.hamcrest.Matchers.equalTo
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class QuoteSpanTest {
    @Test
    fun testIndicatorIsDrawnAfterTheMarginInLeftToRightText() {
        val rect = drawLeadingMargin(x = 0, dir = 1)

        assertThat(rect.left, equalTo(MARGIN))
        assertThat(rect.right, equalTo(MARGIN + INDICATOR_WIDTH))
    }

    @Test
    fun testIndicatorIsDrawnBeforeTheMarginInRightToLeftText() {
        val rect = drawLeadingMargin(x = CANVAS_WIDTH, dir = -1)

        assertThat(rect.right, equalTo(CANVAS_WIDTH - MARGIN))
        assertThat(rect.left, equalTo(CANVAS_WIDTH - MARGIN - INDICATOR_WIDTH))
    }

    @Test
    fun testIndicatorHasTheConfiguredWidthInBothDirections() {
        assertThat(drawLeadingMargin(x = 0, dir = 1).width(), equalTo(INDICATOR_WIDTH))
        assertThat(drawLeadingMargin(x = CANVAS_WIDTH, dir = -1).width(), equalTo(INDICATOR_WIDTH))
    }

    private fun drawLeadingMargin(x: Int, dir: Int): Rect {
        val span = QuoteSpan(
            indicatorColor = 0xC0A0A0A0.toInt(),
            indicatorWidth = INDICATOR_WIDTH,
            indicatorPadding = INDICATOR_PADDING,
            margin = MARGIN,
        )
        val drawnRect = slot<Rect>()
        val canvas = mockk<Canvas>(relaxed = true) {
            every { width } returns CANVAS_WIDTH
            every { drawRect(capture(drawnRect), any()) } returns Unit
        }

        span.drawLeadingMargin(
            canvas, Paint(), x, dir, TOP, BASELINE, BOTTOM, "a quote", 0, 7, true, null
        )

        return drawnRect.captured
    }

    companion object {
        private const val CANVAS_WIDTH = 1000
        private const val MARGIN = 10
        private const val INDICATOR_WIDTH = 4
        private const val INDICATOR_PADDING = 6
        private const val TOP = 0
        private const val BASELINE = 10
        private const val BOTTOM = 20
    }
}
