//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import XCTest

extension WysiwygUITests {
    func testCodeBlock() throws {
        // Type something into composer.
        textView.typeTextCharByChar("Some text")
        button(.codeBlockButton).tap()
        assertTextViewContent("Some text")

        assertTreeEquals(
            """
            └>codeblock
              └>p
                └>"Some text"
            """
        )
    }
}
