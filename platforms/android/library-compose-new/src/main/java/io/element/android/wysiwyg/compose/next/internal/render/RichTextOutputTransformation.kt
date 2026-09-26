/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next.internal.render

import androidx.compose.foundation.text.input.OutputTransformation
import androidx.compose.foundation.text.input.TextFieldBuffer
import io.element.android.wysiwyg.compose.next.internal.isLooselyEqualTo

/**
 * Transforms the logical text in the text field into the visual one using the current
 * [RenderPlan]. Since [plan] reads snapshot state, the transformation will run again when it
 * changes.
 */
internal class RichTextOutputTransformation(
    private val plan: () -> RenderPlan,
) : OutputTransformation {
    override fun TextFieldBuffer.transformOutput() {
        val plan = plan()
        // The plan may be outdated for a moment if the text field state was modified outside of
        // the editor. Displaying the text without styles is better than displaying wrong ones.
        if (!asCharSequence().isLooselyEqualTo(plan.logicalText)) return

        for (edit in plan.edits.asReversed()) {
            replace(edit.start, edit.end, edit.replacement)
        }
        for (range in plan.paragraphStyles) {
            addStyle(range.item, range.start, range.end)
        }
        for (range in plan.spanStyles) {
            addStyle(range.item, range.start, range.end)
        }
    }
}
