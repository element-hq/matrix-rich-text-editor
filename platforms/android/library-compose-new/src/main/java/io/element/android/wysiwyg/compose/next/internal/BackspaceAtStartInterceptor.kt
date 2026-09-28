/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next.internal

import android.view.inputmethod.InputConnection
import android.view.inputmethod.InputConnectionWrapper
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.platform.PlatformTextInputInterceptor
import androidx.compose.ui.platform.PlatformTextInputMethodRequest
import androidx.compose.ui.platform.PlatformTextInputSession

/**
 * Soft keyboards delete backwards with `deleteSurroundingText`, even when there's nothing before
 * the cursor. The text field ignores these calls since the text doesn't change, but the composer
 * may still need to handle them, i.e. to turn the first list item into a paragraph.
 *
 * @param onBackspaceAtStart called when deleting backwards, returns whether it handled the deletion.
 */
@OptIn(ExperimentalComposeUiApi::class)
internal class BackspaceAtStartInterceptor(
    private val onBackspaceAtStart: () -> Boolean,
) : PlatformTextInputInterceptor {
    override suspend fun interceptStartInputMethod(
        request: PlatformTextInputMethodRequest,
        nextHandler: PlatformTextInputSession,
    ): Nothing {
        nextHandler.startInputMethod(
            PlatformTextInputMethodRequest { outAttributes ->
                Connection(request.createInputConnection(outAttributes))
            }
        )
    }

    private inner class Connection(target: InputConnection) : InputConnectionWrapper(target, false) {
        override fun deleteSurroundingText(beforeLength: Int, afterLength: Int): Boolean {
            if (beforeLength > 0 && afterLength == 0 && onBackspaceAtStart()) return true
            return super.deleteSurroundingText(beforeLength, afterLength)
        }

        override fun deleteSurroundingTextInCodePoints(beforeLength: Int, afterLength: Int): Boolean {
            if (beforeLength > 0 && afterLength == 0 && onBackspaceAtStart()) return true
            return super.deleteSurroundingTextInCodePoints(beforeLength, afterLength)
        }
    }
}
