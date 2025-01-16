/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.view.inlinebg

import android.graphics.Canvas
import android.graphics.drawable.Drawable
import android.text.Layout
import android.text.Spanned

/**
 * Helper class to render a single 'block' with a bordered round rectangle as its background.
 */
internal class BlockRenderer(
    private val drawable: Drawable,
    horizontalPadding: Int,
    verticalPadding: Int,
): SpanBackgroundRenderer(horizontalPadding, verticalPadding) {

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
        val actualStartLine = startLine.coerceIn(0, layout.lineCount - 1)
        val actualEndLine = endLine.coerceIn(0, layout.lineCount - 1)
        val top = layout.getLineTop(actualStartLine)
        val bottom = layout.getLineBottom(actualEndLine)
        drawable.setBounds(
            if (leadingMargin > 0) leadingMargin - horizontalPadding else horizontalPadding,
            top + verticalPadding,
            layout.width - horizontalPadding * 2,
            bottom - verticalPadding * 2
        )
        drawable.draw(canvas)
    }

}
