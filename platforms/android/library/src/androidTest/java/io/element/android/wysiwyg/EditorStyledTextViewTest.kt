/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg

import android.graphics.Canvas
import android.graphics.Paint
import android.text.style.ReplacementSpan
import android.text.style.URLSpan
import android.widget.TextView
import androidx.core.text.buildSpannedString
import androidx.core.text.inSpans
import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.assertion.ViewAssertions.matches
import androidx.test.espresso.matcher.ViewMatchers
import androidx.test.espresso.matcher.ViewMatchers.withText
import androidx.test.ext.junit.rules.ActivityScenarioRule
import io.element.android.wysiwyg.test.R
import io.element.android.wysiwyg.test.utils.FakeLinkClickedListener
import io.element.android.wysiwyg.test.utils.TestActivity
import io.element.android.wysiwyg.test.utils.TextViewActions
import io.element.android.wysiwyg.test.utils.clickXY
import io.element.android.wysiwyg.view.spans.CustomMentionSpan
import io.element.android.wysiwyg.view.spans.LinkSpan
import io.element.android.wysiwyg.view.spans.PillSpan
import org.junit.Rule
import org.junit.Test


internal class EditorStyledTextViewTest {

    @get:Rule
    val scenarioRule = ActivityScenarioRule(TestActivity::class.java)

    private val fakeLinkClickedListener = FakeLinkClickedListener()

    companion object {
        const val HELLO_WORLD = "Hello, world"
        const val HELLO_WORLD_HTML = "<p>$HELLO_WORLD</p>"
        const val MENTION_TEXT = "@Alice"
        const val MENTION_URI = "https://matrix.to/#/@alice:matrix.org"
        const val MENTION_HTML = "<p><a href='$MENTION_URI'>$MENTION_TEXT</a></p>"
        const val URL = "https://matrix.org"
    }

    @Test
    fun testSetText() {
        onView(ViewMatchers.withId(R.id.styledTextView))
            .perform(TextViewActions.setText(HELLO_WORLD))
            .check(matches(withText(HELLO_WORLD)))
    }

    @Test
    fun testSetHtml() {
        onView(ViewMatchers.withId(R.id.styledTextView))
            .perform(TextViewActions.setHtml(HELLO_WORLD_HTML))
            .check(matches(withText(HELLO_WORLD)))
    }

    @Test
    fun testSetHtmlWithMention() {
        onView(ViewMatchers.withId(R.id.styledTextView))
            .perform(TextViewActions.setHtml(MENTION_HTML))
            .check(matches(withText(MENTION_TEXT)))
    }

    @Test
    fun testUrlClicks() {
        val urlSpanText = buildSpannedString {
            inSpans(URLSpan(URL)) {
                append(HELLO_WORLD)
            }
        }
        onView(ViewMatchers.withId(R.id.styledTextView))
            .perform(TextViewActions.setText(urlSpanText, TextView.BufferType.SPANNABLE))
            .perform(TextViewActions.setOnLinkClickedListener(fakeLinkClickedListener))
            .check(matches(withText(HELLO_WORLD)))
            .perform(clickXY(0f, 0f))

        fakeLinkClickedListener.assertLinkClicked(url = URL)
    }

    @Test
    fun testLinkClicks() {
        val linkSpanText = buildSpannedString {
            inSpans(LinkSpan(URL)) {
                append(HELLO_WORLD)
            }
        }
        onView(ViewMatchers.withId(R.id.styledTextView))
            .perform(TextViewActions.setText(linkSpanText, TextView.BufferType.SPANNABLE))
            .perform(TextViewActions.setOnLinkClickedListener(fakeLinkClickedListener))
            .check(matches(withText(HELLO_WORLD)))
            .perform(clickXY(0f, 0f))

        fakeLinkClickedListener.assertLinkClicked(url = URL)
    }

    @Test
    fun testPillSpanClicks() {
        val pillSpanText = buildSpannedString {
            inSpans(PillSpan(backgroundColor = 0, url = URL)) {
                append(HELLO_WORLD)
            }
        }
        onView(ViewMatchers.withId(R.id.styledTextView))
            .perform(TextViewActions.setText(pillSpanText, TextView.BufferType.SPANNABLE))
            .perform(TextViewActions.setOnLinkClickedListener(fakeLinkClickedListener))
            .check(matches(withText(HELLO_WORLD)))
            .perform(clickXY(0f, 0f))

        fakeLinkClickedListener.assertLinkClicked(url = URL)
    }

    @Test
    fun testCustomMentionSpanClicks() {
        val mentionSpanText = buildSpannedString {
            inSpans(CustomMentionSpan(DummyReplacementSpan, url = URL)) {
                append(HELLO_WORLD)
            }
        }
        onView(ViewMatchers.withId(R.id.styledTextView))
            .perform(TextViewActions.setText(mentionSpanText, TextView.BufferType.SPANNABLE))
            .perform(TextViewActions.setOnLinkClickedListener(fakeLinkClickedListener))
            .check(matches(withText(HELLO_WORLD)))
            .perform(clickXY(0f, 0f))

        fakeLinkClickedListener.assertLinkClicked(url = URL)
    }

    @Test
    fun testParsedMentionHtmlClicks() {
        onView(ViewMatchers.withId(R.id.styledTextView))
            .perform(TextViewActions.setHtml(MENTION_HTML))
            .perform(TextViewActions.setOnLinkClickedListener(fakeLinkClickedListener))
            .check(matches(withText(MENTION_TEXT)))
            .perform(clickXY(0f, 0f))

        fakeLinkClickedListener.assertLinkClicked(MENTION_URI)
    }
}

object DummyReplacementSpan : ReplacementSpan() {
    override fun getSize(paint: Paint, text: CharSequence?, start: Int, end: Int, fm: Paint.FontMetricsInt?): Int = 100

    override fun draw(canvas: Canvas, text: CharSequence?, start: Int, end: Int, x: Float, top: Int, y: Int, bottom: Int, paint: Paint)  = Unit

}

