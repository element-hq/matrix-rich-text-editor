//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import UIKit

/// Temporary color used to detect the range of the HTML element inside the attributed string.
enum TempColor {
    static let inlineCode: UIColor = .red
    static let codeBlock: UIColor = .green
    static let quote: UIColor = .blue
}
