//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

@testable import HTMLParser
import Testing
import UIKit

struct UIColorExtensionsTests {
    @Test func conversionInHexWithoutAlpha() {
        let red = UIColor.red
        #expect(red.toHexString() == "#ff0000")
        let blue = UIColor.blue
        #expect(blue.toHexString() == "#0000ff")
        let green = UIColor.green
        #expect(green.toHexString() == "#00ff00")
        let black = UIColor.black
        #expect(black.toHexString() == "#000000")
        let white = UIColor.white
        #expect(white.toHexString() == "#ffffff")
        let color = UIColor(red: 17.0 / 255.0, green: 11.0 / 255.0, blue: 64.0 / 255.0, alpha: 1.0)
        #expect(color.toHexString() == "#110b40")
    }

    @Test func conversionInHexWithAlpha() {
        let black = UIColor.black
        #expect(black.toHexString(shouldIncludeAlpha: true) == "#000000ff")
        let clear = UIColor.clear
        #expect(clear.toHexString(shouldIncludeAlpha: true) == "#00000000")
        let color = UIColor(red: 17.0 / 255.0, green: 11.0 / 255, blue: 64.0 / 255.0, alpha: 32 / 255.0)
        #expect(color.toHexString(shouldIncludeAlpha: true) == "#110b4020")
    }
}
