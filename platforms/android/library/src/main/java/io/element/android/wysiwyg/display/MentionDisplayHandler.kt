/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.display

/**
 * Clients can implement a mention display handler to let the editor
 * know how to display mentions.
 */
interface MentionDisplayHandler {
    /**
     * Return the method with which to display a given mention
     */
    fun resolveMentionDisplay(text: String, url: String): TextDisplay

    /**
     * Return the method with which to display an at-room mention
     */
    fun resolveAtRoomMentionDisplay(): TextDisplay
}
