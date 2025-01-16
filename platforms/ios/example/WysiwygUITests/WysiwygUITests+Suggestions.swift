//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import XCTest

extension WysiwygUITests {
    func testAtMention() throws {
        textView.typeTextCharByChar("@ali")
        XCTAssertTrue(button(.aliceButton).exists)
        button(.aliceButton).tap()
        assertMatchingPill("Alice")
        // Mention is replaced by a pill view, so there
        // is only the space after it in the field.
        assertTextViewContent("￼\u{00A0}")
        assertTreeEquals(
            """
            ├>mention "Alice", https://matrix.to/#/@alice:matrix.org
            └>" "
            """
        )
    }

    func testHashMention() throws {
        textView.typeTextCharByChar("#roo")
        XCTAssertTrue(button(.room1Button).exists)
        button(.room1Button).tap()
        // FIXME: room links are not considered valid links through parsing, so no mention is displayed atm
        // assertTextViewContent("￼ ")
        assertTreeEquals(
            """
            ├>mention "Room 1", https://matrix.to/#/#room1:matrix.org
            └>" "
            """
        )
    }

    func testCommand() throws {
        textView.typeTextCharByChar("/inv")
        XCTAssertTrue(button(.inviteCommandButton).exists)
        button(.inviteCommandButton).tap()
        assertTextViewContent("/invite\u{00A0}")
        assertTreeEquals(
            """
            └>"/invite "
            """
        )
    }
}
