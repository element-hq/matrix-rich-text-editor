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
    /// A list with three items.
    static let sampleListHtml = "<ol><li>Item 1</li><li>Item 2</li><li>Item 3</li></ol>"
    /// A list with three items. Second item is indented, Third item is indented twice.
    static let indentedSampleListHtml = """
    <ol><li><p>Item 1</p>\
    <ol><li><p>Item 2</p>\
    <ol><li>Item 3</li></ol></li></ol></li></ol>
    """
}

extension WysiwygComposerTests {
    func testIndent() {
        ComposerModelWrapper()
            .action { $0.setContentFromHtml(html: Constants.sampleListHtml) }
            // Select somewhere on item 2
            .action { $0.select(startUtf16Codeunit: 9, endUtf16Codeunit: 9) }
            .action { $0.apply(.indent) }
            .execute { XCTAssertTrue($0.actionStates()[.indent] == .disabled) }
            // Select somewhere on item 3
            .action { $0.select(startUtf16Codeunit: 18, endUtf16Codeunit: 18) }
            .action { $0.apply(.indent) }
            .action { $0.apply(.indent) }
            .execute { XCTAssertTrue($0.actionStates()[.indent] == .disabled) }
            .assertHtml(Constants.indentedSampleListHtml)
    }

    func testUnindent() {
        ComposerModelWrapper()
            .action { $0.setContentFromHtml(html: Constants.indentedSampleListHtml) }
            // Select somewhere on item 3
            .action { $0.select(startUtf16Codeunit: 18, endUtf16Codeunit: 18) }
            .action { $0.apply(.unindent) }
            .action { $0.apply(.unindent) }
            .execute { XCTAssertTrue($0.actionStates()[.unindent] == .disabled) }
            // Select somewhere on item 2
            .action { $0.select(startUtf16Codeunit: 9, endUtf16Codeunit: 9) }
            .action { $0.apply(.unindent) }
            .execute { XCTAssertTrue($0.actionStates()[.unindent] == .disabled) }
            .assertHtml(Constants.sampleListHtml)
    }
}
