/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next.internal.render

import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.text.TextLayoutResult
import androidx.compose.ui.text.TextMeasurer
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.drawText
import io.element.android.wysiwyg.compose.next.internal.document.ListMarker

/**
 * Draws the [RenderPlan.decorations] that can't be expressed as text styles, using the [layout]
 * of the visual text.
 */
internal fun DrawScope.drawDecorations(
    plan: RenderPlan,
    layout: TextLayoutResult,
    textMeasurer: TextMeasurer,
    markerTextStyle: TextStyle,
    bulletRadius: Float,
) {
    // The layout may belong to a previous version of the text
    if (layout.layoutInput.text.length != plan.visualLength) return

    for (decoration in plan.decorations) {
        when (decoration) {
            is Decoration.BlockBackground -> {
                val (top, bottom) = layout.verticalBounds(decoration.start, decoration.end)
                drawRoundRect(
                    color = decoration.color,
                    topLeft = Offset(decoration.x, top),
                    size = Size(size.width - decoration.x, bottom - top),
                    cornerRadius = CornerRadius(decoration.cornerRadius),
                )
            }
            is Decoration.Bar -> {
                val (top, bottom) = layout.verticalBounds(decoration.start, decoration.end)
                drawRoundRect(
                    color = decoration.color,
                    topLeft = Offset(decoration.x, top),
                    size = Size(decoration.width, bottom - top),
                    cornerRadius = CornerRadius(decoration.width / 2),
                )
            }
            is Decoration.InlineBackground -> drawInlineBackground(decoration, layout)
            is Decoration.Marker -> {
                val line = layout.getLineForOffset(decoration.offset)
                when (val marker = decoration.marker) {
                    is ListMarker.Bullet -> {
                        val center = Offset(
                            x = decoration.endX - bulletRadius,
                            y = (layout.getLineTop(line) + layout.getLineBottom(line)) / 2,
                        )
                        if (marker.depth % 2 == 0) {
                            drawCircle(decoration.color, bulletRadius, center)
                        } else {
                            drawCircle(decoration.color, bulletRadius, center, style = Stroke(bulletRadius / 2))
                        }
                    }
                    is ListMarker.Number -> {
                        val markerLayout = textMeasurer.measure(
                            text = "${marker.value}.",
                            style = markerTextStyle.copy(color = decoration.color),
                        )
                        drawText(
                            textLayoutResult = markerLayout,
                            topLeft = Offset(
                                x = decoration.endX - markerLayout.size.width,
                                y = layout.getLineBaseline(line) - markerLayout.firstBaseline,
                            ),
                        )
                    }
                }
            }
        }
    }
}

private fun DrawScope.drawInlineBackground(decoration: Decoration.InlineBackground, layout: TextLayoutResult) {
    if (decoration.end <= decoration.start) return
    val firstLine = layout.getLineForOffset(decoration.start)
    val lastLine = layout.getLineForOffset(decoration.end - 1)
    for (line in firstLine..lastLine) {
        val start = maxOf(decoration.start, layout.getLineStart(line))
        val end = minOf(decoration.end, layout.getLineEnd(line, visibleEnd = true))
        if (end <= start) continue
        val left = layout.getBoundingBox(start).left - decoration.horizontalPadding
        val right = layout.getBoundingBox(end - 1).right + decoration.horizontalPadding
        val top = layout.getLineTop(line)
        val bottom = layout.getLineBottom(line)
        drawRoundRect(
            color = decoration.color,
            topLeft = Offset(left, top),
            size = Size(right - left, bottom - top),
            cornerRadius = CornerRadius(minOf(decoration.cornerRadius, (bottom - top) / 2)),
        )
    }
}

private fun TextLayoutResult.verticalBounds(start: Int, end: Int): Pair<Float, Float> {
    val firstLine = getLineForOffset(start)
    val lastLine = getLineForOffset((end - 1).coerceAtLeast(start))
    return getLineTop(firstLine) to getLineBottom(lastLine)
}
