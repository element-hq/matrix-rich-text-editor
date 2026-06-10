//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import HTMLParser
import Testing
import UIKit
@testable import WysiwygComposer

extension WysiwygComposerTests {
    @Test func formatBold() {
        ComposerModelWrapper()
            .action { $0.replaceText(newText: "This is bold text") }
            .action { $0.select(startUtf16Codeunit: 8, endUtf16Codeunit: 12) }
            .action { $0.apply(.bold) }
            .assertHtml("This is <strong>bold</strong> text")
            // Selection is kept after format.
            .assertSelection(start: 8, end: 12)
            .execute {
                // Constructed attributed string sets bold on the selected range.
                guard let attributed = try? HTMLParser.parse(html: $0.getContentAsHtml()) else {
                    Issue.record("Parsing unexpectedly failed")
                    return
                }
                attributed.enumerateTypedAttribute(.font, in: .init(location: 8, length: 4)) { (font: UIFont, range, _) in
                    #expect(range == .init(location: 8, length: 4))
                    #expect(font.fontDescriptor.symbolicTraits.contains(.traitBold))
                }
            }
            .assertTree(
                """
                
                ├>\"This is \"
                ├>strong
                │ └>\"bold\"
                └>\" text\"
                
                """
            )
    }
}
