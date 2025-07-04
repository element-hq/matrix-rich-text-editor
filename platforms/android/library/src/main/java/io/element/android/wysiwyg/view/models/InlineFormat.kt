/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.view.models

import androidx.compose.runtime.Immutable
import uniffi.wysiwyg_composer.ComposerAction

/**
 * Mapping of [ComposerAction] inline format actions. These are text styles that can be applied to
 * a text selection in the editor.
 */
@Immutable
sealed interface InlineFormat {
    data object Bold: InlineFormat
    data object Italic: InlineFormat
    data object Underline: InlineFormat
    data object StrikeThrough: InlineFormat
    data object InlineCode: InlineFormat
}
