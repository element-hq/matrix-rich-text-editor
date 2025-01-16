/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.wysiwyg.compose

import android.os.Bundle
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import io.element.android.wysiwyg.compose.EditorStyledText
import io.element.android.wysiwyg.compose.RichTextEditor
import io.element.android.wysiwyg.compose.RichTextEditorDefaults
import io.element.android.wysiwyg.compose.StyledHtmlConverter
import io.element.android.wysiwyg.compose.rememberRichTextEditorState
import io.element.android.wysiwyg.display.TextDisplay
import io.element.android.wysiwyg.view.models.InlineFormat
import io.element.android.wysiwyg.view.models.LinkAction
import io.element.wysiwyg.compose.matrix.Mention
import io.element.wysiwyg.compose.ui.components.FormattingButtons
import io.element.wysiwyg.compose.ui.theme.RichTextEditorTheme
import kotlinx.collections.immutable.toPersistentMap
import kotlinx.coroutines.launch
import timber.log.Timber
import uniffi.wysiwyg_composer.ComposerAction
import uniffi.wysiwyg_composer.newMentionDetector

class MainActivity : ComponentActivity() {

    private val roomMemberSuggestions = mutableStateListOf<Mention>()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val mentionDisplayHandler = DefaultMentionDisplayHandler()
        val mentionDetector = if (window.decorView.isInEditMode) null else newMentionDetector()
        setContent {
            val context = LocalContext.current
            RichTextEditorTheme {
                val style = RichTextEditorDefaults.style()
                val htmlConverter = remember(style) {
                    StyledHtmlConverter(
                        context = context,
                        mentionDisplayHandler = mentionDisplayHandler,
                        isEditor = false,
                        isMention = mentionDetector?.let { detector ->
                            { _, url ->
                                detector.isMention(url)
                            }
                        }
                    ).apply {
                        configureWith(style = style)
                    }
                }

                val state = rememberRichTextEditorState(initialFocus = true)

                LaunchedEffect(state.menuAction) {
                    processMenuAction(state.menuAction, roomMemberSuggestions)
                }

                var linkDialogAction by remember { mutableStateOf<LinkAction?>(null) }
                val coroutineScope = rememberCoroutineScope()

                LaunchedEffect(state.messageHtml) {
                    Timber.d("Message HTML: '${state.messageHtml}'")
                }
                val htmlText = htmlConverter.fromHtmlToSpans(state.messageHtml)

                linkDialogAction?.let { linkAction ->
                    LinkDialog(linkAction = linkAction,
                        onRemoveLink = { coroutineScope.launch { state.removeLink() } },
                        onSetLink = { coroutineScope.launch { state.setLink(it) } },
                        onInsertLink = { url, text ->
                            coroutineScope.launch {
                                state.insertLink(
                                    url, text
                                )
                            }
                        },
                        onDismissRequest = { linkDialogAction = null })
                }
                Surface(
                    modifier = Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background
                ) {
                    Column(
                        modifier = Modifier.fillMaxSize(),
                        verticalArrangement = Arrangement.SpaceBetween
                    ) {
                        var isTyping by remember { mutableStateOf(false) }
                        Surface(
                            modifier = Modifier
                                .padding(8.dp)
                                .border(
                                    border = BorderStroke(
                                        1.dp, MaterialTheme.colorScheme.outlineVariant
                                    ),
                                )
                                .padding(8.dp),
                            color = MaterialTheme.colorScheme.surface,
                        ) {
                            RichTextEditor(
                                state = state,
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .padding(10.dp),
                                style = RichTextEditorDefaults.style(),
                                onError = { Timber.e(it) },
                                resolveMentionDisplay = { _,_ -> TextDisplay.Pill },
                                resolveRoomMentionDisplay = { TextDisplay.Pill },
                                onTyping = { isTyping = it }
                            )
                        }
                        if (isTyping) {
                            Text(
                                text = "Typing...",
                                style = MaterialTheme.typography.labelSmall,
                                modifier = Modifier
                                    .height(32.dp)
                                    .padding(horizontal = 8.dp)
                            )
                        } else {
                            Spacer(Modifier.height(32.dp))
                        }
                        EditorStyledText(
                            text = htmlText,
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(16.dp),
                            resolveMentionDisplay = { _,_ -> TextDisplay.Pill },
                            resolveRoomMentionDisplay = { TextDisplay.Pill },
                            onLinkClickedListener = { url ->
                                Toast.makeText(this@MainActivity, "Clicked: $url", Toast.LENGTH_SHORT).show()
                            }
                        )

                        Spacer(modifier = Modifier.weight(1f))
                        SuggestionView(
                            modifier = Modifier.heightIn(max = 320.dp),
                            roomMemberSuggestions = roomMemberSuggestions,
                            onReplaceSuggestion = { text ->
                                coroutineScope.launch {
                                    state.replaceSuggestion(text)
                                }
                            },
                            onInsertAtRoomMentionAtSuggestion = {
                                coroutineScope.launch {
                                    state.insertAtRoomMentionAtSuggestion()
                                }
                            },
                            onInsertMentionAtSuggestion = { text, link ->
                                coroutineScope.launch {
                                    state.insertMentionAtSuggestion(text, link)
                                }
                            },
                        )

                        FormattingButtons(onResetText = {
                            coroutineScope.launch {
                                state.setHtml("")
                            }
                        }, actionStates = state.actions.toPersistentMap(), onActionClick = {
                            coroutineScope.launch {
                                when (it) {
                                    ComposerAction.BOLD -> state.toggleInlineFormat(
                                        InlineFormat.Bold
                                    )

                                    ComposerAction.ITALIC -> state.toggleInlineFormat(
                                        InlineFormat.Italic
                                    )

                                    ComposerAction.STRIKE_THROUGH -> state.toggleInlineFormat(
                                        InlineFormat.StrikeThrough
                                    )

                                    ComposerAction.UNDERLINE -> state.toggleInlineFormat(
                                        InlineFormat.Underline
                                    )

                                    ComposerAction.INLINE_CODE -> state.toggleInlineFormat(
                                        InlineFormat.InlineCode
                                    )

                                    ComposerAction.LINK -> linkDialogAction = state.linkAction

                                    ComposerAction.UNDO -> state.undo()
                                    ComposerAction.REDO -> state.redo()
                                    ComposerAction.ORDERED_LIST -> state.toggleList(ordered = true)
                                    ComposerAction.UNORDERED_LIST -> state.toggleList(
                                        ordered = false
                                    )

                                    ComposerAction.INDENT -> state.indent()
                                    ComposerAction.UNINDENT -> state.unindent()
                                    ComposerAction.CODE_BLOCK -> state.toggleCodeBlock()
                                    ComposerAction.QUOTE -> state.toggleQuote()
                                }
                            }
                        })
                    }
                }
            }
        }
    }
}

@Preview
@Composable
fun EditorPreview() {
    RichTextEditorTheme {
        val state = rememberRichTextEditorState("Hello, world")
        RichTextEditor(
            state = state,
            modifier = Modifier.fillMaxWidth(),
        )
    }
}

