/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.utils

import io.element.android.wysiwyg.BuildConfig
import timber.log.Timber
import uniffi.wysiwyg_composer.Disposable

internal class RustCleanerTask(
    private val disposable: Disposable,
) : Runnable {
    override fun run() {
        if (BuildConfig.DEBUG) {
            Timber.d("Cleaning up disposable: $disposable")
        }
        disposable.destroy()
    }
}