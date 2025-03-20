/*
 * Copyright 2025 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE files in the repository root for full details.
 */

package io.element.android.wysiwyg.link

/**
 * Data class defining a link, i.e. a target url and a text.
 * @property url The url of the string
 * @property text The text of the link. If not provided, the url will be used, but in this case, no
 * validation will be performed.
 */
data class Link(
    val url: String,
    val text: String = url,
)
