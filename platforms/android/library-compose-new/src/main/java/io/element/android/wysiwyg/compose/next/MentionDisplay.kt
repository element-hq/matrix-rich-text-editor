/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next

import androidx.compose.runtime.Immutable

/**
 * A mention found in the editor's content.
 *
 * @property url the permalink of the mentioned user or room, `#` for `@room` mentions.
 * @property text the text stored in the mention, usually the display name of the user or room.
 * @property isAtRoom whether this is an `@room` mention.
 */
@Immutable
data class Mention(
    val url: String,
    val text: String,
    val isAtRoom: Boolean,
)

/**
 * How a [Mention] should be rendered in the editor.
 */
@Immutable
sealed interface MentionDisplay {
    val text: String

    /** Rendered as [text] on top of a rounded background. */
    data class Pill(override val text: String) : MentionDisplay

    /** Rendered as [text] using the mention span style, without any background. */
    data class Text(override val text: String) : MentionDisplay
}
