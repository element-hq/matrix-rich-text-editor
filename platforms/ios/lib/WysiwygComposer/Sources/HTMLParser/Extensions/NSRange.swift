//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Foundation

extension NSRange {
    /// Returns an `NSRange` with the length reduced by 1.
    var excludingLast: NSRange {
        .init(location: location, length: length - 1)
    }
}
