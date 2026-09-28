/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next

import android.os.SystemClock
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.waitForUpOrCancellation
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.graphics.drawscope.translate
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEvent
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isCtrlPressed
import androidx.compose.ui.input.key.isMetaPressed
import androidx.compose.ui.input.key.isShiftPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.InterceptPlatformTextInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.TextLayoutResult
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.rememberTextMeasurer
import io.element.android.wysiwyg.compose.next.internal.BackspaceAtStartInterceptor
import io.element.android.wysiwyg.compose.next.internal.render.RenderPlanBuilder
import io.element.android.wysiwyg.compose.next.internal.render.RichTextOutputTransformation
import io.element.android.wysiwyg.compose.next.internal.render.correctTappedCursor
import io.element.android.wysiwyg.compose.next.internal.render.drawDecorations
import io.element.android.wysiwyg.view.models.InlineFormat

/**
 * A rich text editor built on top of [BasicTextField].
 *
 * The text input from the IME and the keyboard is sent to the Rust composer in [state], and its
 * resulting content is rendered back in the text field.
 *
 * @param state the state of the editor, see [rememberRichTextEditorState].
 * @param modifier the modifier to apply to the text field.
 * @param style the style used to render the content.
 * @param placeholder text to display when the editor is empty.
 * @param enabled whether the editor can be focused and edited.
 * @param minLines the minimum height of the editor in lines.
 * @param maxLines the maximum height of the editor in lines, the content will scroll after it.
 * @param keyboardOptions options for the software keyboard.
 * @param interactionSource the interaction source of the text field.
 * @param resolveMentionDisplay decides how each mention is displayed.
 */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun RichTextEditor(
    state: RichTextEditorState,
    modifier: Modifier = Modifier,
    style: RichTextEditorStyle = RichTextEditorDefaults.style(),
    placeholder: String? = null,
    enabled: Boolean = true,
    minLines: Int = 1,
    maxLines: Int = Int.MAX_VALUE,
    keyboardOptions: KeyboardOptions = KeyboardOptions(capitalization = KeyboardCapitalization.Sentences),
    interactionSource: MutableInteractionSource? = null,
    resolveMentionDisplay: (Mention) -> MentionDisplay = { MentionDisplay.Pill(it.text) },
) {
    val density = LocalDensity.current
    val currentResolveMentionDisplay by rememberUpdatedState(resolveMentionDisplay)
    val renderPlan = remember(state, style, density) {
        val builder = RenderPlanBuilder(style, density) { currentResolveMentionDisplay(it) }
        derivedStateOf { builder.build(state.document) }
    }
    val outputTransformation = remember(renderPlan) {
        RichTextOutputTransformation { renderPlan.value }
    }
    val textMeasurer = rememberTextMeasurer()
    val scrollState = rememberScrollState()
    val textLayout = remember { TextLayoutHolder() }
    val bulletRadius = with(density) { style.list.bulletRadius.toPx() }

    val tapTracker = remember { TapTracker() }

    LaunchedEffect(state) {
        snapshotFlow { state.textFieldState.selection }
            .collect { selection ->
                val correctedCursor = tapTracker.consume()?.let { tapY ->
                    val layout = textLayout.getResult?.invoke()
                    if (layout == null || !selection.collapsed) return@let null
                    renderPlan.value.correctTappedCursor(selection.start, tapY + scrollState.value, layout)
                }
                if (correctedCursor != null) {
                    // This will trigger a new selection change
                    state.textFieldState.edit { this.selection = TextRange(correctedCursor) }
                } else {
                    state.onSelectionChanged(selection)
                }
            }
    }
    LaunchedEffect(state) {
        // Undo is handled by the composer, the text field history would get out of sync with it
        snapshotFlow { state.textFieldState.text.toString() }
            .collect { state.textFieldState.undoState.clearHistory() }
    }

    val backspaceInterceptor = remember(state) { BackspaceAtStartInterceptor(state::onBackspaceAtStart) }
    InterceptPlatformTextInput(backspaceInterceptor) {
        BasicTextField(
            state = state.textFieldState,
            modifier = modifier
                .pointerInput(Unit) {
                    // Only observe taps, the text field handles them
                    awaitEachGesture {
                        val down = awaitFirstDown(requireUnconsumed = false, pass = PointerEventPass.Initial)
                        val up = waitForUpOrCancellation(pass = PointerEventPass.Initial)
                        if (up != null && (up.position - down.position).getDistance() < viewConfiguration.touchSlop) {
                            tapTracker.onTap(up.position.y)
                        }
                    }
                }
                .onPreviewKeyEvent { state.handleShortcut(it) },
            enabled = enabled,
            inputTransformation = state.inputTransformation,
            textStyle = style.textStyle,
            keyboardOptions = keyboardOptions,
            lineLimits = TextFieldLineLimits.MultiLine(minHeightInLines = minLines, maxHeightInLines = maxLines),
            onTextLayout = { getResult -> textLayout.getResult = getResult },
            interactionSource = interactionSource,
            cursorBrush = style.cursorBrush,
            outputTransformation = outputTransformation,
            scrollState = scrollState,
            decorator = { innerTextField ->
                Box(
                    modifier = Modifier.drawBehind {
                        val layout = textLayout.getResult?.invoke() ?: return@drawBehind
                        clipRect {
                            translate(top = -scrollState.value.toFloat()) {
                                drawDecorations(renderPlan.value, layout, textMeasurer, style.textStyle, bulletRadius)
                            }
                        }
                    }
                ) {
                    // An empty list item or quote is not empty, it has a marker or decoration
                    val showPlaceholder = state.isEmpty && state.document.lines.single().indents.isEmpty()
                    if (placeholder != null && showPlaceholder) {
                        BasicText(
                            text = placeholder,
                            style = style.textStyle.copy(color = style.placeholderColor),
                            maxLines = 1,
                        )
                    }
                    innerTextField()
                }
            },
        )
    }
}

/**
 * Keeps the position of the last tap, so the cursor placed by the text field can be corrected.
 */
private class TapTracker {
    private var tapY: Float? = null
    private var time = 0L

    fun onTap(y: Float) {
        tapY = y
        time = SystemClock.uptimeMillis()
    }

    /** Returns the vertical position of the last tap, if it just happened. */
    fun consume(): Float? {
        val result = tapY?.takeIf { SystemClock.uptimeMillis() - time < 500 }
        tapY = null
        return result
    }
}

private class TextLayoutHolder {
    var getResult: (() -> TextLayoutResult?)? = null
}

private fun RichTextEditorState.handleShortcut(event: KeyEvent): Boolean {
    if (event.type != KeyEventType.KeyDown) return false
    if (event.key == Key.Backspace) return onBackspaceAtStart()
    if (!(event.isCtrlPressed || event.isMetaPressed)) return false
    when (event.key) {
        Key.Z -> if (event.isShiftPressed) redo() else undo()
        Key.Y -> redo()
        Key.B -> toggleInlineFormat(InlineFormat.Bold)
        Key.I -> toggleInlineFormat(InlineFormat.Italic)
        Key.U -> toggleInlineFormat(InlineFormat.Underline)
        else -> return false
    }
    return true
}
