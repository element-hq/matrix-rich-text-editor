//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import SnapshotTesting

final class ListsSnapshotTests: SnapshotTests {
    func testOrderedListContent() throws {
        viewModel.setHtmlContent("<ol><li>Item 1</li><li>Item 2</li></ol><p>Standard text</p>")
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }

    func testUnorderedListContent() throws {
        viewModel.setHtmlContent("<ul><li>Item 1</li><li>Item 2</li></ul><p>Standard text</p>")
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }

    func testMultipleListsContent() throws {
        viewModel.setHtmlContent(
            """
            <ol><li>Item 1</li><li>Item2</li></ol>\
            <ul><li>Item 1</li><li>Item2</li></ul>
            """
        )
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }

    func testIndentedListContent() throws {
        viewModel.setHtmlContent(
            """
            <ol><li>Item 1</li><li><p>Item 2</p>\
            <ol><li>Item 2A</li><li>Item 2B</li><li>Item 2C</li></ol>\
            </li><li>Item 3</li></ol>
            """
        )
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }

    func testListInQuote() throws {
        viewModel.setHtmlContent(
            """
            <blockquote>\
            <ol><li>Item 1</li><li>Item 2</li></ol>\
            <p>Some text</p>\
            </blockquote>
            """
        )
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }
}
