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
struct BlocksSnapshotTests {
    let isRecord = SnapshotScene.isRecord
    let viewModel: WysiwygComposerViewModel
    let hostingController: UIViewController

    init() {
        (viewModel, hostingController) = SnapshotScene.make()
    }

    @Test func inlineCodeContent() {
        viewModel.setHtmlContent("<code>test</code>")
        assertSnapshot(
            of: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }

    @Test func codeBlockContent() {
        viewModel.setHtmlContent("<pre><code>if snapshot {\n\treturn true\n}</code></pre>")
        assertSnapshot(
            of: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }

    @Test func quoteContent() {
        viewModel.setHtmlContent("<blockquote>Some quote with<br/><br/><br/><br/>line breaks inside</blockquote>")
        assertSnapshot(
            of: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }

    @Test func multipleBlocksContent() {
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
            of: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }
}
