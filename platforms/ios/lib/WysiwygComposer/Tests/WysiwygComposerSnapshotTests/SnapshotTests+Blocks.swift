//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import SnapshotTesting

final class BlocksSnapshotTests: SnapshotTests {
    func testInlineCodeContent() throws {
        viewModel.setHtmlContent("<code>test</code>")
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }

    func testCodeBlockContent() throws {
        viewModel.setHtmlContent("<pre><code>if snapshot {\n\treturn true\n}</code></pre>")
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }

    func testQuoteContent() throws {
        viewModel.setHtmlContent("<blockquote>Some quote with<br/><br/><br/><br/>line breaks inside</blockquote>")
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }

    func testMultipleBlocksContent() throws {
        viewModel.setHtmlContent(
            """
            <blockquote>Some<br/>\
            multi-line<br/>\
            quote</blockquote>\
            <br/>\
            <br/>\
            Some text<br/>\
            <br/>\
            <pre>A\n\tcode\nblock</pre>\
            <br/>\
            <br/>\
            Some <code>inline</code> code
            """
        )
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }
}
