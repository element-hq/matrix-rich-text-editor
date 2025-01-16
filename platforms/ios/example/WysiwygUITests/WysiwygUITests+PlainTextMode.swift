//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import XCTest

extension WysiwygUITests {
    func testMarkdownFromPlainTextModeIsParsed() throws {
        textView.typeTextCharByChar("text ")
        button(.plainRichButton).tap()
        textView.typeTextCharByChar("__bold__ *italic*")
        assertTextViewContent("text __bold__ *italic*")
        button(.plainRichButton).tap()
        assertTextViewContent("text bold italic")
        assertTreeEquals(
            """
            ├>"text "
            ├>strong
            │ └>"bold"
            ├>" "
            └>em
              └>"italic"
            """
        )
        // Re-toggling restores the markdown.
        button(.plainRichButton).tap()
        assertTextViewContent("text __bold__ *italic*")
    }

    // FIXME: disabled for now, should be re-enabled when this is supported
    func testPlainTextModePreservesPills() throws {
        // Create a Pill in RTE.
        textView.typeTextCharByChar("@ali")
        button(.aliceButton).tap()
        // Switch to plain text mode and assert Pill exists
        button(.plainRichButton).tap()
        assertMatchingPill("Alice")
        // Write something.
        textView.typeTextCharByChar("hello")
        // Switch back to RTE and assert model.
        button(.plainRichButton).tap()
        assertMatchingPill("Alice")
        assertTreeEquals(
            """
            ├>a "https://matrix.to/#/@alice:matrix.org"
            │ └>"Alice"
            └>" hello"
            """
        )
    }
}
