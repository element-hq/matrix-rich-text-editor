//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import XCTest

extension WysiwygUITests {
    func testCreateLinkWithTextEditAndRemove() {
        // Create with text
        button(.linkButton).tap()
        XCTAssertTrue(textField(.linkUrlTextField).exists)
        XCTAssertTrue(textField(.linkTextTextField).exists)
        textField(.linkUrlTextField).typeTextCharByChar("url")
        textField(.linkTextTextField).tap()
        textField(.linkTextTextField).typeTextCharByChar("text")
        app.buttons["Ok"].tap()
        assertTreeEquals(
            """
            └>a "https://url"
              └>"text"
            """
        )

        // Edit
        button(.linkButton).tap()
        XCTAssertFalse(textField(.linkTextTextField).exists)
        textField(.linkUrlTextField).doubleTap()
        textField(.linkUrlTextField).typeTextCharByChar("new_url")
        app.buttons["Ok"].tap()
        assertTreeEquals(
            """
            └>a "https://new_url"
              └>"text"
            """
        )

        // Remove
        button(.linkButton).tap()
        XCTAssertFalse(textField(.linkTextTextField).exists)
        app.buttons["Remove"].tap()
        assertTreeEquals(
            """
            └>"text"
            """
        )
    }

    func testCreateLinkFromSelection() {
        textView.typeTextCharByChar("text")
        assertTreeEquals(
            """
            └>"text"
            """
        )

        textView.doubleTap()
        button(.linkButton).tap()
        XCTAssertFalse(textField(.linkTextTextField).exists)
        textField(.linkUrlTextField).typeTextCharByChar("url")
        app.buttons["Ok"].tap()
        assertTreeEquals(
            """
            └>a "https://url"
              └>"text"
            """
        )
    }
}
