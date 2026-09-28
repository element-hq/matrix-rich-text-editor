/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next

import androidx.compose.foundation.text.input.TextFieldBuffer
import androidx.compose.ui.text.TextRange
import io.element.android.wysiwyg.compose.next.internal.document.HtmlDocumentParser
import io.element.android.wysiwyg.compose.next.internal.isLooselyEqualTo
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Assume.assumeTrue
import org.junit.Before
import uniffi.wysiwyg_composer.newComposerModel

/**
 * Base class for tests using the real Rust composer. They need a host build of the Rust library,
 * otherwise they're skipped.
 */
abstract class RustTest {
    @Before
    fun assumeRustAvailable() {
        assumeTrue("Host build of the Rust library not found, run `cargo build -p uniffi-wysiwyg-composer`", isRustAvailable)
    }

    companion object {
        val isRustAvailable by lazy {
            runCatching { newComposerModel().destroy() }.isSuccess
        }
    }
}

/**
 * Drives a [RichTextEditorState] the same way a `BasicTextField` would: user edits are applied to
 * the text field buffer and then the editor's input transformation runs.
 */
internal class EditorTester(initialHtml: String = "") : AutoCloseable {
    val state = RichTextEditorState(initialHtml).apply {
        onError = { throw AssertionError("Rust composer error", it) }
    }

    val text: String get() = state.textFieldState.text.toString()
    val selection: TextRange get() = state.textFieldState.selection
    val html: String get() = state.internalHtml

    fun userEdit(block: TextFieldBuffer.() -> Unit) {
        state.textFieldState.edit {
            block()
            with(state.inputTransformation) { transformInput() }
        }
        assertConsistent()
    }

    /** Types [value] one character (or surrogate pair) at a time. */
    fun type(value: String) {
        var i = 0
        while (i < value.length) {
            val count = if (value[i].isHighSurrogate()) 2 else 1
            commit(value.substring(i, i + count))
            i += count
        }
    }

    /** Commits [value] as a single edit, like an IME suggestion or a paste. */
    fun commit(value: String) = userEdit {
        val current = selection
        replace(current.min, current.max, value)
        selection = TextRange(current.min + value.length)
    }

    /** Replaces the [start]..[end] range, like an IME updating its composing text. */
    fun replace(start: Int, end: Int, value: String) = userEdit {
        replace(start, end, value)
        selection = TextRange(start + value.length)
    }

    fun enter() = commit("\n")

    /** Deletes backwards like the IME does: [count] code units or the selected text. */
    fun backspace(count: Int = 1) = userEdit {
        val current = selection
        if (current.collapsed) {
            replace(current.start - count, current.start, "")
            selection = TextRange(current.start - count)
        } else {
            replace(current.min, current.max, "")
            selection = TextRange(current.min)
        }
    }

    /** Deletes forwards like the delete key: [count] code units or the selected text. */
    fun forwardDelete(count: Int = 1) = userEdit {
        val current = selection
        if (current.collapsed) {
            replace(current.start, current.start + count, "")
            selection = TextRange(current.start)
        } else {
            replace(current.min, current.max, "")
            selection = TextRange(current.min)
        }
    }

    fun select(start: Int, end: Int = start) {
        state.textFieldState.edit { selection = TextRange(start, end) }
        // Done by a snapshotFlow in the composable
        state.onSelectionChanged(selection)
        assertConsistent()
    }

    fun assertConsistent() {
        assertTrue(
            "Text field '$text' doesn't match document '${state.document.text}'",
            text.isLooselyEqualTo(state.document.text),
        )
        assertEquals(
            "Selection doesn't match the composer's one",
            state.composerSelection(),
            TextRange(selection.min, selection.max),
        )
        // The composer's actual content must match the text field too, not only the last update
        val composerText = HtmlDocumentParser.parse(state.composerHtml().orEmpty()) { state.getPlainText() }.text
        assertTrue(
            "Text field '$text' doesn't match the composer's content '$composerText'",
            text.isLooselyEqualTo(composerText),
        )
    }

    override fun close() = state.close()
}

internal fun editorTest(initialHtml: String = "", block: EditorTester.() -> Unit) {
    EditorTester(initialHtml).use { it.apply(block) }
}
