//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

@testable import WysiwygComposer
import XCTest

extension WysiwygComposerTests {
    func testInlineCode() {
        ComposerModelWrapper()
            .action { $0.apply(.inlineCode) }
            .action { $0.replaceText(newText: "code") }
            .assertTree(
                """

                └>code
                  └>\"code\"

                """
            )
    }

    func testInlineCodeWithFormatting() {
        ComposerModelWrapper()
            .action { $0.apply(.bold) }
            .action { $0.replaceText(newText: "bold") }
            // This should get ignored
            .action { $0.apply(.italic) }
            .action { $0.apply(.inlineCode) }
            .action { $0.replaceText(newText: "code") }
            .assertTree(
                """

                ├>strong
                │ └>"bold"
                └>code
                  └>"code"

                """
            )
    }
}
