//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import SnapshotTesting

final class LinksSnapshotTests: SnapshotTests {
    func testLinkContent() throws {
        viewModel.setHtmlContent("<a href=\"https://element.io\">test</a>")
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }
}
