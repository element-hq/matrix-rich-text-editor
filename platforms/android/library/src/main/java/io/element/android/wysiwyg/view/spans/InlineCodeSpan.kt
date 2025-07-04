/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.view.spans

import android.text.TextPaint
import android.text.style.RelativeSizeSpan
import android.text.style.TypefaceSpan
import androidx.annotation.FloatRange

/**
 * Inline code (`some code` in Markdown, <code> in HTML) Span that applies a monospaced font style.
 *
 * Note that this span does not apply a background style; it simply tells the TextView where to
 * apply an inline background.
 *
 * To display this span, either:
 * - use [io.element.android.wysiwyg.EditorStyledTextView], or
 * - add [io.element.android.wysiwyg.inlinebg.SpanBackgroundRenderer] to your TextView, using
 *   [io.element.android.wysiwyg.inlinebg.InlineBgHelper] as a reference
 *
 * See [io.element.android.wysiwyg.inlinebg.SpanBackgroundRenderer], based on the official Google sample:
 * - https://medium.com/androiddevelopers/drawing-a-rounded-corner-background-on-text-5a610a95af5
 * - https://github.com/googlearchive/android-text/tree/996fdb65bbfbb786c3ca4e4e40b30509067201fc/RoundedBackground-Kotlin
 */
class InlineCodeSpan(
    @FloatRange(from = 0.0) relativeSizeProportion: Float =
        CodeSpanConstants.DEFAULT_RELATIVE_SIZE_PROPORTION,
) : TypefaceSpan("monospace"), PlainAtRoomMentionDisplaySpan {
    private val relativeSizeSpan = RelativeSizeSpan(relativeSizeProportion)

    override fun updateMeasureState(paint: TextPaint) {
        super.updateMeasureState(paint)
        relativeSizeSpan.updateMeasureState(paint)
    }

    override fun updateDrawState(ds: TextPaint) {
        super.updateDrawState(ds)
        relativeSizeSpan.updateMeasureState(ds)
    }
}