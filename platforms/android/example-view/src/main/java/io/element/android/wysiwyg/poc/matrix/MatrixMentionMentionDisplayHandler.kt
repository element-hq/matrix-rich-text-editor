/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.poc.matrix

import io.element.android.wysiwyg.display.TextDisplay
import io.element.android.wysiwyg.display.MentionDisplayHandler
import uniffi.wysiwyg_composer.MentionDetector

/**
 * Convenience implementation of a [MentionDisplayHandler] that detects Matrix mentions and
 * displays them as default pills.
 */
object MatrixMentionMentionDisplayHandler: MentionDisplayHandler {
    override fun resolveMentionDisplay(text: String, url: String): TextDisplay =
        TextDisplay.Pill

    override fun resolveAtRoomMentionDisplay(): TextDisplay =
        TextDisplay.Pill
}
