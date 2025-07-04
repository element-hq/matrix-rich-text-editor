/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.view.spans

/**
 * Used to override any [MentionDisplayHandler] and force text to be plain.
 * This can be used, for example, inside a code block where text must be displayed as-is.
 */
internal interface PlainAtRoomMentionDisplaySpan