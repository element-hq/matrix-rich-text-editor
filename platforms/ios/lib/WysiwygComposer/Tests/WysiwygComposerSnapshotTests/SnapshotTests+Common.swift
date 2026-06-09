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
struct CommonSnapshotTests {
    let isRecord = SnapshotScene.isRecord
    let viewModel: WysiwygComposerViewModel
    let hostingController: UIViewController

    init() {
        (viewModel, hostingController) = SnapshotScene.make()
    }

    @Test func clearState() {
        assertSnapshot(
            of: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }

    @Test func plainTextContent() {
        viewModel.setHtmlContent("Test")
        assertSnapshot(
            of: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }
}
