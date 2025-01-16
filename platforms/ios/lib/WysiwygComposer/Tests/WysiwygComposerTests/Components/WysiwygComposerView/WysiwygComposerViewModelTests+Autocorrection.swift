//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

@testable import WysiwygComposer
import XCTest

extension WysiwygComposerViewModelTests {
    func testAutocorrectionIsDisabled() throws {
        mockTrailingTyping("/")
        assertAutocorrectDisabled()

        mockTrailingTyping("join")
        assertAutocorrectDisabled()

        mockTrailingTyping(" #some_room:matrix.org")
        assertAutocorrectDisabled()
    }

    func testAutocorrectionIsEnabled() throws {
        mockTrailingTyping("Just some text")
        assertAutoCorrectEnabled()

        mockTrailingTyping(" /not_a_command")
        assertAutoCorrectEnabled()
    }

    func testDoubleSlashKeepAutocorrectionEnabled() throws {
        mockTrailingTyping("//")
        assertAutoCorrectEnabled()
    }

    func testAutocorrectionIsReEnabled() throws {
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

    func testAutocorrectionAfterSetHtmlContent() {
        viewModel.setHtmlContent("/join #some_room:matrix.org")
        assertAutocorrectDisabled()

        viewModel.setHtmlContent("<strong>some text</strong>")
        assertAutoCorrectEnabled()
    }

    // Note: disable for now as this is broken by escaping the slash character
    // it could be fixed in `toggleAutocorrectionIfNeeded` text view function
    // but it would have a performance impact
//    func testAutocorrectionAfterSetHtmlContentInPlainTextMode() {
//        viewModel.plainTextMode = true
//
//        viewModel.setHtmlContent("/join #some_room:matrix.org")
//        assertAutocorrectDisabled()
//
//        viewModel.setHtmlContent("<strong>some text</strong>")
//        assertAutoCorrectEnabled()
//    }

    func testAutocorrectionAfterSetMarkdownContent() {
        viewModel.setMarkdownContent("/join #some_room:matrix.org")
        assertAutocorrectDisabled()

        viewModel.setMarkdownContent("__some text__")
        assertAutoCorrectEnabled()
    }

    // Note: disable for now as this is broken by escaping the slash character
    // it could be fixed in `toggleAutocorrectionIfNeeded` text view function
    // but it would have a performance impact
//    func testAutocorrectionAfterSetMarkdownContentInPlainTextMode() {
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
        XCTAssertEqual(viewModel.textView.autocorrectionType, .yes)
    }

    func assertAutocorrectDisabled() {
        XCTAssertEqual(viewModel.textView.autocorrectionType, .no)
    }
}
