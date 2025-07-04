/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.mocks

import io.element.android.wysiwyg.extensions.toUShortList
import uniffi.wysiwyg_composer.TextUpdate

object MockTextUpdateFactory {
    fun createReplaceAll(
        html: String = "",
        start: Int = 0,
        end: Int = 0,
    ) = TextUpdate.ReplaceAll(
        replacementHtml = html.toUShortList(),
        startUtf16Codeunit = start.toUInt(),
        endUtf16Codeunit = end.toUInt()
    )
}
