/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Context.CLIPBOARD_SERVICE
import android.graphics.Canvas
import android.net.Uri
import android.os.Build
import android.os.Parcelable
import android.text.Selection
import android.text.Spanned
import android.text.TextWatcher
import android.util.AttributeSet
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.View
import android.view.inputmethod.BaseInputConnection
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputConnection
import android.widget.EditText
import androidx.annotation.VisibleForTesting
import androidx.appcompat.widget.AppCompatEditText
import androidx.core.graphics.withTranslation
import androidx.core.view.ViewCompat
import io.element.android.wysiwyg.display.MentionDisplayHandler
import io.element.android.wysiwyg.inputhandlers.InterceptInputConnection
import io.element.android.wysiwyg.internal.display.MemoizingMentionDisplayHandler
import io.element.android.wysiwyg.internal.utils.UriContentListener
import io.element.android.wysiwyg.internal.view.EditorEditTextAttributeReader
import io.element.android.wysiwyg.internal.view.viewModel
import io.element.android.wysiwyg.internal.viewmodel.ComposerResult.ReplaceText
import io.element.android.wysiwyg.internal.viewmodel.EditorInputAction
import io.element.android.wysiwyg.internal.viewmodel.EditorViewModel
import io.element.android.wysiwyg.utils.EditorIndexMapper
import io.element.android.wysiwyg.utils.HtmlConverter
import io.element.android.wysiwyg.utils.HtmlToSpansParser.FormattingSpans.removeFormattingSpans
import io.element.android.wysiwyg.utils.RustErrorCollector
import io.element.android.wysiwyg.view.StyleConfig
import io.element.android.wysiwyg.view.inlinebg.SpanBackgroundHelper
import io.element.android.wysiwyg.view.inlinebg.SpanBackgroundHelperFactory
import io.element.android.wysiwyg.view.models.InlineFormat
import io.element.android.wysiwyg.view.models.LinkAction
import io.element.android.wysiwyg.view.spans.ReuseSourceSpannableFactory
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction
import uniffi.wysiwyg_composer.MentionsState
import uniffi.wysiwyg_composer.MenuAction
import uniffi.wysiwyg_composer.newComposerModel

/**
 * An [EditText] that handles rich text editing.
 */
class EditorEditText : AppCompatEditText {
    private lateinit var textWatcher: EditorTextWatcher
    private var inputConnection: InterceptInputConnection? = null

    /**
     * The [StyleConfig] used to style the spans generated from the HTML in this EditText.
     */
    lateinit var styleConfig: StyleConfig
        private set

    private lateinit var inlineCodeBgHelper: SpanBackgroundHelper
    private lateinit var codeBlockBgHelper: SpanBackgroundHelper

    private val viewModel: EditorViewModel by viewModel(
        viewModelInitializer = {
            val provideComposer = if (!isInEditMode) { { newComposerModel() } } else { { null } }
            EditorViewModel(provideComposer)
        }
    )

    private var mentionDisplayHandler: MentionDisplayHandler? = null
        set(value) {
            field = value?.let { MemoizingMentionDisplayHandler(it) }
        }

    private var htmlConverter: HtmlConverter? = null
        set(value) {
            field = value
            viewModel.htmlConverter = value
        }

    private fun createHtmlConverter(styleConfig: StyleConfig, mentionDisplayHandler: MentionDisplayHandler?): HtmlConverter {
        return HtmlConverter.Factory.create(
            context = context.applicationContext,
            styleConfig = styleConfig,
            mentionDisplayHandler = mentionDisplayHandler,
            isEditor = true,
        )
    }

    private val spannableFactory = ReuseSourceSpannableFactory()

    private var isInitialized = false

    constructor(context: Context) : this(context, null)

    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs) {
        updateStyle(styleConfig = EditorEditTextAttributeReader(context, attrs).styleConfig, mentionDisplayHandler = mentionDisplayHandler)
    }

    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int) : super(context, attrs, defStyleAttr) {
        updateStyle(styleConfig = EditorEditTextAttributeReader(context, attrs).styleConfig, mentionDisplayHandler = mentionDisplayHandler)
    }

    init {
        setSpannableFactory(spannableFactory)
        addHardwareKeyInterceptor()

        isInitialized = true
    }

    fun interface OnSelectionChangeListener {
        fun selectionChanged(start: Int, end: Int)
    }

    fun interface OnActionStatesChangedListener {
        fun actionStatesChanged(actionStates: Map<ComposerAction, ActionState>)
    }

    fun interface OnMenuActionChangedListener {
        fun onMenuActionChanged(menuAction: MenuAction)
    }

    fun interface OnLinkActionChangedListener {
        fun onLinkActionChanged(linkAction: LinkAction?)
    }

    fun interface OnMentionsStateChangedListener {
        fun onMentionsStateChanged(mentionsState: MentionsState?)
    }

    var selectionChangeListener: OnSelectionChangeListener? = null
    var actionStatesChangedListener: OnActionStatesChangedListener? = null
        set(value) {
            field = value

            viewModel.setActionStatesCallback { actionStates ->
                value?.actionStatesChanged(actionStates)
            }
        }
    var menuActionListener: OnMenuActionChangedListener? = null
        set(value) {
            field = value

            viewModel.menuActionCallback = { menuAction ->
                value?.onMenuActionChanged(menuAction)
            }
        }

    /**
     * Set the link action listener to be notified when the available link action changes.
     */
    var linkActionChangedListener: OnLinkActionChangedListener? = null
        set(value) {
            field = value

            viewModel.linkActionCallback = { linkAction ->
                value?.onLinkActionChanged(linkAction)
            }
        }

    /**
     * Set the [MentionsState] change listener to be notified when the state of mentions in the composer changes.
     */
    var mentionsStateChangedListener: OnMentionsStateChangedListener? = null
        set(value) {
            field = value

            viewModel.setMentionsStateCallback { mentionsState ->
                value?.onMentionsStateChanged(mentionsState)
            }
        }

    /**
     * When not null, it will serve as an error callback for the client integrating this lib.
     */
    var rustErrorCollector: RustErrorCollector?
        set(value) {
            viewModel.rustErrorCollector = value
        }
        get() = viewModel.rustErrorCollector

    /**
     * We'll do our own text restoration.
     */
    override fun getFreezesText(): Boolean {
        return false
    }

    override fun onRestoreInstanceState(state: Parcelable?) {
        val spannedText = viewModel.getCurrentFormattedText()
        editableText.replace(0, editableText.length, spannedText)

        super.onRestoreInstanceState(state)

        val start = Selection.getSelectionStart(editableText)
        val end = Selection.getSelectionEnd(editableText)

        viewModel.updateSelection(editableText, start, end)
    }

    override fun onSelectionChanged(selStart: Int, selEnd: Int) {
        super.onSelectionChanged(selStart, selEnd)
        if (this.isInitialized && !this.textWatcher.isInEditorChange) {
            this.viewModel.updateSelection(editableText, selStart, selEnd)
        }
        selectionChangeListener?.selectionChanged(selStart, selEnd)
    }

    /**
     * We wrap the internal `com.android.internal.widget.EditableInputConnection` as it's not
     * public, but its internal behavior is needed to work properly with the EditText.
     */
    override fun onCreateInputConnection(outAttrs: EditorInfo): InputConnection {
        val baseInputConnection = requireNotNull(super.onCreateInputConnection(outAttrs))
        val inputConnection =
            InterceptInputConnection(baseInputConnection, this, viewModel, textWatcher)
        this.inputConnection = inputConnection
        return inputConnection
    }

    /**
     * Override context menu actions, such as cut & paste so its input is redirected to the [viewModel].
     */
    override fun onTextContextMenuItem(id: Int): Boolean {
        when (id) {
            android.R.id.cut -> {
                val clipboardManager =
                    context.getSystemService(CLIPBOARD_SERVICE) as ClipboardManager
                val clpData = ClipData.newPlainText(
                    "newText",
                    this.editableText.slice(this.selectionStart until this.selectionEnd)
                )
                clipboardManager.setPrimaryClip(clpData)

                val result = viewModel.processInput(
                    EditorInputAction.DeleteIn(
                        this.selectionStart,
                        this.selectionEnd
                    )
                )

                if (result is ReplaceText) {
                    setTextFromComposerUpdate(result.text)
                    setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
                }

                return true
            }
            android.R.id.paste, android.R.id.pasteAsPlainText -> {
                val clipBoardManager =
                    context.getSystemService(CLIPBOARD_SERVICE) as ClipboardManager
                // Only special-case paste behaviour if it is text content, otherwise default to EditText implementation
                // which calls ViewCompat.performReceiveContent and fires the expected listeners.
                val copiedString = clipBoardManager.primaryClip?.getItemAt(0)?.text
                    ?: return super.onTextContextMenuItem(id)
                val result = viewModel.processInput(EditorInputAction.ReplaceText(copiedString))

                if (result is ReplaceText) {
                    setTextFromComposerUpdate(result.text)
                    setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
                }

                return true
            }
            android.R.id.undo -> {
                undo()
                return true
            }
            android.R.id.redo -> {
                redo()
                return true
            }
            else -> return super.onTextContextMenuItem(id)
        }
    }

    /**
     * Sets up the styling used to translate HTML to Spanned text.
     * @param styleConfig The styles to use for the generated spans.
     * @param mentionDisplayHandler Used to decide how to display any mentions found in the HTML text.
     */
    fun updateStyle(styleConfig: StyleConfig, mentionDisplayHandler: MentionDisplayHandler?) {
        var forceNewRendering = false
        val hasNewStyle = !this::styleConfig.isInitialized || this.styleConfig != styleConfig
        val hasNewMentionDisplayHandler = (this.mentionDisplayHandler as? MemoizingMentionDisplayHandler)
            ?.delegateEquals(mentionDisplayHandler) != true
        if (hasNewStyle || hasNewMentionDisplayHandler) {
            this.styleConfig = styleConfig
            this.mentionDisplayHandler = mentionDisplayHandler
            forceNewRendering = true
        }

        inlineCodeBgHelper = SpanBackgroundHelperFactory.createInlineCodeBackgroundHelper(styleConfig.inlineCode)
        codeBlockBgHelper = SpanBackgroundHelperFactory.createCodeBlockBackgroundHelper(styleConfig.codeBlock)

        htmlConverter = createHtmlConverter(styleConfig, mentionDisplayHandler)
        if (forceNewRendering) {
            rerender()
        }
    }

    fun setOnRichContentSelected(onRichContentSelected: ((Uri) -> Unit)?) {
        if (onRichContentSelected != null) {
            ViewCompat.setOnReceiveContentListener(
                this,
                arrayOf("image/*"),
                UriContentListener { onRichContentSelected(it) }
            )
        }
    }

    private fun addHardwareKeyInterceptor() {
        // This seems to be the only way to prevent EditText from automatically handling key strokes
        setOnKeyListener { _, keyCode, event ->
            when {
                keyCode == KeyEvent.KEYCODE_ENTER -> {
                    if (event.action == MotionEvent.ACTION_DOWN) {
                        inputConnection?.sendHardwareKeyboardInput(event)
                    }
                    true
                }
                event.action != MotionEvent.ACTION_DOWN -> false
                event.isMovementKey() -> false
                event.keyCode == KeyEvent.KEYCODE_Z && event.isCtrlPressed && event.isShiftPressed -> {
                    redo()
                    true
                }
                event.keyCode == KeyEvent.KEYCODE_Z && event.isCtrlPressed -> {
                    undo()
                    true
                }
                event.metaState != 0 && event.unicodeChar == 0 -> {
                    // Is a modifier key
                    false
                }
                event.isPrintableCharacter() ||
                    keyCode == KeyEvent.KEYCODE_DEL ||
                    keyCode == KeyEvent.KEYCODE_FORWARD_DEL -> {
                    // Consume printable characters
                    inputConnection?.sendHardwareKeyboardInput(event)
                    true
                }
                else -> {
                    // Don't consume other key codes (HW back button, i.e.)
                    false
                }
            }
        }
    }

    override fun setText(text: CharSequence?, type: BufferType?) {
        val currentText = this.text
        val end = currentText?.length ?: 0
        // Although inputProcessor is assured to be not null, that's not the case while inflating.
        // We have to add this here to prevent some NullPointerExceptions from being thrown.
        if (!this.isInitialized) {
            return super.setText(text, type)
        }

        inlineCodeBgHelper.clearCachedPositions()
        codeBlockBgHelper.clearCachedPositions()

        viewModel.updateSelection(editableText, 0, end)

        val result = processInputAsTextResult(EditorInputAction.ReplaceText(text.toString()))
            ?: return super.setText(text, type)

        setTextFromComposerUpdate(result.text)
        setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
    }

    override fun append(text: CharSequence?, start: Int, end: Int) {
        val result = processInputAsTextResult(EditorInputAction.ReplaceText(text.toString()))
            ?: return super.append(text, start, end)

        setTextFromComposerUpdate(result.text)
        setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
    }

    fun toggleInlineFormat(inlineFormat: InlineFormat): Boolean {
        val result = processInputAsTextResult(EditorInputAction.ApplyInlineFormat(inlineFormat))
            ?: return false

        setTextFromComposerUpdate(result.text)
        setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
        return true
    }

    fun toggleCodeBlock(): Boolean {
        val result = processInputAsTextResult(EditorInputAction.CodeBlock) ?: return false
        setTextFromComposerUpdate(result.text)
        setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
        return true
    }

    fun toggleQuote(): Boolean {
        val result = processInputAsTextResult(EditorInputAction.Quote) ?: return false
        setTextFromComposerUpdate(result.text)
        setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
        return true
    }

    fun undo() {
        val result = processInputAsTextResult(EditorInputAction.Undo) ?: return

        setTextFromComposerUpdate(result.text)
        setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
    }

    fun redo() {
        val result = processInputAsTextResult(EditorInputAction.Redo) ?: return

        setTextFromComposerUpdate(result.text)
        setSelectionFromComposerUpdate(result.selection.last)
    }

    /**
     * Get the action that can be performed based on the current link
     * and selection state.
     *
     * Based on this the caller can decide whether to call [setLink],
     * [removeLink], or [insertLink].
     *
     * @return The link action or null if no action is available.
     */
    fun getLinkAction(): LinkAction? =
        viewModel.getLinkAction()

    /**
     * Set a link for the current selection. This method does nothing if there is no text selected.
     *
     * @param url The link URL to set or null to remove
     */
    fun setLink(url: String?) {
        val result = processInputAsTextResult(
            if (url != null) EditorInputAction.SetLink(url) else EditorInputAction.RemoveLink
        ) ?: return

        setTextFromComposerUpdate(result.text)
        setSelectionFromComposerUpdate(result.selection.last)
    }

    /**
     * Remove a link for the current selection. Convenience for setLink(null).
     *
     * @see [setLink]
     */
    fun removeLink() = setLink(null)

    /**
     * Insert new text with a link.
     *
     * @param url The link URL to set
     * @param text The new text to insert
     */
    fun insertLink(url: String, text: String) {
        val result = processInputAsTextResult(EditorInputAction.SetLinkWithText(url, text)) ?: return

        setTextFromComposerUpdate(result.text)
        setSelectionFromComposerUpdate(result.selection.last)
    }

    fun toggleList(ordered: Boolean) {
        val result = processInputAsTextResult(EditorInputAction.ToggleList(ordered)) ?: return

        setTextFromComposerUpdate(result.text)
        setSelectionFromComposerUpdate(result.selection.last)
    }

    fun indent() {
        val result = processInputAsTextResult(EditorInputAction.Indent) ?: return

        setTextFromComposerUpdate(result.text)
        setSelectionFromComposerUpdate(result.selection.last)
    }

    fun unindent() {
        val result = processInputAsTextResult(EditorInputAction.Unindent) ?: return

        setTextFromComposerUpdate(result.text)
        setSelectionFromComposerUpdate(result.selection.last)
    }

    fun setHtml(html: String) {
        val result = processInputAsTextResult(EditorInputAction.ReplaceAllHtml(html)) ?: return

        setTextFromComposerUpdate(result.text)
        setSelectionFromComposerUpdate(result.selection.last)
    }

    /**
     * Get the editor content as clean HTML suitable for sending as a message.
     */
    fun getContentAsMessageHtml(): String {
        return viewModel.getContentAsMessageHtml()
    }

    fun getInternalHtml(): String {
        return viewModel.getInternalHtml()
    }

    /**
     * Get the text as markdown.
     */
    fun getMarkdown(): String = viewModel.getMarkdown()

    /**
     * Get the editor content as clean markdown suitable for sending as a message.
     */
    fun getContentAsMessageMarkdown(): String {
        return viewModel.getContentAsMessageMarkdown()
    }

    /**
     * Set the text as markdown, it will be turned into to HTML internally.
     */
    fun setMarkdown(markdown: String) {
        val result = processInputAsTextResult(EditorInputAction.ReplaceAllMarkdown(markdown)) ?: return
        setTextFromComposerUpdate(result.text)
        setSelectionFromComposerUpdate(result.selection.last)
    }

    /**
     * Set a mention link that applies to the current suggestion range
     *
     * @param url The url of the new link
     * @param text The text to insert into the current suggestion range
     */
    fun insertMentionAtSuggestion(url: String, text: String) {
        val result = processInputAsTextResult(
            EditorInputAction.InsertMentionAtSuggestion(
                text = text,
                url = url,
            )
        ) ?: return
        setTextFromComposerUpdate(result.text)
        setSelectionFromComposerUpdate(result.selection.last)
    }

    /**
     * Set a mention link that applies to the current suggestion range
     */
    fun insertAtRoomMentionAtSuggestion() {
        val result = processInputAsTextResult(
            EditorInputAction.InsertAtRoomMentionAtSuggestion
        ) ?: return
        setTextFromComposerUpdate(result.text)
        setSelectionFromComposerUpdate(result.selection.last)
    }

    /**
     * Replace text in the current suggestion range
     *
     * @param text The text to insert into the current suggestion range
     */
    fun replaceTextSuggestion(text: String) {
        val result = processInputAsTextResult(
            EditorInputAction.ReplaceTextSuggestion(
                value = text,
            )
        ) ?: return
        setTextFromComposerUpdate(result.text)
        setSelectionFromComposerUpdate(result.selection.last)
    }

    /**
     * Get the current [MentionsState] of the editor.
     */
    fun getMentionsState(): MentionsState? {
        return viewModel.getMentionsState()
    }

    @VisibleForTesting
    internal fun testComposerCrashRecovery() =
        viewModel.testComposerCrashRecovery()

    override fun onDraw(canvas: Canvas) {
        // need to draw bg first so that text can be on top during super.onDraw()
        if (text is Spanned && layout != null) {
            canvas.withTranslation(totalPaddingLeft.toFloat(), totalPaddingTop.toFloat()) {
                inlineCodeBgHelper.draw(canvas, text as Spanned, layout)
                codeBlockBgHelper.draw(canvas, text as Spanned, layout)
            }
        }
        super.onDraw(canvas)
    }

    override fun setSelection(start: Int, stop: Int) {
        if (editableText.isEmpty()) return
        super.setSelection(
            start.coerceIn(0, editableText.length),
            stop.coerceIn(0, editableText.length)
        )
    }

    override fun addTextChangedListener(watcher: TextWatcher) {
        if (!this::textWatcher.isInitialized) {
            this.textWatcher = EditorTextWatcher()
            super.addTextChangedListener(this.textWatcher)
        }
        textWatcher.addChild(watcher)
    }

    override fun removeTextChangedListener(watcher: TextWatcher) {
        textWatcher.removeChild(watcher)
    }

    // This workaround is needed around compose to prevent the EditText focus from causing ANRs
    override fun focusSearch(direction: Int): View? {
        return null
    }

    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) {
        // The size changed, so the cached positions for the code renderers won't match anymore
        inlineCodeBgHelper.clearCachedPositions()
        codeBlockBgHelper.clearCachedPositions()

        super.onSizeChanged(w, h, oldw, oldh)
    }

    /**
     * Force redisplay the current editor model.
     *
     * If the style of the content should change, the content of the EditText
     * will be updated to reflect this.
     */
    private fun rerender() {
        val compositionStart = BaseInputConnection.getComposingSpanStart(editableText)
        val compositionEnd = BaseInputConnection.getComposingSpanEnd(editableText)
        val text = viewModel.rerender()
        setTextFromComposerUpdate(text)
        val indices = text.indices
        if (compositionStart in indices && compositionEnd in indices) {
            inputConnection?.setComposingRegion(compositionStart, compositionEnd)
        }
    }

    private fun setTextFromComposerUpdate(text: CharSequence) {
        beginBatchEdit()
        textWatcher.runInEditor {
            val beforeLength = editableText.length
            notifyBeforeTextChanged(editableText, 0, beforeLength, text.length)
            editableText.removeFormattingSpans()
            editableText.replace(0, editableText.length, text)
            notifyOnTextChanged(editableText, 0, beforeLength, text.length)
            notifyAfterTextChanged(editableText)
        }
        endBatchEdit()
    }

    private fun setSelectionFromComposerUpdate(start: Int, end: Int = start) {
        val (newStart, newEnd) = EditorIndexMapper.fromComposerToEditor(start, end, editableText)
        if (newStart in 0..editableText.length && newEnd in 0..editableText.length) {
            setSelection(newStart, newEnd)
        }
    }

    private fun EditorEditText.processInputAsTextResult(action: EditorInputAction): ReplaceText? {
        val result = viewModel.processInput(action)
        return result as? ReplaceText
    }
}

private fun KeyEvent.isMovementKey(): Boolean {
    val baseIsMovement = this.keyCode in KeyEvent.KEYCODE_DPAD_UP..KeyEvent.KEYCODE_DPAD_CENTER
    val api24IsMovement = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
        this.keyCode in KeyEvent.KEYCODE_DPAD_UP_LEFT..KeyEvent.KEYCODE_DPAD_DOWN_RIGHT
    } else false
    return baseIsMovement || api24IsMovement
}

private fun KeyEvent.isPrintableCharacter(): Boolean {
    return isPrintingKey || keyCode == KeyEvent.KEYCODE_SPACE
}