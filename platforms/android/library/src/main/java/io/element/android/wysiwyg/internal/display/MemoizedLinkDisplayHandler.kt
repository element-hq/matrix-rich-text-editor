/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.internal.display

import io.element.android.wysiwyg.display.TextDisplay
import io.element.android.wysiwyg.display.MentionDisplayHandler

/**
 * This [MentionDisplayHandler] ensures that the editor does not request how to display the same item
 * from the host app on every editor update by caching the results in memory.
 */
internal class MemoizingMentionDisplayHandler(
    private val delegate: MentionDisplayHandler
): MentionDisplayHandler {
    private val cache = mutableMapOf<Pair<String, String>, TextDisplay>()
    private var atRoomCache: TextDisplay? = null
    override fun resolveMentionDisplay(text: String, url: String): TextDisplay {
        val key = text to url
        val cached = cache[key]

        if(cached != null) {
            return cached
        }

        val calculated = delegate.resolveMentionDisplay(text, url)

        cache[key] = calculated

        return calculated
    }

    override fun resolveAtRoomMentionDisplay(): TextDisplay {
        atRoomCache?.let {
            return it
        }

        val calculated = delegate.resolveAtRoomMentionDisplay()

        atRoomCache = calculated

        return calculated
    }

    fun delegateEquals(other: MentionDisplayHandler?): Boolean {
        return delegate == other
    }
}
