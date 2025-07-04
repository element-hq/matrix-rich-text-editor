/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.internal

import io.element.android.wysiwyg.view.models.InlineFormat

internal sealed class ViewAction {
    data class ToggleInlineFormat(val inlineFormat: InlineFormat): ViewAction()
    data class ToggleList(val ordered: Boolean): ViewAction()
    data object ToggleCodeBlock: ViewAction()
    data object ToggleQuote: ViewAction()
    data object Undo: ViewAction()
    data object Redo: ViewAction()
    data object Indent: ViewAction()
    data object Unindent: ViewAction()
    data class SetHtml(val html: String): ViewAction()
    data class SetMarkdown(val markdown: String): ViewAction()
    data object RequestFocus: ViewAction()
    data class SetLink(val url: String?): ViewAction()
    data object RemoveLink: ViewAction()
    data class InsertLink(val url: String, val text: String): ViewAction()
    data class ReplaceSuggestionText(val text: String): ViewAction()
    data class InsertMentionAtSuggestion(val text: String, val url: String): ViewAction()
    data object InsertAtRoomMentionAtSuggestion: ViewAction()
    data class SetSelection(val start: Int, val end: Int): ViewAction()
}
