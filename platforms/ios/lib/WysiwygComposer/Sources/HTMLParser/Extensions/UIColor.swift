//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import UIKit

public extension UIColor {
    func toHexString(shouldIncludeAlpha: Bool = false) -> String {
        var r: CGFloat = 0
        var g: CGFloat = 0
        var b: CGFloat = 0
        var a: CGFloat = 0
        getRed(&r, green: &g, blue: &b, alpha: &a)
        let hexValue: Int
        if shouldIncludeAlpha {
            hexValue = Int(r * 255) << 24 | Int(g * 255) << 16 | Int(b * 255) << 8 | Int(a * 255) << 0
            return String(format: "#%08x", hexValue)
        } else {
            hexValue = Int(r * 255) << 16 | Int(g * 255) << 8 | Int(b * 255) << 0
            return String(format: "#%06x", hexValue)
        }
    }
}
