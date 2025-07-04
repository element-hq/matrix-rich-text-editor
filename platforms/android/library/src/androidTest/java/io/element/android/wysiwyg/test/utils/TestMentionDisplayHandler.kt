/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.test.utils

import io.element.android.wysiwyg.display.MentionDisplayHandler
import io.element.android.wysiwyg.display.TextDisplay

class TestMentionDisplayHandler(
    val textDisplay: TextDisplay = TextDisplay.Pill,
) : MentionDisplayHandler {
    override fun resolveAtRoomMentionDisplay(): TextDisplay = textDisplay
    override fun resolveMentionDisplay(text: String, url: String): TextDisplay = textDisplay
}
