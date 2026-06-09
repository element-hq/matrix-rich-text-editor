//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Testing
import UIKit
@testable import WysiwygComposer

extension WysiwygComposerViewModelTests {
    @Test func autocorrectionIsDisabled() {
        mockTrailingTyping("/")
        assertAutocorrectDisabled()

        mockTrailingTyping("join")
        assertAutocorrectDisabled()

        mockTrailingTyping(" #some_room:matrix.org")
        assertAutocorrectDisabled()
    }

    @Test func autocorrectionIsEnabled() {
        mockTrailingTyping("Just some text")
        assertAutoCorrectEnabled()

        mockTrailingTyping(" /not_a_command")
        assertAutoCorrectEnabled()
    }

    @Test func doubleSlashKeepAutocorrectionEnabled() {
        mockTrailingTyping("//")
        assertAutoCorrectEnabled()
    }

    @Test func autocorrectionIsReEnabled() {
        mockTrailingTyping("/")
        assertAutocorrectDisabled()

        mockTrailingBackspace()
        assertAutoCorrectEnabled()

        mockTrailingTyping("/join")
        assertAutocorrectDisabled()

        for _ in 0...4 {
            mockTrailingBackspace()
        }
        assertAutoCorrectEnabled()
    }

    @Test func autocorrectionAfterSetHtmlContent() {
        viewModel.setHtmlContent("/join #some_room:matrix.org")
        assertAutocorrectDisabled()

        viewModel.setHtmlContent("<strong>some text</strong>")
        assertAutoCorrectEnabled()
    }

    // Note: disable for now as this is broken by escaping the slash character
    // it could be fixed in `toggleAutocorrectionIfNeeded` text view function
    // but it would have a performance impact
//    @Test func autocorrectionAfterSetHtmlContentInPlainTextMode() {
//        viewModel.plainTextMode = true
//
//        viewModel.setHtmlContent("/join #some_room:matrix.org")
//        assertAutocorrectDisabled()
//
//        viewModel.setHtmlContent("<strong>some text</strong>")
//        assertAutoCorrectEnabled()
//    }

    @Test func autocorrectionAfterSetMarkdownContent() {
        viewModel.setMarkdownContent("/join #some_room:matrix.org")
        assertAutocorrectDisabled()

        viewModel.setMarkdownContent("__some text__")
        assertAutoCorrectEnabled()
    }

    // Note: disable for now as this is broken by escaping the slash character
    // it could be fixed in `toggleAutocorrectionIfNeeded` text view function
    // but it would have a performance impact
//    @Test func autocorrectionAfterSetMarkdownContentInPlainTextMode() {
//        viewModel.plainTextMode = true
//
//        viewModel.setMarkdownContent("/join #some_room:matrix.org")
//        assertAutocorrectDisabled()
//
//        viewModel.setMarkdownContent("__some text__")
//        assertAutoCorrectEnabled()
//    }
}

private extension WysiwygComposerViewModelTests {
    func assertAutoCorrectEnabled() {
        #expect(viewModel.textView.autocorrectionType == .yes)
    }

    func assertAutocorrectDisabled() {
        #expect(viewModel.textView.autocorrectionType == .no)
    }
}
