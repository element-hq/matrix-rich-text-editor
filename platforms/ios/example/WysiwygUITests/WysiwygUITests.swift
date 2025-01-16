//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import XCTest

class WysiwygUITests: XCTestCase {
    let app = XCUIApplication(bundleIdentifier: "org.matrix.Wysiwyg")

    override func setUpWithError() throws {
        continueAfterFailure = false
        app.launch()
        try focusComposerAndClearTutorialIfNeeded()
    }

    override func tearDownWithError() throws { }

    func testMinMaxResizing() throws {
        sleep(1)
        XCTAssertEqual(textView.frame.height.roundedToFirstTwoDigits(),
                       WysiwygSharedConstants.composerMinHeight.roundedToFirstTwoDigits())
        button(.minMaxButton).tap()
        sleep(1)
        XCTAssertEqual(textView.frame.height.roundedToFirstTwoDigits(),
                       WysiwygSharedConstants.composerMaxExtendedHeight.roundedToFirstTwoDigits())
        button(.minMaxButton).tap()
        sleep(1)
        XCTAssertEqual(textView.frame.height.roundedToFirstTwoDigits(),
                       WysiwygSharedConstants.composerMinHeight.roundedToFirstTwoDigits())
    }

    func testCrashRecovery() throws {
        button(.boldButton).tap()
        textView.typeTextCharByChar("Some ")
        button(.italicButton).tap()
        textView.typeTextCharByChar("text")
        assertTreeEquals(
            """
            └>strong
              ├>"Some "
              └>em
                └>"text"
            """
        )
        button(.forceCrashButton).tap()
        assertTreeEquals(
            """
            └>"Some text"
            """
        )
    }

    func testRemoveFocus() throws {
        textView.typeTextCharByChar("Test")
        XCTAssertTrue(keyboardIsDisplayed)
        button(.toggleFocusButton).tap()
        XCTAssertFalse(keyboardIsDisplayed)
        button(.toggleFocusButton).tap()
        XCTAssertTrue(keyboardIsDisplayed)
    }
}

extension WysiwygUITests {
    /// Returns the text view component of the composer.
    var textView: XCUIElement {
        app.textViews[rawIdentifier(.composerTextView)]
    }

    /// Returns true if the application is currently displaying a keyboard.
    var keyboardIsDisplayed: Bool {
        app.keyboards.count > 0
    }

    /// Get the button with given id
    ///
    /// - Parameter id: Accessibility identifier
    /// - Returns: Associated button, if it exists
    func button(_ id: WysiwygSharedAccessibilityIdentifier) -> XCUIElement {
        app.buttons[rawIdentifier(id)]
    }

    /// Get the text field with given id
    ///
    /// - Parameter id: Accessibility identifier
    /// - Returns: Associated text field, if it exists
    func textField(_ id: WysiwygSharedAccessibilityIdentifier) -> XCUIElement {
        app.textFields[rawIdentifier(id)]
    }

    /// Get the image with given id
    ///
    /// - Parameter id: Accessibility identifier
    /// - Returns: Associated image, if it exists
    func image(_ id: WysiwygSharedAccessibilityIdentifier) -> XCUIElement {
        app.images[rawIdentifier(id)]
    }

    /// Wait for buton with given id to exist, then tap it.
    ///
    /// - Parameter id: Accessibility identifier
    func waitForButtonToExistAndTap(_ id: WysiwygSharedAccessibilityIdentifier) {
        let expectation = expectation(
            for: NSPredicate(format: "exists == true"),
            evaluatedWith: button(id),
            handler: .none
        )
        let result = XCTWaiter.wait(for: [expectation], timeout: 30.0)
        XCTAssertEqual(result, .completed)
        button(id).tap()
    }

    /// Get the static text with given id
    ///
    /// - Parameter id: Accessibility identifier
    /// - Returns: Associated static text, if it exists
    func staticText(_ id: WysiwygSharedAccessibilityIdentifier) -> XCUIElement {
        app.staticTexts[rawIdentifier(id)]
    }

    /// Helper for a XCTAssert on the current content of the composer's text view.
    func assertTextViewContent(_ content: @autoclosure () throws -> String,
                               _ message: @autoclosure () -> String = "",
                               file: StaticString = #filePath,
                               line: UInt = #line) {
        guard var text = textView.value as? String else {
            XCTFail("Unable to retrieve text view content")
            return
        }
        // Remove occurences of ZWSP to avoid issues with expected content.
        text = text.replacingOccurrences(of: "\u{200B}", with: "")
        XCTAssertEqual(text, try content(), message(), file: file, line: line)
    }

    /// Focus the composer text view inside given app and
    /// clear the tutorial for keyboard swipe if it is displayed.
    func focusComposerAndClearTutorialIfNeeded() throws {
        textView.tap()
        let continueButton = app.buttons["Continue"]
        // If a continue button exists, we are on the keyboard Swipe tutorial.
        if continueButton.exists {
            continueButton.tap()
        }
    }

    /// Get the raw value of an UI element accessibility identifier
    ///
    /// - Parameter id: accessibility identifier of the UI element
    /// - Returns: raw string value
    func rawIdentifier(_ id: WysiwygSharedAccessibilityIdentifier) -> String {
        id.rawValue
    }
    
    /// Check if the current tree content of the text view is equal to provided content
    ///
    /// - Parameter content: the tree content to assert, must be provided without newlines at the start and at the end.
    func assertTreeEquals(_ content: String) {
        sleep(1)
        XCTAssertEqual(staticText(.treeText).label, "\n\(content)\n")
    }

    /// Assert that a Pill for given `displayName` currently
    /// exists in the text view and that the label matches.
    ///
    /// - Parameter displayName: The display name for the Pill.
    func assertMatchingPill(_ displayName: String) {
        let pill = textView.staticTexts["WysiwygAttachmentViewLabel" + displayName]
        XCTAssertTrue(pill.exists)
        XCTAssertEqual(pill.label, displayName)
    }
    
    func assertContentText(plainText: String, htmlText: String) {
        XCTAssert(staticText(.contentText).label == plainText)
        XCTAssert(staticText(.htmlContentText).label == htmlText)
    }
}

extension XCUIElement {
    /// Types a text inside the UI element character by character.
    /// This is especially useful to avoid missing some characters on
    /// UI tests running on a rather slow CI.
    ///
    /// - Parameters:
    ///   - text: Text to type in the UI element.
    func typeTextCharByChar(_ text: String) {
        text.forEach { self.typeText(String($0)) }
    }
}

private extension CGFloat {
    func roundedToFirstTwoDigits() -> CGFloat {
        CGFloat(self * 100).rounded() / 100.0
    }
}
