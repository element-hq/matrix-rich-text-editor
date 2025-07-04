/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.display

import android.text.style.ReplacementSpan

/**
 * Different ways to display text
 */
sealed class TextDisplay {
    /**
     * Display the text using a custom span
     */
    data class Custom(val customSpan: ReplacementSpan): TextDisplay()

    /**
     * Display the text using a default pill shape
     */
    object Pill: TextDisplay()

    /**
     * Display the text without any replacement
     */
    object Plain: TextDisplay()
}

