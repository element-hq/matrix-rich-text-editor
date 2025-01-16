//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import XCTest

extension WysiwygUITests {
    /// Type a text and make it bold in the composer.
    /// A screenshot is saved since string attributes can't be read from this context.
    func testTypingAndBolding() throws {
        // Type something into composer.
        textView.typeTextCharByChar("Some bold text")

        textView.doubleTap()
        // We can't detect data being properly reported back to the model but
        // 1s is more than enough for the Rust side to get notified for the selection.
        sleep(1)

        button(.boldButton).tap()
        // Bolding doesn't change text and we can't test text attributes from this context.
        assertTextViewContent("Some bold text")

        assertTreeEquals(
            """
            ├>"Some bold "
            └>strong
              └>"text"
            """
        )
    }
}
