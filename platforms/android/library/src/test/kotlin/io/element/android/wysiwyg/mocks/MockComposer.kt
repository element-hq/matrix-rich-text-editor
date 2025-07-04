/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.mocks

import io.element.android.wysiwyg.extensions.toUShortList
import io.mockk.every
import io.mockk.mockk
import uniffi.wysiwyg_composer.ComposerModelInterface
import uniffi.wysiwyg_composer.ComposerState
import uniffi.wysiwyg_composer.ComposerUpdate
import uniffi.wysiwyg_composer.LinkAction
import uniffi.wysiwyg_composer.MentionsState

class MockComposer {
    val instance = mockk<ComposerModelInterface>()

    init {
        givenCurrentDomState()
        givenActionStates()
        givenDummyToExampleFormat()
        givenGetContentAsPlainText()
        givenMentionsState(MentionsState(userIds = emptyList(), roomIds = emptyList(), roomAliases = emptyList(), hasAtRoomMention = false))
    }

    fun givenCurrentDomState(
        html: String = "",
        start: Int = 0,
        end: Int = 0,
    ) = every { instance.getCurrentDomState() } returns
            ComposerState(html.toUShortList(), start.toUInt(), end.toUInt())

    fun givenActionStates() =
        every {
            instance.actionStates()
        } returns mapOf()

    fun givenReplaceTextResult(
        update: ComposerUpdate = MockComposerUpdateFactory.create(),
    ) = every { instance.replaceText(any()) } returns update

    fun givenEnterResult(
        update: ComposerUpdate = MockComposerUpdateFactory.create(),
    ) = every { instance.enter() } returns update

    fun givenBackspaceResult(
        update: ComposerUpdate = MockComposerUpdateFactory.create(),
    ) = every { instance.backspace() } returns update

    fun givenBoldResult(
        update: ComposerUpdate = MockComposerUpdateFactory.create(),
    ) = every { instance.bold() } returns update

    fun givenItalicResult(
        update: ComposerUpdate = MockComposerUpdateFactory.create(),
    ) = every { instance.italic() } returns update

    fun givenUnderlineResult(
        update: ComposerUpdate = MockComposerUpdateFactory.create(),
    ) = every { instance.underline() } returns update

    fun givenStrikeThroughResult(
        update: ComposerUpdate = MockComposerUpdateFactory.create(),
    ) = every { instance.strikeThrough() } returns update

    fun givenInlineCodeResult(
        update: ComposerUpdate = MockComposerUpdateFactory.create(),
    ) = every { instance.inlineCode() } returns update

    fun givenDeleteInResult(
        start: Int,
        end: Int,
        update: ComposerUpdate = MockComposerUpdateFactory.create(),
    ) = every { instance.deleteIn(start.toUInt(), end.toUInt()) } returns update

    fun givenLinkAction(
        linkAction: LinkAction,
    ) = every { instance.getLinkAction() } returns linkAction

    fun givenSetLinkResult(
        url: String,
        update: ComposerUpdate = MockComposerUpdateFactory.create(),
    ) = every { instance.setLink(url = url, attributes = emptyList()) } returns update

    fun givenSetLinkWithTextResult(
        url: String,
        text: String,
        update: ComposerUpdate = MockComposerUpdateFactory.create(),
    ) = every { instance.setLinkWithText(url = url, text = text, attributes = emptyList()) } returns update

    fun givenRemoveLinkResult(
        update: ComposerUpdate = MockComposerUpdateFactory.create(),
    ) = every { instance.removeLinks() } returns update

    fun givenInsertMentionFromSuggestionResult(
        name: String,
        link: String,
        update: ComposerUpdate = MockComposerUpdateFactory.create(),
    ) = every { instance.insertMentionAtSuggestion(url = link, attributes = emptyList(), text = name, suggestion = any()) } returns update

    fun givenInsertAtMentionFromSuggestionResult(
        update: ComposerUpdate = MockComposerUpdateFactory.create(),
    ) = every { instance.insertAtRoomMentionAtSuggestion(suggestion = any()) } returns update

    fun givenReplaceAllHtmlResult(
        html: String,
        update: ComposerUpdate = MockComposerUpdateFactory.create(),
    ) = every { instance.setContentFromHtml(html = html) } returns update

    fun givenReplaceAllMarkdownResult(
        markdown: String,
        update: ComposerUpdate = MockComposerUpdateFactory.create(),
    ) = every { instance.setContentFromMarkdown(markdown = markdown) } returns update

    fun givenUndoResult(
        update: ComposerUpdate = MockComposerUpdateFactory.create(),
    ) = every { instance.undo() } returns update

    fun givenRedoResult(
        update: ComposerUpdate = MockComposerUpdateFactory.create(),
    ) = every { instance.redo() } returns update

    fun givenToggleOrderedListResult(
        update: ComposerUpdate = MockComposerUpdateFactory.create(),
    ) = every { instance.orderedList() } returns update

    fun givenToggleUnorderedListResult(
        update: ComposerUpdate = MockComposerUpdateFactory.create(),
    ) = every { instance.unorderedList() } returns update

    fun givenToggleCodeBlock(
        update: ComposerUpdate = MockComposerUpdateFactory.create(),
    ) = every { instance.codeBlock() } returns update

    fun givenToggleQuote(
        update: ComposerUpdate = MockComposerUpdateFactory.create(),
    ) = every { instance.quote() } returns update

    fun givenErrorInUpdateSelection(
        throwable: Throwable = IllegalStateException("Invalid selection range"),
    ) = every { instance.select(any(), any()) } throws throwable

    fun givenGetContentAsMessageHtml(
        html: String = ""
    ) = every { instance.getContentAsMessageHtml() } returns html

    fun givenGetContentAsMarkdown(
        markdown: String = ""
    ) = every { instance.getContentAsMarkdown() } returns markdown

    fun givenGetContentAsPlainText(
        plainText: String = ""
    ) = every { instance.getContentAsPlainText() } returns plainText

    fun givenDummyToExampleFormat() = every { instance.toExampleFormat() } returns ""

    fun givenMentionsState(mentionsState: MentionsState) = every { instance.getMentionsState() } returns mentionsState
}
