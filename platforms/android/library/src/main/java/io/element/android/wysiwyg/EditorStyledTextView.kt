/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg

import android.content.Context
import android.graphics.Canvas
import android.text.Layout
import android.text.Spanned
import android.text.style.URLSpan
import android.util.AttributeSet
import android.view.GestureDetector
import android.view.MotionEvent
import androidx.appcompat.widget.AppCompatTextView
import androidx.core.graphics.withTranslation
import androidx.core.text.getSpans
import androidx.core.view.GestureDetectorCompat
import com.sun.jna.internal.Cleaner
import io.element.android.wysiwyg.display.MentionDisplayHandler
import io.element.android.wysiwyg.internal.view.EditorEditTextAttributeReader
import io.element.android.wysiwyg.utils.HtmlConverter
import io.element.android.wysiwyg.utils.RustCleanerTask
import io.element.android.wysiwyg.view.StyleConfig
import io.element.android.wysiwyg.view.inlinebg.SpanBackgroundHelper
import io.element.android.wysiwyg.view.inlinebg.SpanBackgroundHelperFactory
import io.element.android.wysiwyg.view.spans.CustomMentionSpan
import io.element.android.wysiwyg.view.spans.PillSpan
import io.element.android.wysiwyg.view.spans.ReuseSourceSpannableFactory
import uniffi.wysiwyg_composer.MentionDetector
import uniffi.wysiwyg_composer.newMentionDetector

/**
 * This TextView can display all spans used by the editor.
 */
open class EditorStyledTextView : AppCompatTextView {

    // Used to automatically clean up the native resources when this instance is GCed
    private val cleaner = Cleaner.getCleaner()

    private val mentionDetector: MentionDetector? by lazy {
        if (!isInEditMode && isNativeCodeEnabled) {
            val detector = newMentionDetector()
            cleaner.register(this, RustCleanerTask(detector))
            detector
        } else {
            null
        }
    }

    private lateinit var inlineCodeBgHelper: SpanBackgroundHelper
    private lateinit var codeBlockBgHelper: SpanBackgroundHelper

    /**
     * The [StyleConfig] used to style the spans generated from the HTML in this TextView.
     */
    lateinit var styleConfig: StyleConfig
        private set

    private var isInit = false

    private val spannableFactory = ReuseSourceSpannableFactory()

    var mentionDisplayHandler: MentionDisplayHandler? = null
    private var htmlConverter: HtmlConverter? = null

    var onLinkClickedListener: ((String) -> Unit)? = null
    var onLinkLongClickedListener: ((String) -> Unit)? = null

    var onTextLayout: ((Layout) -> Unit)? = null

    /**
     * In some contexts, such as screenshot tests, [isInEditMode] is may be forced to be false, when we
     * need it to be true to disable native library loading. With this we can override this behaviour.
     */
    var isNativeCodeEnabled: Boolean = !isInEditMode

    // This gesture detector will be used to detect clicks on spans
    private val gestureDetector =
        GestureDetectorCompat(context, object : GestureDetector.SimpleOnGestureListener() {

            private fun hasAnyLinkListener() =
                onLinkClickedListener != null || onLinkLongClickedListener != null

            private fun handleLinkClicks(
                motionEvent: MotionEvent, listener: (String) -> Unit
            ): Boolean {
                val spans = findSpansForTouchEvent(motionEvent)
                for (span in spans) {
                    when (span) {
                        is URLSpan -> {
                            listener(span.url)
                            return true
                        }

                        is PillSpan -> {
                            span.url?.let(listener)
                            return true
                        }

                        is CustomMentionSpan -> {
                            span.url?.let(listener)
                            return true
                        }

                        else -> Unit
                    }
                }
                return false
            }

            override fun onDown(e: MotionEvent): Boolean {
                // No need to detect user interaction if there is no listener
                if (!hasAnyLinkListener()) return false
                // Find any spans with URLs in the coordinates
                val spans = findSpansForTouchEvent(e)
                return spans.any { it is URLSpan || it is PillSpan || it is CustomMentionSpan }
            }

            override fun onLongPress(e: MotionEvent) {
                // No need to process more if there is no listener
                val onLinkLongClickedListener = onLinkLongClickedListener ?: return
                handleLinkClicks(e, onLinkLongClickedListener)
            }

            override fun onSingleTapUp(e: MotionEvent): Boolean {
                // No need to detect user interaction if there is no listener
                val onLinkClickedListener = onLinkClickedListener ?: return false
                return handleLinkClicks(e, onLinkClickedListener)
            }
        })

    init {
        setSpannableFactory(spannableFactory)
        isInit = true
    }

    constructor(context: Context) : super(context, null)

    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs) {
        styleConfig = EditorEditTextAttributeReader(context, attrs).styleConfig
    }

    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int) : super(
        context, attrs, defStyleAttr
    ) {
        styleConfig = EditorEditTextAttributeReader(context, attrs).styleConfig
    }

    override fun setText(text: CharSequence?, type: BufferType?) {
        super.setText(text, type)
        // setText may be called during initialisation when we're not yet
        // ready to load the background helpers
        if (!isInit) return
        inlineCodeBgHelper.clearCachedPositions()
        codeBlockBgHelper.clearCachedPositions()
    }

    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) {
        if (isInit) {
            // The size changed, so the cached positions for the code renderers won't match anymore
            inlineCodeBgHelper.clearCachedPositions()
            codeBlockBgHelper.clearCachedPositions()
        }

        super.onSizeChanged(w, h, oldw, oldh)
    }

    /**
     * Sets up the styling used to translate HTML to Spanned text.
     * @param styleConfig The styles to use for the generated spans.
     * @param mentionDisplayHandler Used to decide how to display any mentions found in the HTML text.
     */
    fun updateStyle(styleConfig: StyleConfig, mentionDisplayHandler: MentionDisplayHandler?) {
        this.styleConfig = styleConfig
        this.mentionDisplayHandler = mentionDisplayHandler

        inlineCodeBgHelper =
            SpanBackgroundHelperFactory.createInlineCodeBackgroundHelper(styleConfig.inlineCode)
        codeBlockBgHelper =
            SpanBackgroundHelperFactory.createCodeBlockBackgroundHelper(styleConfig.codeBlock)

        htmlConverter = createHtmlConverter(styleConfig, mentionDisplayHandler)
    }

    /**
     * Set the text of the TextView with HTML formatting.
     * @param htmlText The text to display, with HTML formatting.
     * Consider using [HtmlConverter.fromHtmlToSpans] and [setText] instead.
     */
    fun setHtml(htmlText: String) {
        if (!isInit) return
        htmlConverter?.fromHtmlToSpans(htmlText)?.let { setText(it, BufferType.SPANNABLE) }
    }

    override fun onMeasure(widthMeasureSpec: Int, heightMeasureSpec: Int) {
        super.onMeasure(widthMeasureSpec, heightMeasureSpec)

        layout?.let { onTextLayout?.invoke(it) }
    }

    override fun onDraw(canvas: Canvas) {
        // need to draw bg first so that text can be on top during super.onDraw()
        if (text is Spanned && layout != null && isInit) {
            canvas.withTranslation(totalPaddingLeft.toFloat(), totalPaddingTop.toFloat()) {
                codeBlockBgHelper.draw(canvas, text as Spanned, layout)
                inlineCodeBgHelper.draw(canvas, text as Spanned, layout)
            }
        }
        super.onDraw(canvas)
    }

    override fun onAttachedToWindow() {
        super.onAttachedToWindow()

        updateStyle(styleConfig, mentionDisplayHandler)
    }

    private fun createHtmlConverter(
        styleConfig: StyleConfig, mentionDisplayHandler: MentionDisplayHandler?
    ): HtmlConverter {
        return HtmlConverter.Factory.create(context = context,
            styleConfig = styleConfig,
            mentionDisplayHandler = mentionDisplayHandler,
            isEditor = false,
            isMention = mentionDetector?.let { detector ->
                { _, url ->
                    detector.isMention(url)
                }
            })
    }

    override fun onTouchEvent(event: MotionEvent): Boolean {
        // We pass the event to the gesture detector
        val handled = gestureDetector.onTouchEvent(event)
        // We return if we handled the event and want to intercept it or not
        return if (!handled) {
            // This will handle the default actions for any touch event in the TextView
            super.onTouchEvent(event)
        } else {
            true
        }
    }

    private fun findSpansForTouchEvent(event: MotionEvent): Array<out Any> {
        val layout = this.layout ?: return emptyArray()
        // Find selection matching the pointer coordinates
        val offset = getOffsetForPosition(event.x, event.y)
        // For links that wrap several lines, we want to avoid opening the link if the touch event
        // happened on the empty space after the line wrapped.
        val currentLineWidth = layout.getLineWidth(layout.getLineForOffset(offset))
        return if (event.x <= currentLineWidth) {
            (text as? Spanned)?.getSpans<Any>(offset, offset).orEmpty()
        } else {
            emptyArray()
        }
    }
}
