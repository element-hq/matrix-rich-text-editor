/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.test.utils

import android.text.TextUtils

fun CharSequence.dumpSpans(): List<String> {
    val spans = mutableListOf<String>()
    TextUtils.dumpSpans(
        this, { span ->
            val spanWithoutHash = span.split(" ").filterIndexed { index, _ ->
                index != 1
            }.joinToString(" ")

            spans.add(spanWithoutHash)
        }, ""
    )
    return spans
}