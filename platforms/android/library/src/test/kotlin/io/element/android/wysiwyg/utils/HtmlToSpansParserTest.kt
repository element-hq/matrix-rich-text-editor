/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.utils

import android.text.Spanned
import io.element.android.wysiwyg.display.MentionDisplayHandler
import io.element.android.wysiwyg.display.TextDisplay
import io.element.android.wysiwyg.test.fakes.createFakeStyleConfig
import io.element.android.wysiwyg.test.utils.dumpSpans
import io.element.android.wysiwyg.view.spans.ExtraCharacterSpan
import io.element.android.wysiwyg.view.spans.OrderedListSpan
import io.element.android.wysiwyg.view.spans.TableRowSpan
import io.element.android.wysiwyg.view.spans.TableSpan
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
    fun testOrderedListWithStartAttribute() {
        val html = """
            <ol start="3">
                <li>ordered1</li>
                <li>ordered2</li>
            </ol>
        """.trimIndent()
        val spanned = convertHtml(html)

        assertThat(
            spanned.dumpSpans().joinToString(",\n"), equalTo(
                """
                    ordered1: io.element.android.wysiwyg.view.spans.OrderedListSpan (0-8) fl=#17,
                    ordered2: io.element.android.wysiwyg.view.spans.OrderedListSpan (9-17) fl=#17
                """.trimIndent()
            )
        )

        val listItemSpans = spanned.getSpans(0, spanned.length, OrderedListSpan::class.java)
        // The first item should have order 3
        assert(listItemSpans.first().order == 3)
        // The second item should continue the order
        assert(listItemSpans.last().order == 4)
    }

    @Test
    fun testListsWithPreviousText() {
        val html = """
            Hey
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

        assertThat(spanned.toString(), equalTo("Hey\nordered1\nordered2\nbullet1\nbullet2"))

        assertThat(
            spanned.dumpSpans().joinToString(",\n"), equalTo(
                """
                    ordered1: io.element.android.wysiwyg.view.spans.OrderedListSpan (4-12) fl=#17,
                    ordered2: io.element.android.wysiwyg.view.spans.OrderedListSpan (13-21) fl=#17,
                    bullet1: io.element.android.wysiwyg.view.spans.UnorderedListSpan (22-29) fl=#17,
                    bullet2: io.element.android.wysiwyg.view.spans.UnorderedListSpan (30-37) fl=#17
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
    fun testTables() {
        val html = "<table><tbody>" +
                "<tr><td>a</td><td>b</td></tr>" +
                "<tr><td>c</td><td>d</td></tr>" +
                "</tbody></table>"
        val spanned = convertHtml(html)

        // The separators' text contains spaces, which trips up dumpSpans()'s naive
        // hash-stripping (it splits on " "), so check span ranges directly instead. Only 2 of
        // the 3 separator characters are extra - the 3rd is real, representing the
        // ComposerModel's +1 position for the cell-to-cell transition (see parseTable's kdoc).
        // The row-break '\n' is entirely real for the same reason.
        val extraCharacterRanges = spanned.getSpans(0, spanned.length, ExtraCharacterSpan::class.java)
            .map { spanned.getSpanStart(it) to spanned.getSpanEnd(it) }
            .sortedBy { it.first }
        assertThat(
            extraCharacterRanges, equalTo(
                listOf(1 to 3, 7 to 9)
            )
        )
        assertThat(
            spanned.toString(), equalTo("a | b\nc | d")
        )

        // TableSpan covers the whole table; each TableRowSpan covers only its own row's content
        // (not the leading '\n' transition into it) - these back the box/row-divider rendering.
        val tableSpans = spanned.getSpans(0, spanned.length, TableSpan::class.java)
        assertThat(tableSpans.size, equalTo(1))
        assertThat(
            spanned.getSpanStart(tableSpans[0]) to spanned.getSpanEnd(tableSpans[0]),
            equalTo(0 to 11)
        )
        val rowRanges = spanned.getSpans(0, spanned.length, TableRowSpan::class.java)
            .map { spanned.getSpanStart(it) to spanned.getSpanEnd(it) }
            .sortedBy { it.first }
        assertThat(
            rowRanges, equalTo(
                listOf(0 to 5, 6 to 11)
            )
        )
    }

    @Test
    fun testEmptyTableCells() {
        val html = "<table><tbody><tr><td></td><td></td></tr></tbody></table>"
        val spanned = convertHtml(html)

        // Every empty cell gets its own NBSP placeholder so it's individually tappable/typeable,
        // regardless of whether it's the very first content in the document.
        assertThat(
            spanned.toString(), equalTo("$NBSP | $NBSP")
        )
        val extraCharacterRanges = spanned.getSpans(0, spanned.length, ExtraCharacterSpan::class.java)
            .map { spanned.getSpanStart(it) to spanned.getSpanEnd(it) }
            .sortedBy { it.first }
        assertThat(
            extraCharacterRanges, equalTo(
                listOf(0 to 1, 1 to 3, 4 to 5)
            )
        )
        assertNoOverlappingExtraCharacterSpans(spanned)
    }

    @Test
    fun testTablesWithEmptyAndNonEmptyCellsDontOverlapExtraCharacterSpans() {
        // Regression test: an empty cell immediately after the " | " separator used to fool
        // handleNbspInBlock into thinking it wasn't the first content in the document, adding a
        // second ExtraCharacterSpan that overlapped the separator's own span. The resulting
        // double-counted index math made every tap/type in a non-first cell resolve back near
        // the start of the composer model, so typed text always landed in the first cell.
        val html = "<table><tbody><tr><td>a</td><td></td><td>c</td></tr></tbody></table>"
        val spanned = convertHtml(html)

        assertThat(
            spanned.toString(), equalTo("a | $NBSP | c")
        )
        val extraCharacterRanges = spanned.getSpans(0, spanned.length, ExtraCharacterSpan::class.java)
            .map { spanned.getSpanStart(it) to spanned.getSpanEnd(it) }
            .sortedBy { it.first }
        assertThat(
            extraCharacterRanges, equalTo(
                listOf(1 to 3, 4 to 5, 5 to 7)
            )
        )
        assertNoOverlappingExtraCharacterSpans(spanned)
    }

    @Test
    fun testTableCellToCellTransitionsMatchComposerModelPositions() {
        // Ground truth verified directly against the Rust ComposerModel: moving from one table
        // cell to the next - whether via a column or a row transition, and regardless of whether
        // the cells are empty - always advances its internal position by exactly 1. This test
        // guards against undercounting that (the underlying cause of the "text always lands in
        // the first cell" bug), which the "no overlapping spans" checks above wouldn't catch on
        // their own since undercounting doesn't require any overlap.

        // Case A: <table><tbody><tr><td>a</td><td></td></tr></tbody></table>, cursor at the
        // start of the (empty) 2nd cell. Verified in Rust: position 2 (1 for "a" + 1 transition).
        val caseA = convertHtml("<table><tbody><tr><td>a</td><td></td></tr></tbody></table>")
        assertThat(
            EditorIndexMapper.fromEditorToComposer(caseA.length - 1, caseA.length - 1, caseA)!!,
            equalTo(2u to 2u)
        )

        // Case B: <table><tbody><tr><td></td><td></td></tr></tbody></table>, cursor at the start
        // of the (empty) 2nd cell, 1st cell also empty. Verified in Rust: position 1.
        val caseB = convertHtml("<table><tbody><tr><td></td><td></td></tr></tbody></table>")
        assertThat(
            EditorIndexMapper.fromEditorToComposer(caseB.length - 1, caseB.length - 1, caseB)!!,
            equalTo(1u to 1u)
        )

        // Case C: <table><tbody><tr><td>a</td></tr><tr><td></td></tr></tbody></table>, cursor at
        // the start of the (empty) 2nd row's cell. Verified in Rust: position 2.
        val caseC = convertHtml(
            "<table><tbody><tr><td>a</td></tr><tr><td></td></tr></tbody></table>"
        )
        assertThat(
            EditorIndexMapper.fromEditorToComposer(caseC.length - 1, caseC.length - 1, caseC)!!,
            equalTo(2u to 2u)
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

    @Test
    fun testLeadingLineBreakCharsInHtmlTextAreIgnored() {
        val html = "<p>First Line</p>\n<p>Line after Empty Line<br />\nThird Line</p>\n"
        val spanned = convertHtml(
            html = html,
            isEditor = false,
        )
        assertThat(
            spanned.toString(), equalTo("First Line\n\nLine after Empty Line\nThird Line")
        )
    }

    @Test
    fun testLineBreakCharsInMiddleOrEndOfHtmlTextAreConvertedToWhitespace() {
        val html = "<p>First Line\n</p><p>Line after Empty Line<br />Third\n\n\n\nWith more</p>"
        val spanned = convertHtml(
            html = html,
            isEditor = false,
        )
        assertThat(
            spanned.toString(),
            equalTo("First Line\n\nLine after Empty Line\nThird With more")
        )
    }

    @Test
    fun testMultipleLineBreaksWithNoBlockTags() {
        val html = "1<br/>2<br/>3<br/>4<br/><br/>5"
        val spanned = convertHtml(
            html = html,
            isEditor = false,
        )
        assertThat(
            spanned.toString(),
            equalTo("1\n2\n3\n4\n\n5")
        )
    }

    @Test
    fun testLineBreakNotAddedAfterStrikethroughOrCodeTags() {
        val htmlStrikethrough = "Testing <del>strikethrough</del>$NBSP"
        val spannedStrikeThrough = convertHtml(
            html = htmlStrikethrough,
            isEditor = false,
        )
        assertThat(
            spannedStrikeThrough.toString(),
            equalTo("Testing strikethrough$NBSP")
        )

        val htmlCode = "Testing <code>inline code</code>$NBSP"
        val spannedStrikeCode = convertHtml(
            html = htmlCode,
            isEditor = false,
        )
        assertThat(
            spannedStrikeCode.toString(),
            equalTo("Testing inline code$NBSP")
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
            dom = HtmlToDomParser.document(html),
            styleConfig = styleConfig,
            mentionDisplayHandler = mentionDisplayHandler,
            isEditor = isEditor,
            isMention = { _, url ->
                url.startsWith("https://matrix.to/#/@")
            }
        ).convert()
    }

    /**
     * Overlapping [ExtraCharacterSpan]s are double-counted by [EditorIndexMapper]'s span-length
     * summing, which corrupts editor <-> composer index mapping (see the tables regression tests
     * above). Touching-but-not-overlapping spans (e.g. (1-4) and (4-5)) are fine.
     */
    private fun assertNoOverlappingExtraCharacterSpans(spanned: Spanned) {
        val ranges = spanned.getSpans(0, spanned.length, ExtraCharacterSpan::class.java)
            .map { spanned.getSpanStart(it) to spanned.getSpanEnd(it) }
            .sortedBy { it.first }
        for (i in 1 until ranges.size) {
            assertThat(
                "Overlapping ExtraCharacterSpans: ${ranges[i - 1]} and ${ranges[i]}",
                ranges[i - 1].second <= ranges[i].first,
                equalTo(true)
            )
        }
    }
}
