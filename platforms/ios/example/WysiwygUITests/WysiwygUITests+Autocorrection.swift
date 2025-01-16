//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import XCTest

extension WysiwygUITests {
    func testRichTextModeAutocorrection() throws {
        textView.typeTextCharByChar("/")
        XCTAssertFalse(image(.autocorrectionIndicator).exists)
        textView.typeText(XCUIKeyboardKey.delete.rawValue)
        XCTAssertTrue(image(.autocorrectionIndicator).exists)
        textView.typeTextCharByChar("/join")
        XCTAssertFalse(image(.autocorrectionIndicator).exists)
        // Send message
        button(.sendButton).tap()
        XCTAssertTrue(image(.autocorrectionIndicator).exists)
    }

    func testPlainTextModeAutocorrection() throws {
        waitForButtonToExistAndTap(.plainRichButton)
        textView.typeTextCharByChar("/")
        XCTAssertFalse(image(.autocorrectionIndicator).exists)
        textView.typeText(XCUIKeyboardKey.delete.rawValue)
        XCTAssertTrue(image(.autocorrectionIndicator).exists)
        textView.typeTextCharByChar("/join")
        XCTAssertFalse(image(.autocorrectionIndicator).exists)
        // Send message
        button(.sendButton).tap()
        XCTAssertTrue(image(.autocorrectionIndicator).exists)
    }

    func testRichTextModeNonLeadingCommand() throws {
        textView.typeTextCharByChar("text /not_a_command")
        XCTAssertTrue(image(.autocorrectionIndicator).exists)
    }

    func testPlainTextModeNonLeadingCommand() throws {
        waitForButtonToExistAndTap(.plainRichButton)
        textView.typeTextCharByChar("text /not_a_command")
        XCTAssertTrue(image(.autocorrectionIndicator).exists)
    }
}
