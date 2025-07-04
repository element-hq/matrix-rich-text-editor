/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.wysiwyg.compose.matrix

/**
 * Utility model class for the sample app to represent a mention to a
 * matrix.org user or room
 */
sealed class Mention(
    val display: String,
) {
    abstract val key: String
    val link get() = "https://matrix.to/#/$key$display:matrix.org"
    val text get() = "$key$display"

    class Room(
        display: String
    ): Mention(display) {
        override val key: String = "#"
    }

    class User(
        display: String
    ): Mention(display) {
        override val key: String = "@"
    }

    class SlashCommand(
        display: String
    ): Mention(display) {
        override val key: String = "/"
    }

    object NotifyEveryone: Mention("room") {
        override val key: String = "@"
    }
}
