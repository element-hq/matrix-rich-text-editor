/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.internal

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.rememberCoroutineScope
import io.element.android.wysiwyg.compose.RichTextEditorState
import io.element.android.wysiwyg.view.models.InlineFormat
import io.element.android.wysiwyg.view.models.LinkAction
import kotlinx.coroutines.flow.FlowCollector
import kotlinx.coroutines.launch
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction

/**
 * Fake behaviour for use in preview and test environments.
 * This implementation does not actually connect to a view, but instead updates the state
 * in _some_ way. The changes made to the state are not guaranteed to be the same as the
 * real implementation.
 */
@Composable
internal fun FakeViewConnection(
    state: RichTextEditorState
) {
    val coroutineScope = rememberCoroutineScope()

    LaunchedEffect(Unit) {
        coroutineScope.launch {
            state.activeViewKey = "fake"
            state.viewActions.collect(FakeViewActionCollector(state))
        }
    }

}

internal class FakeViewActionCollector(
    val state: RichTextEditorState
): FlowCollector<ViewAction> {
    override suspend fun emit(value: ViewAction) {
        when(value) {
            ViewAction.Indent -> indent()
            is ViewAction.InsertLink -> insertLink(value.url)
            ViewAction.Redo -> redo()
            ViewAction.RemoveLink -> removeLink()
            ViewAction.RequestFocus -> requestFocus()
            is ViewAction.SetHtml -> setHtml(value.html)
            is ViewAction.SetMarkdown -> setMarkdown(value.markdown)
            is ViewAction.SetLink -> setLink(value.url)
            ViewAction.ToggleCodeBlock -> toggleCodeBlock()
            is ViewAction.ToggleInlineFormat -> toggleInlineFormat(value.inlineFormat)
            is ViewAction.ToggleList -> toggleList(value.ordered)
            ViewAction.ToggleQuote -> toggleQuote()
            ViewAction.Undo -> undo()
            ViewAction.Unindent -> unindent()
            is ViewAction.ReplaceSuggestionText -> {
                state.messageHtml = state.messageHtml.replaceLast(Regex("(/\\w+)"), value.text)
                state.messageMarkdown = state.messageMarkdown.replaceLast(Regex("(/\\w+)"), value.text)
            }
            is ViewAction.InsertMentionAtSuggestion -> {
                state.messageHtml = state.messageHtml.replaceLast(Regex("(@\\w+)"), "<a href='${value.url}'>${value.text}</a>")
                state.messageMarkdown = state.messageMarkdown.replaceLast(Regex("(@\\w+)"), "[${value.text}](${value.url})")
            }
            is ViewAction.InsertAtRoomMentionAtSuggestion -> {
                val htmlReplacement = "<a data-mention-type=\"at-room\" href=\"#\" contenteditable=\"false\">@room</a>"
                state.messageHtml = state.messageHtml.replaceLast(Regex("(@\\w+)"), htmlReplacement)
                state.messageMarkdown = state.messageMarkdown.replaceLast(Regex("(@\\w+)"), "@room")
            }
            is ViewAction.SetSelection -> {
                state.selection = value.start to value.end
            }
        }
    }
    private fun toggleInlineFormat(inlineFormat: InlineFormat): Boolean {
        updateActionState(inlineFormat.toComposerAction())
        return true
    }

    private fun toggleList(ordered: Boolean) {
        updateActionState(
            if (ordered) {
                ComposerAction.ORDERED_LIST
            } else {
                ComposerAction.UNORDERED_LIST
            }
        )
    }

    private fun toggleCodeBlock(): Boolean {
        updateActionState(ComposerAction.CODE_BLOCK)
        return true
    }

    private fun toggleQuote(): Boolean {
        updateActionState(ComposerAction.QUOTE)
        return true
    }

    private fun undo() {
        updateActionState(ComposerAction.UNDO)
    }

    private fun redo() {
        updateActionState(ComposerAction.REDO)
    }

    private fun indent() {
        updateActionState(ComposerAction.INDENT)
    }

    private fun unindent() {
        updateActionState(ComposerAction.UNINDENT)
    }

    private fun setHtml(html: String) {
        state.messageHtml = html
        state.messageMarkdown = html
    }

    private fun setMarkdown(markdown: String) {
        state.messageHtml = markdown
        state.messageMarkdown = markdown
    }

    private fun requestFocus(): Boolean {
        state.hasFocus = true
        return true
    }

    private fun setLink(url: String?) {
        state.linkAction = url?.let { LinkAction.SetLink(it) } ?: LinkAction.InsertLink
    }

    private fun removeLink() {
        state.linkAction = LinkAction.InsertLink
    }

    private fun insertLink(url: String) {
        state.linkAction = LinkAction.SetLink(url)
    }

    private fun updateActionState(action: ComposerAction) {
        val actions = state.actions.toMutableMap()
        val currentState: ActionState =
            actions[action] ?: ActionState.ENABLED
        val newAction = if (currentState == ActionState.ENABLED) {
            ActionState.REVERSED
        } else {
            ActionState.ENABLED
        }
        actions[action] = newAction
        state.actions = actions
    }

    private fun String.replaceLast(regex: Regex, value: String): String {
        val lastMatch = regex.findAll(this).lastOrNull()
        return lastMatch?.let {
            replaceRange(it.range, value)
        } ?: this
    }
}

private fun InlineFormat.toComposerAction() = when (this) {
    InlineFormat.Bold -> ComposerAction.BOLD
    InlineFormat.Italic -> ComposerAction.ITALIC
    InlineFormat.StrikeThrough -> ComposerAction.STRIKE_THROUGH
    InlineFormat.InlineCode -> ComposerAction.INLINE_CODE
    InlineFormat.Underline -> ComposerAction.UNDERLINE
}
