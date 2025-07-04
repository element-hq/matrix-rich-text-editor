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
    func testLists() {
        ComposerModelWrapper()
            .action { $0.apply(.orderedList) }
            .action { $0.replaceText(newText: "Item 1") }
            .action { $0.enter() }
            .action { $0.replaceText(newText: "Item 2") }
            // Add a third list item
            .action { $0.enter() }
            .assertHtml("<ol><li>Item 1</li><li>Item 2</li><li></li></ol>")
            .assertSelection(start: 14, end: 14)
            // Remove it
            .action { $0.enter() }
            .assertHtml("<ol><li>Item 1</li><li>Item 2</li></ol><p>\(Character.nbsp)</p>")
            .assertSelection(start: 14, end: 14)
            // Insert some text afterwards
            .action { $0.replaceText(newText: "Some text") }
            .assertHtml("<ol><li>Item 1</li><li>Item 2</li></ol><p>Some text</p>")
            .assertSelection(start: 23, end: 23)
    }
}
