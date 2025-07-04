//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import XCTest

extension WysiwygUITests {
    /// Type a text and delete some different kind of text selections with the composer.
    func testTypingAndDeleting() throws {
        // Type something into composer.
        textView.typeTextCharByChar("abc🎉🎉👩🏿‍🚀")
        assertTextViewContent("abc🎉🎉👩🏿‍🚀")

        // Test deleting parts of the text.
        let deleteKey = app.keys["delete"]
        deleteKey.tap()
        assertTextViewContent("abc🎉🎉")

        let delete3CharString = String(repeating: XCUIKeyboardKey.delete.rawValue, count: 3)
        textView.typeTextCharByChar(delete3CharString)
        assertTextViewContent("ab")

        // Rewrite some content.
        textView.typeTextCharByChar("cde 🥳 fgh")
        assertTextViewContent("abcde 🥳 fgh")

        // Double tap results in selecting the last word.
        textView.doubleTap()
        deleteKey.tap()
        // Note: iOS is removing the whitespace right after the emoji, even though it reports
        // through `shouldChangeTextIn` that it is removing only the 3 last chars.
        assertTextViewContent("abcde 🥳")

        // Triple tap selects the entire line.
        textView.tap(withNumberOfTaps: 3, numberOfTouches: 1)
        deleteKey.tap()
        assertTextViewContent("")
    }

    /// Type and send a message with the composer.
    ///
    /// Expected plain text content is "Some bold text" and
    /// HTML representation is "Some bold <strong>text</strong>"
    func testTypingAndSending() throws {
        // Type something into composer.
        textView.typeTextCharByChar("Some bold text")

        textView.doubleTap()
        // 1s is more than enough for the Rust side to get notified for the selection.
        sleep(1)
        button(.boldButton).tap()
        // We can't detect data being properly reported back to the model but
        // 1s is more than enough for the Rust side to get notified for the selection.
        sleep(1)
        button(.sendButton).tap()

        XCTAssertEqual(staticText(.contentText).label, "Some bold __text__")
        XCTAssertEqual(staticText(.htmlContentText).label, "Some bold <strong>text</strong>")
    }

    // Remember to disable hardware keyboard and use only software keyboard for this UITest
    func testTypingFast() throws {
        let text = "Some long text that I am going to type very fast"
        textView.tap()
        sleep(1)
        textView.typeText(text)
        let options = XCTExpectedFailure.Options()
        options.isStrict = false
        XCTExpectFailure("Typing fast might fail on CI", options: options)
        assertTextViewContent(text)
    }

    func testLongPressDelete() throws {
        let multilineText =
            """
            test1
            test2
            test3
            test4
            test5
            test6
            test7
            test8
            test9
            test10
            """
        app.typeTextCharByChar(multilineText)
        XCUIApplication().keys["delete"].press(forDuration: 15.0)
        assertTextViewContent("")
    }
}
