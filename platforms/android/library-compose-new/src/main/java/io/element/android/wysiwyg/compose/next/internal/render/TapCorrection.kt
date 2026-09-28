/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next.internal.render

import androidx.compose.ui.text.TextLayoutResult

/**
 * Where a paragraph ends, the end of its last line and the start of the next paragraph are the
 * same visual offset, so tapping after the end of that line places the cursor at the start of
 * the next line instead. The vertical position of the tap tells both cases apart.
 *
 * @param cursor the logical offset where the text field placed the cursor after the tap.
 * @param tapY the vertical position of the tap in the text layout coordinates.
 * @return the logical offset the cursor should be moved to, or null if it's already correct.
 */
internal fun RenderPlan.correctTappedCursor(cursor: Int, tapY: Float, layout: TextLayoutResult): Int? {
    if (cursor == 0 || (cursor - 1) !in paragraphBreaks) return null
    // The layout may belong to a previous version of the text
    if (layout.layoutInput.text.length != visualLength) return null
    val cursorLine = layout.getLineForOffset(mapper.mapStart(cursor))
    val tappedLine = layout.getLineForVerticalPosition(tapY)
    return if (tappedLine < cursorLine) cursor - 1 else null
}
