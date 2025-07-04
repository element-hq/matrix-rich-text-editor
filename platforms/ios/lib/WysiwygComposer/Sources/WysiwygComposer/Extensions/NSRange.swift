//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Foundation

public extension NSRange {
    /// Returns a range at starting location, i.e. {0, 0}.
    static let zero = Self(location: 0, length: 0)
}
