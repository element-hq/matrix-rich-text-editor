package io.element.android.wysiwyg.inputhandlers

import android.content.Context
import android.text.Editable
import android.text.Spanned
import androidx.core.text.getSpans
import io.element.android.wysiwyg.BuildConfig
import io.element.android.wysiwyg.extensions.log
import io.element.android.wysiwyg.extensions.string
import io.element.android.wysiwyg.inputhandlers.models.EditorInputAction
import io.element.android.wysiwyg.inputhandlers.models.InlineFormat
import io.element.android.wysiwyg.inputhandlers.models.ReplaceTextResult
import io.element.android.wysiwyg.spans.ExtraCharacterSpan
import io.element.android.wysiwyg.utils.EditorIndexMapper
import io.element.android.wysiwyg.utils.HtmlToSpansParser
import uniffi.wysiwyg_composer.ComposerModelInterface
import uniffi.wysiwyg_composer.MenuState
import uniffi.wysiwyg_composer.TextUpdate
import kotlin.math.absoluteValue

internal class InputProcessor(
    private val context: Context,
    private val menuStateCallback: (MenuState) -> Unit,
    private val composer: ComposerModelInterface?,
) {

    fun updateSelection(editable: Editable, start: Int, end: Int) {
        val (newStart, newEnd) = EditorIndexMapper.fromEditorToComposer(start, end, editable) ?: return

        val update = composer?.select(newStart, newEnd)
        val menuState = update?.menuState()
        if (menuState is MenuState.Update) {
            menuStateCallback(menuState)
        }
        composer?.log()
    }

    fun processInput(action: EditorInputAction): TextUpdate? {
        val update = runCatching {
            when (action) {
                is EditorInputAction.ReplaceText -> {
                    // This conversion to a plain String might be too simple
                    composer?.replaceText(action.value.toString())
                }
                is EditorInputAction.InsertParagraph -> {
                    composer?.enter()
                }
                is EditorInputAction.BackPress -> {
                    composer?.backspace()
                }
                is EditorInputAction.ApplyInlineFormat -> {
                    when (action.format) {
                        InlineFormat.Bold -> composer?.bold()
                        InlineFormat.Italic -> composer?.italic()
                        InlineFormat.Underline -> composer?.underline()
                        InlineFormat.StrikeThrough -> composer?.strikeThrough()
                        InlineFormat.InlineCode -> composer?.inlineCode()
                    }
                }
                is EditorInputAction.Delete -> {
                    composer?.deleteIn(action.start.toUInt(), action.end.toUInt())
                }
                is EditorInputAction.SetLink -> composer?.setLink(action.link)
                is EditorInputAction.ReplaceAllHtml -> composer?.replaceAllHtml(action.html)
                is EditorInputAction.Undo -> composer?.undo()
                is EditorInputAction.Redo -> composer?.redo()
                is EditorInputAction.ToggleList -> {
                    if (action.ordered) composer?.orderedList() else composer?.unorderedList()
                }
            }
        }.onFailure {
            if (BuildConfig.DEBUG) {
                throw it
            } else {
                it.printStackTrace()
            }
        }.getOrNull()

        update?.menuState()?.let { menuStateCallback(it) }

        return update?.textUpdate().also {
            composer?.log()
        }
    }

    fun processUpdate(update: TextUpdate): ReplaceTextResult? {
        return when (update) {
            is TextUpdate.Keep -> null
            is TextUpdate.ReplaceAll -> {
                ReplaceTextResult(
                    text = stringToSpans(update.replacementHtml.string()),
                    selection = update.startUtf16Codeunit.toInt()..update.endUtf16Codeunit.toInt(),
                )
            }
            is TextUpdate.Select -> null
        }
    }

    fun getHtml(): String {
        return composer?.let { it.dumpState().html.string() }.orEmpty()
    }

    private fun stringToSpans(string: String): Spanned {
        return HtmlToSpansParser(context, string).convert()
    }
}
