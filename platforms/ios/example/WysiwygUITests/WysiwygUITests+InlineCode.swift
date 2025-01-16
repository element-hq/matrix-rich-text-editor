//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import XCTest

extension WysiwygUITests {
    func testTypingInlineCodeDisablesOtherFormatters() {
        button(.inlineCodeButton).tap()
        textView.typeTextCharByChar("code")
        let reactiveButtonsIdentifiers: [WysiwygSharedAccessibilityIdentifier] = [
            .boldButton,
            .italicButton,
            .strikeThroughButton,
            .linkButton,
            // FIXME: this should contain other incompatible buttons when Rust is ready
        ]
        for identifier in reactiveButtonsIdentifiers {
            XCTAssertFalse(button(identifier).isEnabled)
        }
        // Inline code is enabled
        XCTAssertTrue(button(.inlineCodeButton).isEnabled)
    }
}
