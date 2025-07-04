/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.extensions

import io.element.android.wysiwyg.BuildConfig
import timber.log.Timber
import uniffi.wysiwyg_composer.ComposerModelInterface
import uniffi.wysiwyg_composer.ComposerState

val LOG_ENABLED = BuildConfig.DEBUG

/**
 * Get the current HTML representation of the formatted text in the Rust code, along with its
 * selection.
 */
fun ComposerState.dump() = "'${html.string()}' | Start: $start | End: $end"

/**
 * Log the current state of the editor in the Rust code.
 */
fun ComposerModelInterface.log() = if (LOG_ENABLED) {
    Timber.d(toExampleFormat())
} else Unit
