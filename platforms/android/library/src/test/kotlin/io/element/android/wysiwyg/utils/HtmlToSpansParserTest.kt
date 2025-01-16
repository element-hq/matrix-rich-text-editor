/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.utils

import android.text.Spanned
import io.element.android.wysiwyg.display.TextDisplay
import io.element.android.wysiwyg.display.MentionDisplayHandler
import io.element.android.wysiwyg.test.fakes.createFakeStyleConfig
import io.element.android.wysiwyg.test.utils.dumpSpans
import org.hamcrest.MatcherAssert.assertThat
import org.hamcrest.Matchers.contains
import org.hamcrest.Matchers.equalTo
import org.hamcrest.Matchers.not
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.RuntimeEnvironment

@RunWith(RobolectricTestRunner::class)
class HtmlToSpansParserTest {
    @Test
    fun testStyles() {
        val html = "<b>bold</b>" +
                "<i>italic</i>" +
                "<u>underline</u>" +
                "<strong>strong</strong>" +
                "<em>emphasis</em>" +
                "<del>strikethrough</del>" +
                "<code>code</code>"
        val spanned = convertHtml(html)

        assertThat(
            spanned.dumpSpans(), equalTo(
                listOf(
                    "bold: android.text.style.StyleSpan (0-4) fl=#17",
                    "italic: android.text.style.StyleSpan (4-10) fl=#17",
                    "underline: android.text.style.UnderlineSpan (10-19) fl=#17",
                    "strong: android.text.style.StyleSpan (19-25) fl=#17",
                    "emphasis: android.text.style.StyleSpan (25-33) fl=#17",
                    "strikethrough: android.text.style.StrikethroughSpan (33-46) fl=#17",
                    "code: io.element.android.wysiwyg.view.spans.InlineCodeSpan (46-50) fl=#17",
                )
            )
        )
    }

    @Test
    fun testLists() {
        val html = """
            <ol>
                <li>ordered1</li>
                <li>ordered2</li>
            </ol>
            <ul> 
                <li>bullet1</li>
                <li>bullet2</li>
            </ul>
        """.trimIndent()
        val spanned = convertHtml(html)

        assertThat(
            spanned.dumpSpans().joinToString(",\n"), equalTo(
                """
                    ordered1: io.element.android.wysiwyg.view.spans.OrderedListSpan (0-8) fl=#17,
                    ordered2: io.element.android.wysiwyg.view.spans.OrderedListSpan (9-17) fl=#17,
                    bullet1: io.element.android.wysiwyg.view.spans.UnorderedListSpan (18-25) fl=#17,
                    bullet2: io.element.android.wysiwyg.view.spans.UnorderedListSpan (26-33) fl=#17
                """.trimIndent()
            )
        )
    }

    @Test
    fun testLineBreaks() {
        val html = "Hello<br>world"
        val spanned = convertHtml(html)
        assertThat(
            spanned.dumpSpans(), equalTo(
                emptyList()
            )
        )
        assertThat(
            spanned.toString(), equalTo("Hello\nworld")
        )
    }

    @Test
    fun testParagraphs() {
        val html = "<p>Hello</p><p>world</p>"
        val spanned = convertHtml(html)
        assertThat(
            spanned.dumpSpans(), equalTo(
                emptyList()
            )
        )
        assertThat(
            spanned.toString(), equalTo("Hello\nworld")
        )
    }

    @Test
    fun testEmptyParagraphs() {
        val html = "<p></p><p></p>"
        val spanned = convertHtml(html)
        assertThat(
            spanned.dumpSpans(), equalTo(
                listOf(
                    "\n: io.element.android.wysiwyg.view.spans.ExtraCharacterSpan (0-1) fl=#17",
                )
            )
        )
        assertThat(
            spanned.toString(), equalTo("\n$NBSP")
        )
    }

    @Test
    fun testLineBreakCanWorkWithParagraphs() {
        val html = "<p>Hello</p><br /><p>world</p>"
        val spanned = convertHtml(html)
        assertThat(
            spanned.dumpSpans(), equalTo(emptyList())
        )
        assertThat(
            spanned.toString(), equalTo("Hello\n\nworld")
        )
    }

    @Test
    fun testMentionDisplayWithCustomMentionDisplayHandler() {
        val html = """
            <a href="https://element.io">link</a>$NBSP
            <a href="https://matrix.to/#/@test:example.org" contenteditable="false">jonny</a>$NBSP@room
        """.trimIndent()
        val spanned = convertHtml(html, mentionDisplayHandler = object : MentionDisplayHandler {
            override fun resolveAtRoomMentionDisplay(): TextDisplay =
                TextDisplay.Pill

            override fun resolveMentionDisplay(text: String, url: String): TextDisplay =
                TextDisplay.Pill
        })
        assertThat(
            spanned.dumpSpans(), equalTo(
                listOf(
                    "link: io.element.android.wysiwyg.view.spans.LinkSpan (0-4) fl=#17",
                    "onny: io.element.android.wysiwyg.view.spans.ExtraCharacterSpan (6-10) fl=#33",
                    "jonny: io.element.android.wysiwyg.view.spans.PillSpan (5-10) fl=#17",
                    "@room: io.element.android.wysiwyg.view.spans.PillSpan (11-16) fl=#33",
                )
            )
        )
        assertThat(
            spanned.toString().replace(NBSP, ' '), equalTo("link jonny @room")
        )
    }

    @Test
    fun testMentionWithNoTextIsIgnored() {
        val html = """
            foo<a href="https://matrix.to/#/@test:example.org" contenteditable="false"></a>bar
        """.trimIndent()
        val spanned = convertHtml(html, mentionDisplayHandler = object : MentionDisplayHandler {
            override fun resolveAtRoomMentionDisplay(): TextDisplay =
                TextDisplay.Pill

            override fun resolveMentionDisplay(text: String, url: String): TextDisplay =
                TextDisplay.Pill
        })
        assertThat(
            spanned.dumpSpans(), not(contains("PillSpan"))
        )
        assertThat(
            spanned.toString(), equalTo("foobar")
        )
    }

    @Test
    fun testParagraphsAreTranslatedToSingleLineBreakWhenEditorModeIsEnabled() {
        val html = """
            <p>Hello</p><p>World!</p>
        """.trimIndent()
        val spanned = convertHtml(html, isEditor = true, mentionDisplayHandler = object : MentionDisplayHandler {
            override fun resolveAtRoomMentionDisplay(): TextDisplay =
                TextDisplay.Pill

            override fun resolveMentionDisplay(text: String, url: String): TextDisplay =
                TextDisplay.Pill
        })
        assertThat(
            spanned.toString(), equalTo("Hello\nWorld!")
        )
    }

    @Test
    fun testParagraphsAreTranslatedToDoubleLineBreakWhenEditorModeIsDisabled() {
        val html = """
            <p>Hello</p><p>World!</p>
        """.trimIndent()
        val spanned = convertHtml(html, isEditor = false, mentionDisplayHandler = object : MentionDisplayHandler {
            override fun resolveAtRoomMentionDisplay(): TextDisplay =
                TextDisplay.Pill

            override fun resolveMentionDisplay(text: String, url: String): TextDisplay =
                TextDisplay.Pill
        })
        assertThat(
            spanned.toString(), equalTo("Hello\n\nWorld!")
        )
    }

    private fun convertHtml(
        html: String,
        isEditor: Boolean = true,
        mentionDisplayHandler: MentionDisplayHandler? = null,
    ): Spanned {
        val app = RuntimeEnvironment.getApplication()
        val styleConfig = createFakeStyleConfig()
        return HtmlToSpansParser(
            resourcesHelper = AndroidResourcesHelper(context = app),
            html = html,
            styleConfig = styleConfig,
            mentionDisplayHandler = mentionDisplayHandler,
            isEditor = isEditor,
            isMention = { _, url ->
                url.startsWith("https://matrix.to/#/@")
            }
        ).convert()
    }
}
