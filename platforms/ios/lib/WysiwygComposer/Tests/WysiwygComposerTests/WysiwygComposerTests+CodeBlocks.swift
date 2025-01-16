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
    static let resultHtml = "<pre><code>Some code\n\tmore code</code></pre><p>\(Character.nbsp)</p>"
    static let resultTree =
        """

        ├>codeblock
        │ ├>p
        │ │ └>"Some code"
        │ └>p
        │   └>"\tmore code"
        └>p

        """
}

extension WysiwygComposerTests {
    func testCodeBlocksFromEmptyComposer() {
        ComposerModelWrapper()
            .action { $0.apply(.codeBlock) }
            .action { $0.replaceText(newText: "Some code") }
            .action { $0.enter() }
            .action { $0.replaceText(newText: "\t") }
            .action { $0.replaceText(newText: "more code") }
            .action { $0.enter() }
            .action { $0.enter() }
            .assertHtml(Constants.resultHtml)
            .assertTree(Constants.resultTree)
    }

    func testCodeBlocksWithMultilineInput() {
        ComposerModelWrapper()
            .action { $0.apply(.codeBlock) }
            .action { $0.replaceText(newText: "Some code\n\tmore code") }
            .action { $0.enter() }
            .action { $0.enter() }
            .assertHtml(Constants.resultHtml)
            .assertTree(Constants.resultTree)
    }

    func testCodeBlocksFromContent() {
        ComposerModelWrapper()
            .action { $0.replaceText(newText: "Some code") }
            .action { $0.apply(.codeBlock) }
            .action { $0.enter() }
            .action { $0.replaceText(newText: "\t") }
            .action { $0.replaceText(newText: "more code") }
            .action { $0.enter() }
            .action { $0.enter() }
            .assertHtml(Constants.resultHtml)
            .assertTree(Constants.resultTree)
    }
}
