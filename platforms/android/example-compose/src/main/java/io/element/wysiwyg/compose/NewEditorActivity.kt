/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.wysiwyg.compose

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import io.element.android.wysiwyg.compose.next.MentionDisplay
import io.element.android.wysiwyg.compose.next.RichTextEditor
import io.element.android.wysiwyg.compose.next.RichTextEditorDefaults
import io.element.android.wysiwyg.compose.next.rememberRichTextEditorState
import io.element.android.wysiwyg.view.models.InlineFormat
import io.element.android.wysiwyg.view.models.LinkAction
import io.element.wysiwyg.compose.matrix.Mention
import io.element.wysiwyg.compose.ui.components.FormattingButtons
import io.element.wysiwyg.compose.ui.theme.RichTextEditorTheme
import kotlinx.collections.immutable.toPersistentMap
import timber.log.Timber
import uniffi.wysiwyg_composer.ComposerAction

/**
 * Example of the editor from `library-compose-new`, which uses a Compose `BasicTextField`
 * instead of wrapping an `EditText`.
 */
class NewEditorActivity : ComponentActivity() {

    private val roomMemberSuggestions = mutableStateListOf<Mention>()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            RichTextEditorTheme {
                val state = rememberRichTextEditorState()
                state.onError = { Timber.e(it) }

                LaunchedEffect(state.menuAction) {
                    processMenuAction(state.menuAction, roomMemberSuggestions)
                }

                var linkDialogAction by remember { mutableStateOf<LinkAction?>(null) }
                linkDialogAction?.let { linkAction ->
                    LinkDialog(
                        linkAction = linkAction,
                        onRemoveLink = { state.removeLink() },
                        onSetLink = { state.setLink(it) },
                        onInsertLink = { url, text -> state.insertLink(url, text) },
                        onDismissRequest = { linkDialogAction = null },
                    )
                }

                Scaffold(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(MaterialTheme.colorScheme.background)
                        .imePadding(),
                ) { paddingValues ->
                    Column(
                        modifier = Modifier
                            .fillMaxSize()
                            .padding(paddingValues),
                        verticalArrangement = Arrangement.SpaceBetween,
                    ) {
                        Surface(
                            modifier = Modifier
                                .padding(8.dp)
                                .border(BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant))
                                .padding(8.dp),
                            color = MaterialTheme.colorScheme.surface,
                        ) {
                            RichTextEditor(
                                state = state,
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .padding(10.dp)
                                    .testTag("rich-text-editor"),
                                style = RichTextEditorDefaults.style(
                                    textStyle = MaterialTheme.typography.bodyLarge.copy(
                                        color = MaterialTheme.colorScheme.onSurface,
                                    ),
                                ),
                                placeholder = "Type your message here...",
                                maxLines = 10,
                                resolveMentionDisplay = { mention ->
                                    if (mention.isAtRoom) {
                                        MentionDisplay.Pill("@room")
                                    } else {
                                        MentionDisplay.Pill(mention.text)
                                    }
                                },
                            )
                        }

                        Text(
                            text = state.internalHtml,
                            style = MaterialTheme.typography.bodySmall,
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(16.dp),
                        )

                        Spacer(modifier = Modifier.weight(1f))
                        SuggestionView(
                            modifier = Modifier.heightIn(max = 320.dp),
                            roomMemberSuggestions = roomMemberSuggestions,
                            onReplaceSuggestion = { text -> state.replaceSuggestion(text) },
                            onInsertAtRoomMentionAtSuggestion = { state.insertAtRoomMentionAtSuggestion() },
                            onInsertMentionAtSuggestion = { text, link -> state.insertMentionAtSuggestion(text, link) },
                        )

                        FormattingButtons(
                            onResetText = { state.clear() },
                            actionStates = state.actions.toPersistentMap(),
                            onActionClick = {
                                when (it) {
                                    ComposerAction.BOLD -> state.toggleInlineFormat(InlineFormat.Bold)
                                    ComposerAction.ITALIC -> state.toggleInlineFormat(InlineFormat.Italic)
                                    ComposerAction.STRIKE_THROUGH -> state.toggleInlineFormat(InlineFormat.StrikeThrough)
                                    ComposerAction.UNDERLINE -> state.toggleInlineFormat(InlineFormat.Underline)
                                    ComposerAction.INLINE_CODE -> state.toggleInlineFormat(InlineFormat.InlineCode)
                                    ComposerAction.LINK -> linkDialogAction = state.linkAction
                                    ComposerAction.UNDO -> state.undo()
                                    ComposerAction.REDO -> state.redo()
                                    ComposerAction.ORDERED_LIST -> state.toggleList(ordered = true)
                                    ComposerAction.UNORDERED_LIST -> state.toggleList(ordered = false)
                                    ComposerAction.INDENT -> state.indent()
                                    ComposerAction.UNINDENT -> state.unindent()
                                    ComposerAction.CODE_BLOCK -> state.toggleCodeBlock()
                                    ComposerAction.QUOTE -> state.toggleQuote()
                                }
                            },
                        )
                    }
                }
            }
        }
    }
}
