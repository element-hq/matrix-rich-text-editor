//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

@testable import WysiwygComposer
import XCTest

private enum Constants {
    static let resultHtml = "<blockquote><p>Some quote</p><p>More text</p></blockquote><p>\(Character.nbsp)</p>"
    static let resultTree =
        """

        ├>blockquote
        │ ├>p
        │ │ └>"Some quote"
        │ └>p
        │   └>"More text"
        └>p

        """
}

extension WysiwygComposerTests {
    func testQuotesFromEmptyComposer() {
        ComposerModelWrapper()
            .action { $0.apply(.quote) }
            .action { $0.replaceText(newText: "Some quote") }
            .action { $0.enter() }
            .action { $0.replaceText(newText: "More text") }
            .action { $0.enter() }
            .action { $0.enter() }
            .assertHtml(Constants.resultHtml)
            .assertTree(Constants.resultTree)
    }

    func testQuotesWithMultilineInput() {
        ComposerModelWrapper()
            .action { $0.apply(.quote) }
            .action { $0.replaceText(newText: "Some quote\nMore text") }
            .action { $0.enter() }
            .action { $0.enter() }
            .assertHtml(Constants.resultHtml)
            .assertTree(Constants.resultTree)
    }

    func testQuotesFromContent() {
        ComposerModelWrapper()
            .action { $0.replaceText(newText: "Some quote") }
            .action { $0.apply(.quote) }
            .action { $0.enter() }
            .action { $0.replaceText(newText: "More text") }
            .action { $0.enter() }
            .action { $0.enter() }
            .assertHtml(Constants.resultHtml)
            .assertTree(Constants.resultTree)
    }
}
