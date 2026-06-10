//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import SnapshotTesting
import Testing
import UIKit
@testable import WysiwygComposer

@MainActor
struct ListsSnapshotTests {
    let isRecord = SnapshotScene.isRecord
    let viewModel: WysiwygComposerViewModel
    let hostingController: UIViewController

    init() {
        (viewModel, hostingController) = SnapshotScene.make()
    }

    @Test func orderedListContent() {
        viewModel.setHtmlContent("<ol><li>Item 1</li><li>Item 2</li></ol><p>Standard text</p>")
        assertSnapshot(
            of: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }

    @Test func unorderedListContent() {
        viewModel.setHtmlContent("<ul><li>Item 1</li><li>Item 2</li></ul><p>Standard text</p>")
        assertSnapshot(
            of: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }

    @Test func multipleListsContent() {
        viewModel.setHtmlContent(
            """
            <ol><li>Item 1</li><li>Item2</li></ol>\
            <ul><li>Item 1</li><li>Item2</li></ul>
            """
        )
        assertSnapshot(
            of: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }

    @Test func indentedListContent() {
        viewModel.setHtmlContent(
            """
            <ol><li>Item 1</li><li><p>Item 2</p>\
            <ol><li>Item 2A</li><li>Item 2B</li><li>Item 2C</li></ol>\
            </li><li>Item 3</li></ol>
            """
        )
        assertSnapshot(
            of: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }

    @Test func listInQuote() {
        viewModel.setHtmlContent(
            """
            <blockquote>\
            <ol><li>Item 1</li><li>Item 2</li></ol>\
            <p>Some text</p>\
            </blockquote>
            """
        )
        assertSnapshot(
            of: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }
}
