/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next

import androidx.compose.foundation.text.input.InputTransformation
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.Saver
import androidx.compose.runtime.saveable.listSaver
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.text.TextRange
import io.element.android.wysiwyg.compose.next.internal.ComposerSession
import io.element.android.wysiwyg.compose.next.internal.EditProcessor
import io.element.android.wysiwyg.compose.next.internal.document.RichTextDocument
import io.element.android.wysiwyg.view.models.InlineFormat
import io.element.android.wysiwyg.view.models.LinkAction
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction
import uniffi.wysiwyg_composer.ComposerModelInterface
import uniffi.wysiwyg_composer.ComposerUpdate
import uniffi.wysiwyg_composer.MentionsState
import uniffi.wysiwyg_composer.MenuAction
import uniffi.wysiwyg_composer.newComposerModel

/**
 * State of a [RichTextEditor], backed by the Rust composer model.
 *
 * All the formatting operations are synchronous and must be called from the main thread.
 *
 * The state owns native resources: call [close] when it's no longer needed, or create it with
 * [rememberRichTextEditorState] which takes care of it.
 */
@Stable
class RichTextEditorState internal constructor(
    initialHtml: String,
    composerFactory: () -> ComposerModelInterface,
) : AutoCloseable {
    constructor(initialHtml: String = "") : this(initialHtml, ::newComposerModel)

    /**
     * Called when the Rust composer throws an error. The editor will try to recover from it.
     */
    var onError: ((Throwable) -> Unit)? = null

    internal val textFieldState = TextFieldState()

    internal var document: RichTextDocument by mutableStateOf(RichTextDocument.Empty)
        private set

    /** The internal HTML of the composer. Use [getMessageHtml] to get the HTML to send. */
    var internalHtml: String by mutableStateOf("")
        private set

    /** Whether each formatting action is currently enabled, disabled or reversed. */
    var actions: Map<ComposerAction, ActionState> by mutableStateOf(emptyMap())
        private set

    /** The current suggestion (i.e. `@alice`, `#room`, `/command`) being typed, if any. */
    var menuAction: MenuAction by mutableStateOf(MenuAction.None)
        private set

    /** The link action available for the current selection. */
    var linkAction: LinkAction? by mutableStateOf(null)
        private set

    /** The mentions currently in the content. */
    var mentionsState: MentionsState? by mutableStateOf(null)
        private set

    /** The current selection, in the same UTF-16 indexes the composer uses. */
    val selection: TextRange get() = textFieldState.selection

    /** Whether the editor has no text. */
    val isEmpty: Boolean get() = textFieldState.text.isEmpty()

    private val session = ComposerSession(
        factory = composerFactory,
        onError = { onError?.invoke(it) },
    )

    private val processor = EditProcessor(
        session = session,
        listener = object : EditProcessor.Listener {
            override fun onDocumentChanged(document: RichTextDocument, html: String) {
                this@RichTextEditorState.document = document
                internalHtml = html
                mentionsState = session.query(null) { getMentionsState() }
            }

            override fun onActionStatesChanged(actionStates: Map<ComposerAction, ActionState>) {
                actions = actionStates
            }

            override fun onMenuActionChanged(menuAction: MenuAction) {
                this@RichTextEditorState.menuAction = menuAction
            }

            override fun onLinkActionChanged(linkAction: LinkAction?) {
                this@RichTextEditorState.linkAction = linkAction
            }
        },
    )

    internal val inputTransformation = InputTransformation { processor.onUserInput(this) }

    init {
        actions = session.query(emptyMap()) { actionStates() }
        if (initialHtml.isNotEmpty()) {
            setHtml(initialHtml)
        }
    }

    fun toggleInlineFormat(inlineFormat: InlineFormat) = perform {
        when (inlineFormat) {
            InlineFormat.Bold -> bold()
            InlineFormat.Italic -> italic()
            InlineFormat.Underline -> underline()
            InlineFormat.StrikeThrough -> strikeThrough()
            InlineFormat.InlineCode -> inlineCode()
        }
    }

    fun undo() = perform { undo() }

    fun redo() = perform { redo() }

    fun toggleList(ordered: Boolean) = perform { if (ordered) orderedList() else unorderedList() }

    fun indent() = perform { indent() }

    fun unindent() = perform { unindent() }

    fun toggleCodeBlock() = perform { codeBlock() }

    fun toggleQuote() = perform { quote() }

    /** Replaces the whole content with [html]. */
    fun setHtml(html: String) = perform { setContentFromHtml(html) }

    /** Replaces the whole content with [markdown]. */
    fun setMarkdown(markdown: String) = perform { setContentFromMarkdown(markdown) }

    /** Removes all the content. */
    fun clear() = perform { clear() }

    /** Sets a link to [url] in the current selection, or removes it if [url] is null. */
    fun setLink(url: String?) = perform {
        if (url != null) setLink(url, emptyList()) else removeLinks()
    }

    fun removeLink() = perform { removeLinks() }

    /** Inserts [text] linking to [url] at the current selection. */
    fun insertLink(url: String, text: String) = perform { setLinkWithText(url, text, emptyList()) }

    /** Replaces the current suggestion (see [menuAction]) with [text]. */
    fun replaceSuggestion(text: String) {
        val suggestion = (processor.menuAction as? MenuAction.Suggestion)?.suggestionPattern ?: return
        perform { replaceTextSuggestion(text, suggestion, appendSpace = true) }
    }

    /** Replaces the current suggestion (see [menuAction]) with a mention of [url]. */
    fun insertMentionAtSuggestion(text: String, url: String) {
        val suggestion = (processor.menuAction as? MenuAction.Suggestion)?.suggestionPattern ?: return
        perform { insertMentionAtSuggestion(url, text, suggestion, emptyList()) }
    }

    /** Replaces the current suggestion (see [menuAction]) with an `@room` mention. */
    fun insertAtRoomMentionAtSuggestion() {
        val suggestion = (processor.menuAction as? MenuAction.Suggestion)?.suggestionPattern ?: return
        perform { insertAtRoomMentionAtSuggestion(suggestion) }
    }

    /** Inserts a mention of [url] at the current selection. */
    fun insertMention(text: String, url: String) = perform { insertMention(url, text, emptyList()) }

    fun setSelection(start: Int, end: Int = start) {
        val length = textFieldState.text.length
        textFieldState.edit {
            selection = TextRange(start.coerceIn(0, length), end.coerceIn(0, length))
        }
    }

    /** Sets custom suggestion patterns (in addition to `@`, `#` and `/`) to detect in the text. */
    fun setCustomSuggestionPatterns(patterns: List<String>) {
        session.query(Unit) { setCustomSuggestionPatterns(patterns) }
    }

    /** Returns the HTML to use when sending the content as a message. */
    fun getMessageHtml(): String = session.query(internalHtml) { getContentAsMessageHtml() }

    /** Returns the Markdown to use when sending the content as a message. */
    fun getMessageMarkdown(): String = session.query("") { getContentAsMessageMarkdown() }

    /** Returns the content as Markdown, keeping the mentions as links. */
    fun getMarkdown(): String = session.query("") { getContentAsMarkdown() }

    /** Returns the content as plain text. */
    fun getPlainText(): String = session.query("") { getContentAsPlainText() }

    override fun close() {
        session.close()
    }

    /** The selection according to the Rust composer, for testing purposes. */
    internal fun composerSelection(): TextRange? = session.query(null) {
        getCurrentDomState().let { TextRange(it.start.toInt(), it.end.toInt()) }
    }

    internal fun onSelectionChanged(selection: TextRange) {
        processor.syncComposerSelection(selection)
    }

    private fun perform(block: ComposerModelInterface.() -> ComposerUpdate) {
        textFieldState.edit {
            processor.perform(this, block)
        }
        // Undo is handled by the composer
        textFieldState.undoState.clearHistory()
    }

    companion object {
        /**
         * Saves the internal HTML and the selection of the editor.
         */
        val Saver: Saver<RichTextEditorState, *> = listSaver(
            save = { listOf(it.internalHtml, it.selection.start, it.selection.end) },
            restore = { saved ->
                RichTextEditorState(initialHtml = saved[0] as String).apply {
                    setSelection(saved[1] as Int, saved[2] as Int)
                }
            },
        )
    }
}

/**
 * Creates and remembers a [RichTextEditorState], closing it when it leaves the composition.
 */
@Composable
fun rememberRichTextEditorState(
    initialHtml: String = "",
): RichTextEditorState {
    val state = rememberSaveable(saver = RichTextEditorState.Saver) {
        RichTextEditorState(initialHtml = initialHtml)
    }
    DisposableEffect(state) {
        onDispose { state.close() }
    }
    return state
}
