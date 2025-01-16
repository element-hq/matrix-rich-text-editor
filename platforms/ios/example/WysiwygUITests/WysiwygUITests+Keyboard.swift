//
// Copyright 2024 New Vector Ltd.
// Copyright 2024 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import XCTest

// These tests work on the assunmption that we always have the software keyboard enabled which is handled through a build phase run script.
// The following tests may also require specific keyboard languages that will be automatically added if needed.
extension WysiwygUITests {
    func testInlinePredictiveText() {
        sleep(1)
        setupKeyboard(.englishQWERTY)
        
        // Sometimes autocorrection can break capitalisation, so we need to make sure the first letter is lowercase
        app.keyboards.buttons["shift"].tap()
        app.typeTextCharByCharUsingKeyboard("hello how a")
        // We assert both the tree and textview content because the text view is containing the predictive text at that moment
        // Which in the ui test is seen as part of the static text
        assertTextViewContent("hello how are you")
        assertTreeEquals(
            """
            └>"hello how a"
            """
        )
        app.keys["space"].tap()
        assertTextViewContent("hello how are you ")
        assertTreeEquals(
            """
            └>"hello how are you "
            """
        )
    }
    
    func testInlinePredictiveTextIsIgnoredWhenSending() {
        sleep(1)
        setupKeyboard(.englishQWERTY)

        // Sometimes autocorrection can break capitalisation, so we need to make sure the first letter is lowercase
        app.keyboards.buttons["shift"].tap()
        app.typeTextCharByCharUsingKeyboard("hello how")
        // We assert both the tree and textview content because the text view is containing the predictive text at that moment
        // Which in the ui test is seen as part of the static text
        assertTextViewContent("hello how are you")
        assertTreeEquals(
            """
            └>"hello how"
            """
        )
        button(.sendButton).tap()
        sleep(1)
        assertContentText(plainText: "hello how", htmlText: "hello how")
    }
    
    func testInlinePredictiveTextIsIgnoredWhenDeleting() {
        sleep(1)
        setupKeyboard(.englishQWERTY)

        // Sometimes autocorrection can break capitalisation, so we need to make sure the first letter is lowercase
        app.keyboards.buttons["shift"].tap()
        app.typeTextCharByCharUsingKeyboard("hello how")
        app.keys["delete"].tap()
        // We assert both the tree and textview content because the text view is containing the predictive text at that moment
        // Which in the ui test is seen as part of the static text
        assertTextViewContent("hello how are you")
        assertTreeEquals(
            """
            └>"hello ho"
            """
        )
        button(.sendButton).tap()
        sleep(1)
        assertContentText(plainText: "hello ho", htmlText: "hello ho")
    }
    
    func testDoubleSpaceIntoDot() {
        sleep(1)
        setupKeyboard(.englishQWERTY)

        // Sometimes autocorrection can break capitalisation, so we need to make sure the first letter is lowercase
        app.keyboards.buttons["shift"].tap()
        app.typeTextCharByCharUsingKeyboard("hello")
        app.keys["space"].tap()
        app.keys["space"].tap()
        assertTextViewContent("hello. ")
        assertTreeEquals(
            """
            └>"hello. "
            """
        )
    }
    
    // This test only works on a real device, but not on simulator
    func disabled_testDotAfterInlinePredictiveText() {
        sleep(1)
        setupKeyboard(.englishQWERTY)

        // Sometimes autocorrection can break capitalisation, so we need to make sure the first letter is lowercase
        app.keyboards.buttons["shift"].tap()
        app.typeTextCharByCharUsingKeyboard("hello how a")
        // We assert both the tree and textview content because the text view is containing the predictive text at that moment
        // Which in the ui test is seen as part of the static text
        assertTextViewContent("hello how are you")
        app.keys["space"].tap()
        app.keys["more"].tap()
        app.keys["."].tap()
        
        // This optimisation to predictive inline text was introduced in 17.5
        let correctText: String
        if #available(iOS 17.5, *) {
            correctText = "hello how are you."
        } else {
            correctText = "hello how are you ."
        }
        assertTextViewContent(correctText)
        // In the failure case a second dot is added in the tree.
        assertTreeEquals(
            """
            └>"\(correctText)"
            """
        )
    }
    
    func testJapaneseKanaDeletion() {
        sleep(1)
        setupKeyboard(.japaneseKana)

        app.typeTextCharByCharUsingKeyboard("は")
        assertTextViewContent("は")
        assertTreeEquals(
            """
            └>"は"
            """
        )
        app.keys["delete"].tap()
        assertTextViewContent("")
        XCTAssertEqual(staticText(.treeText).label, "\n")
    }
    
    private func setupKeyboard(_ keyboard: TestKeyboard) {
        var changeKeyboardButton: XCUIElement!
        // If only 1 language + emoji keyboards are present the emoji button is used to change language
        // otherwise the button next keyboard button will be present instead
        let nextKeyboard = app.buttons["Next keyboard"]
        let emoji = app.buttons["Emoji"]
        if nextKeyboard.exists {
            changeKeyboardButton = nextKeyboard
        } else if emoji.exists {
            changeKeyboardButton = emoji
        }
        
        if changeKeyboardButton == nil {
            addKeyboardToSettings(keyboard: keyboard)
            return
        }
        
        changeKeyboardButton.press(forDuration: 1)
        let keyboardSelection = app.tables.staticTexts[keyboard.label]
        if !keyboardSelection.exists {
            addKeyboardToSettings(keyboard: keyboard)
            return
        }
        keyboardSelection.tap()
    }
    
    private func addKeyboardToSettings(keyboard: TestKeyboard) {
        let settingsApp = XCUIApplication(bundleIdentifier: "com.apple.Preferences")
        settingsApp.launch()
        
        settingsApp.tables.cells.staticTexts["General"].tap()
        settingsApp.tables.cells.staticTexts["Keyboard"].tap()
        settingsApp.tables.cells.staticTexts["Keyboards"].tap()
        if settingsApp.tables.cells.staticTexts[keyboard.keyboardIdentifier].exists {
            return
        }
        settingsApp.tables.cells.staticTexts["AddNewKeyboard"].tap()
        settingsApp.tables.cells.staticTexts[keyboard.localeIdentifier].tap()
        if keyboard.hasSubSelection {
            settingsApp.tables.cells.staticTexts[keyboard.keyboardIdentifier].tap()
        }
        settingsApp.buttons["Done"].tap()
        sleep(1)
        settingsApp.terminate()
        
        app.launch()
        textView.tap()
        sleep(1)
        
        setupKeyboard(keyboard)
    }
}

private extension XCUIApplication {
    func typeTextCharByCharUsingKeyboard(_ text: String) {
        for char in text {
            if char == " " {
                keys["space"].tap()
                continue
            }
            keys[String(char)].tap()
        }
    }
}

private enum TestKeyboard {
    case englishQWERTY
    case japaneseKana
    
    var keyboardIdentifier: String {
        switch self {
        case .englishQWERTY:
            return "en_US@sw=QWERTY;hw=Automatic"
        case .japaneseKana:
            return "ja_JP-Kana@sw=Kana;hw=Automatic"
        }
    }
    
    var localeIdentifier: String {
        switch self {
        case .englishQWERTY:
            return "en_US"
        case .japaneseKana:
            return "ja_JP"
        }
    }
    
    var label: String {
        switch self {
        case .englishQWERTY:
            return "English (US)"
        case .japaneseKana:
            return "日本語かな"
        }
    }
    
    var hasSubSelection: Bool {
        switch self {
        case .englishQWERTY:
            return false
        case .japaneseKana:
            return true
        }
    }
}
