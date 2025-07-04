/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.internal.viewmodel

import android.text.Spanned
import android.widget.EditText
import uniffi.wysiwyg_composer.TextUpdate.ReplaceAll
import uniffi.wysiwyg_composer.TextUpdate.Select

/**
 * Result of a composer operation to be applied to the [EditText].
 */
internal sealed interface ComposerResult {
    /**
     * Mapped model of [ReplaceAll] from the Rust code to be applied to the [EditText].
     */
    data class ReplaceText(
        /** Text in [Spanned] format after being parsed from HTML. */
        val text: CharSequence,
        /** Selection to apply to the editor. */
        val selection: IntRange,
    ) : ComposerResult

    /**
     * Mapped model of [Select] from the Rust code to be applied to the [EditText].
     */
    data class SelectionUpdated(
        /** Selection to apply to the editor. */
        val selection: IntRange,
    ) : ComposerResult
}
