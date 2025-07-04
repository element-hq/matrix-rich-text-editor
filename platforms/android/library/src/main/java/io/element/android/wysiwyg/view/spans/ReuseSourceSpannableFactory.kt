/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.view.spans

import android.text.Spannable
import android.text.SpannableStringBuilder

/**
 * This factory is used to reuse the current source if possible to improve performance.
 */
internal class ReuseSourceSpannableFactory : Spannable.Factory() {
    override fun newSpannable(source: CharSequence?): Spannable {
        // Try to reuse current source if possible to improve performance
        return source as? Spannable ?: SpannableStringBuilder(source)
    }
}