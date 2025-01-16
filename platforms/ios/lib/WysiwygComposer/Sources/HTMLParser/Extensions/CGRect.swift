//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import CoreGraphics

extension CGRect {
    /// Return a copy of the rect extended to the leading and trailing borders of given frame.
    ///
    /// - Parameters:
    ///   - frame: frame to extend into.
    ///   - verticalPadding: padding to apply vertically
    func extendHorizontally(in frame: CGRect, withVerticalPadding verticalPadding: CGFloat) -> CGRect {
        CGRect(x: frame.minX, y: minY - verticalPadding, width: frame.width, height: height + 2 * verticalPadding)
    }
}
