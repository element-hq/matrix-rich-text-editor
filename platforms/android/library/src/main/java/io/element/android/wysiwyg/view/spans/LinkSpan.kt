/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.view.spans

import android.text.TextPaint
import android.text.style.URLSpan

internal class LinkSpan(
    url: String
) : URLSpan(url), PlainAtRoomMentionDisplaySpan {
    override fun updateDrawState(ds: TextPaint) {
        // Check if the text is already underlined (for example by an UnderlineSpan)
        val wasUnderlinedByAnotherSpan = ds.isUnderlineText

        super.updateDrawState(ds)

        if (!wasUnderlinedByAnotherSpan) {
            ds.isUnderlineText = false
        }
    }
}