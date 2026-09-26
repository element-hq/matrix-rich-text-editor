/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next.internal

import timber.log.Timber
import uniffi.wysiwyg_composer.ComposerModel
import uniffi.wysiwyg_composer.ComposerModelInterface
import uniffi.wysiwyg_composer.ComposerUpdate
import uniffi.wysiwyg_composer.InternalException

/**
 * Owns the Rust [ComposerModelInterface] and makes sure any error it throws is caught.
 *
 * If Rust panics the model can't be used anymore, so a new one is created and restored to the
 * last known content ([recoveryHtml]).
 */
internal class ComposerSession(
    private val factory: () -> ComposerModelInterface,
    private val onError: (Throwable) -> Unit,
) {
    private var composer: ComposerModelInterface? = create()

    /** The last known valid content, used to recover from a Rust panic. */
    var recoveryHtml: String = ""

    val isAvailable: Boolean get() = composer != null

    /**
     * Runs an operation that modifies the composer, returning its [ComposerUpdate] or null if the
     * operation failed.
     */
    fun update(block: ComposerModelInterface.() -> ComposerUpdate): ComposerUpdate? {
        val composer = composer ?: return null
        return runCatching { composer.block() }
            .onFailure(::onFailure)
            .getOrNull()
    }

    /**
     * Runs an operation that only reads the composer state, returning [default] if it failed.
     */
    fun <T> query(default: T, block: ComposerModelInterface.() -> T): T {
        val composer = composer ?: return default
        return runCatching { composer.block() }
            .onFailure(::onFailure)
            .getOrDefault(default)
    }

    fun close() {
        (composer as? ComposerModel)?.destroy()
        composer = null
    }

    private fun create(): ComposerModelInterface? =
        runCatching(factory)
            .onFailure(onError)
            .getOrNull()

    private fun onFailure(error: Throwable) {
        Timber.e(error, "Error in the Rust composer")
        onError(error)
        if (error is InternalException) {
            // The Rust model panicked and its state can no longer be trusted: start over.
            close()
            composer = create()
            runCatching { composer?.setContentFromHtml(recoveryHtml) }
                .onFailure {
                    Timber.e(it, "Could not restore the composer content")
                    onError(it)
                }
        }
    }
}
