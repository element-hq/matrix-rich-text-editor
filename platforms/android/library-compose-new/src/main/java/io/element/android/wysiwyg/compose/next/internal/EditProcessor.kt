/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next.internal

import androidx.compose.foundation.text.input.TextFieldBuffer
import androidx.compose.ui.text.TextRange
import io.element.android.wysiwyg.compose.next.internal.document.HtmlDocumentParser
import io.element.android.wysiwyg.compose.next.internal.document.RichTextDocument
import io.element.android.wysiwyg.view.models.LinkAction
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction
import uniffi.wysiwyg_composer.ComposerModelInterface
import uniffi.wysiwyg_composer.ComposerUpdate
import uniffi.wysiwyg_composer.LinkActionUpdate
import uniffi.wysiwyg_composer.MenuAction
import uniffi.wysiwyg_composer.MenuState
import uniffi.wysiwyg_composer.TextUpdate
import uniffi.wysiwyg_composer.LinkAction as InternalLinkAction

/**
 * Translates the edits made to a [TextFieldBuffer] into operations of the Rust composer and applies
 * the composer's resulting content back to the buffer.
 *
 * The text in the buffer is always the logical text of a [RichTextDocument], so every index can
 * be passed to the composer as is.
 *
 * To avoid resetting the IME composition, the buffer is only modified when the composer's result
 * differs from what the user typed, and even then only the smallest possible edit is applied.
 */
internal class EditProcessor(
    private val session: ComposerSession,
    private val listener: Listener,
) {
    interface Listener {
        fun onDocumentChanged(document: RichTextDocument, html: String)
        fun onActionStatesChanged(actionStates: Map<ComposerAction, ActionState>)
        fun onMenuActionChanged(menuAction: MenuAction)
        fun onLinkActionChanged(linkAction: LinkAction?)
    }

    /** The current selection in the Rust composer. */
    var composerSelection: TextRange = TextRange.Zero
        private set

    var menuAction: MenuAction = MenuAction.None
        private set

    private class Result {
        var html: String? = null
        var selection: TextRange? = null
        var failed = false
    }

    /**
     * Handles a user edit (IME, hardware keyboard, paste...) in [buffer].
     */
    fun onUserInput(buffer: TextFieldBuffer) {
        // Without a composer, just behave like a plain text field
        if (!session.isAvailable) return

        val changes = buffer.changes
        if (changes.changeCount == 0) {
            syncComposerSelection(buffer.selection)
            return
        }

        var originalStart = Int.MAX_VALUE
        var originalEnd = Int.MIN_VALUE
        var newStart = Int.MAX_VALUE
        var newEnd = Int.MIN_VALUE
        for (i in 0 until changes.changeCount) {
            val originalRange = changes.getOriginalRange(i)
            val range = changes.getRange(i)
            originalStart = minOf(originalStart, originalRange.min)
            originalEnd = maxOf(originalEnd, originalRange.max)
            newStart = minOf(newStart, range.min)
            newEnd = maxOf(newEnd, range.max)
        }
        val edit = minimalEdit(
            old = buffer.originalText,
            new = buffer.asCharSequence(),
            oldStart = originalStart,
            oldEnd = originalEnd,
            newStart = newStart,
            newEnd = newEnd,
        )
        if (edit == null) {
            syncComposerSelection(buffer.selection)
            return
        }

        // Make sure the composer's selection is where the edit happened
        syncComposerSelection(buffer.originalSelection)
        val result = Result()
        applyEdit(edit, buffer.originalSelection, result)
        writeResult(buffer, result, preferBufferSelection = true)
    }

    /**
     * Updates the composer's selection if it's not the same as [selection].
     */
    fun syncComposerSelection(selection: TextRange) {
        if (selection == composerSelection) return
        composerSelection = selection
        // The text update is ignored, the selection in the text field is always the one to follow
        val update = session.update { select(selection.min.toUInt(), selection.max.toUInt()) }
        handleMenuUpdates(update)
    }

    /**
     * Runs a programmatic composer operation and applies its result to [buffer].
     */
    fun perform(buffer: TextFieldBuffer, block: ComposerModelInterface.() -> ComposerUpdate) {
        syncComposerSelection(buffer.selection)
        val result = Result()
        consume(session.update(block), result)
        writeResult(buffer, result, preferBufferSelection = false)
    }

    private fun applyEdit(edit: TextEdit, originalSelection: TextRange, result: Result) {
        val (start, end, replacement) = edit
        if (replacement.isEmpty()) {
            val update = session.update {
                val isSingleChar = end - start == 1
                when {
                    // Use backspace/delete when possible, so the composer can apply its special
                    // behaviours (i.e. removing list items or mentions)
                    isSingleChar && originalSelection.collapsed && originalSelection.start == end -> backspace()
                    isSingleChar && originalSelection.collapsed && originalSelection.start == start -> delete()
                    originalSelection.min == start && originalSelection.max == end -> backspace()
                    else -> deleteIn(start.toUInt(), end.toUInt())
                }
            }
            consume(update, result)
            return
        }

        // New lines must be added as new paragraphs using `enter`
        val segments = replacement.split('\n')
        val first = segments.first()
        if (first.isNotEmpty() || start != end) {
            consume(session.update { replaceTextIn(first, start.toUInt(), end.toUInt()) }, result)
        } else {
            syncComposerSelection(TextRange(start))
        }
        for (segment in segments.drop(1)) {
            consume(session.update { enter() }, result)
            if (segment.isNotEmpty()) {
                consume(session.update { replaceText(segment) }, result)
            }
        }
    }

    private fun consume(update: ComposerUpdate?, result: Result) {
        if (update == null) {
            result.failed = true
            return
        }
        handleMenuUpdates(update)
        when (val textUpdate = update.textUpdate()) {
            is TextUpdate.ReplaceAll -> {
                result.html = textUpdate.replacementHtml.toJvmString()
                result.selection = TextRange(textUpdate.startUtf16Codeunit.toInt(), textUpdate.endUtf16Codeunit.toInt())
            }
            is TextUpdate.Select -> {
                result.selection = TextRange(textUpdate.startUtf16Codeunit.toInt(), textUpdate.endUtf16Codeunit.toInt())
            }
            is TextUpdate.Keep -> Unit
        }
    }

    private fun writeResult(buffer: TextFieldBuffer, result: Result, preferBufferSelection: Boolean) {
        if (result.failed && result.html == null) {
            // The composer failed or was recreated, get back to whatever state it has now
            session.query(null) { getCurrentDomState() }?.let { state ->
                result.html = state.html.toJvmString()
                result.selection = TextRange(state.start.toInt(), state.end.toInt())
            }
        }
        result.selection?.let { composerSelection = it }

        val html = result.html
        if (html != null) {
            val document = HtmlDocumentParser.parse(html) {
                session.query(null) { getContentAsPlainText() }
            }
            session.recoveryHtml = html
            listener.onDocumentChanged(document, html)

            val current = buffer.asCharSequence()
            if (!current.isLooselyEqualTo(document.text)) {
                minimalEdit(current, document.text)?.let {
                    buffer.replace(it.start, it.end, it.replacement)
                }
                val selection = (result.selection ?: composerSelection).coerceIn(document.text.length)
                buffer.selection = selection
                composerSelection = selection
                return
            }
        }

        val selection = result.selection?.coerceIn(buffer.length) ?: return
        if (buffer.selection != selection) {
            if (preferBufferSelection) {
                syncComposerSelection(buffer.selection)
            } else {
                buffer.selection = selection
            }
        }
    }

    private fun handleMenuUpdates(update: ComposerUpdate?) {
        update ?: return
        val menuState = update.menuState()
        if (menuState is MenuState.Update) {
            listener.onActionStatesChanged(menuState.actionStates)
        }
        val newMenuAction = update.menuAction()
        if (newMenuAction !is MenuAction.Keep) {
            menuAction = newMenuAction
            listener.onMenuActionChanged(newMenuAction)
        }
        val linkActionUpdate = update.linkAction()
        if (linkActionUpdate is LinkActionUpdate.Update) {
            listener.onLinkActionChanged(linkActionUpdate.linkAction.toApiModel())
        }
    }
}

internal fun InternalLinkAction.toApiModel(): LinkAction? =
    when (this) {
        is InternalLinkAction.Edit -> LinkAction.SetLink(currentUrl = url)
        is InternalLinkAction.Create -> LinkAction.SetLink(currentUrl = null)
        is InternalLinkAction.CreateWithText -> LinkAction.InsertLink
        is InternalLinkAction.Disabled -> null
    }

/**
 * Strings are passed from Rust as lists of UTF-16 code units.
 */
internal fun List<UShort>.toJvmString(): String = String(CharArray(size) { this[it].toInt().toChar() })

private fun TextRange.coerceIn(length: Int) =
    TextRange(start.coerceIn(0, length), end.coerceIn(0, length))
