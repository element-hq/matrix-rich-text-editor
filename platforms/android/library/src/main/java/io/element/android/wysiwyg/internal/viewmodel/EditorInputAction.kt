/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.internal.viewmodel

import io.element.android.wysiwyg.view.models.InlineFormat
import uniffi.wysiwyg_composer.ComposerModel

/**
 * Text editing actions to be performed by the Rust code through the [ComposerModel] component.
 */
internal sealed interface EditorInputAction {
    /**
     * Replaces the text at the current selection with the provided [value] in plain text.
     */
    data class ReplaceText(val value: CharSequence): EditorInputAction

    /**
     * Replaces the text in the [start]..[end] selection with the provided [value] in plain text.
     */
    data class ReplaceTextIn(val start: UInt, val end: UInt, val value: CharSequence): EditorInputAction

    /**
     * Replaces the text of the current suggestion range with the provided [value] in plain text.
     */
    data class ReplaceTextSuggestion(val value: String): EditorInputAction

    /**
     * Replaces the whole contents of the editor with the passed [html], re-creating the Dom.
     */
    data class ReplaceAllHtml(val html: String): EditorInputAction

    /**
     * Replaces the whole contents of the editor with the passed [markdown], re-creating the Dom.
     */
    data class ReplaceAllMarkdown(val markdown: String): EditorInputAction

    /**
     * Deletes text in the [start]..[end] selection
     */
    data class DeleteIn(val start: Int, val end: Int): EditorInputAction

    /**
     * Deletes text for the current selection
     */
    object Delete : EditorInputAction

    /**
     * Adds a new line break at the current selection.
     */
    object InsertParagraph: EditorInputAction

    /**
     * Removes text in a backwards direction given the current selection.
     */
    object BackPress: EditorInputAction

    /**
     * Applies the passed inline [format] to the current selection, either creating or extending it
     * or removing it if it was present in that selection.
     */
    data class ApplyInlineFormat(val format: InlineFormat): EditorInputAction

    object CodeBlock: EditorInputAction

    object Quote: EditorInputAction

    /**
     * Un-does the previous action, restoring the previous editor state.
     */
    object Undo: EditorInputAction

    /**
     * Re-does the last undone action, restoring its state.
     */
    object Redo: EditorInputAction

    /**
     * Add or edit a link to the [url] in the current selection.
     */
    data class SetLink(val url: String): EditorInputAction

    /**
     * Remove link on the current selection.
     */
    object RemoveLink: EditorInputAction

    /**
     * Create text with a link.
     */
    data class SetLinkWithText(val link: String, val text: String): EditorInputAction

    /**
     * Replaces the suggestion text with a mention.
     */
    data class InsertMentionAtSuggestion(
        val url: String,
        val text: String,
    ): EditorInputAction

    /**
     * Replaces the suggesetion with an `@room` mention
     */
    object InsertAtRoomMentionAtSuggestion : EditorInputAction

    /**
     * Creates a list, [ordered] if true or unordered in the current selection.
     */
    data class ToggleList(val ordered: Boolean): EditorInputAction

    object Indent: EditorInputAction

    object Unindent: EditorInputAction

    /**
     * Sets the current selection to the [start]..[end] range in the composer,
     * using composer indices. These may not match the UI text indices.
     *
     * @param start The start index of the selection in the composer.
     * @param end The end index of the selection in the composer.
     */
    data class UpdateSelection(val start: UInt, val end: UInt) : EditorInputAction
}
