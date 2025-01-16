//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import UIKit

extension NSParagraphStyle {
    /// Returns a mutable copy of self, casted as `NSMutableParagraphStyle`.
    func mut() -> NSMutableParagraphStyle {
        guard let mut = mutableCopy() as? NSMutableParagraphStyle else {
            return NSMutableParagraphStyle()
        }
        return mut
    }
}
