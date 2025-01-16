//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import XCTest

extension WysiwygUITests {
    func testIndent() {
        textView.typeTextCharByChar("Item 1")
        button(.orderedListButton).tap()
        textView.typeTextCharByChar("\nItem 2")
        // Indent second list item.
        button(.indentButton).tap()
        assertTreeEquals(
            """
            └>ol
              └>li
                ├>p
                │ └>"Item 1"
                └>ol
                  └>li
                    └>"Item 2"
            """
        )
        XCTAssertFalse(button(.indentButton).isEnabled)
        // Transform indented list item into unordered.
        button(.unorderedListButton).tap()
        assertTreeEquals(
            """
            └>ol
              └>li
                ├>p
                │ └>"Item 1"
                └>ul
                  └>li
                    └>"Item 2"
            """
        )
        // Unindent second list item.
        button(.unindentButton).tap()
        assertTreeEquals(
            """
            └>ol
              ├>li
              │ └>"Item 1"
              └>li
                └>"Item 2"
            """
        )
        XCTAssertFalse(button(.unindentButton).isEnabled)
    }
}
