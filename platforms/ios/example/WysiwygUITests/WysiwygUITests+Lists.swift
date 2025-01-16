//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import XCTest

extension WysiwygUITests {
    func testList() {
        textView.typeTextCharByChar("Item 1")
        // Create list and add a second item.
        button(.orderedListButton).tap()
        textView.typeTextCharByChar("\nItem 2")
        assertTreeEquals(
            """
            └>ol
              ├>li
              │ └>"Item 1"
              └>li
                └>"Item 2"
            """
        )
        // Transform list into unordered.
        button(.unorderedListButton).tap()
        assertTreeEquals(
            """
            └>ul
              ├>li
              │ └>"Item 1"
              └>li
                └>"Item 2"
            """
        )
    }
}
