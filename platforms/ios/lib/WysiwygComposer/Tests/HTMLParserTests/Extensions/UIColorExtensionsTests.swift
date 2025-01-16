//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

@testable import HTMLParser
import XCTest

final class UIColorExtensionsTests: XCTestCase {
    func testConversionInHexWithoutAlpha() {
        let red = UIColor.red
        XCTAssertEqual(red.toHexString(), "#ff0000")
        let blue = UIColor.blue
        XCTAssertEqual(blue.toHexString(), "#0000ff")
        let green = UIColor.green
        XCTAssertEqual(green.toHexString(), "#00ff00")
        let black = UIColor.black
        XCTAssertEqual(black.toHexString(), "#000000")
        let white = UIColor.white
        XCTAssertEqual(white.toHexString(), "#ffffff")
        let color = UIColor(red: 17.0 / 255.0, green: 11.0 / 255.0, blue: 64.0 / 255.0, alpha: 1.0)
        XCTAssertEqual(color.toHexString(), "#110b40")
    }
    
    func testConversionInHexWithAlpha() {
        let black = UIColor.black
        XCTAssertEqual(black.toHexString(shouldIncludeAlpha: true), "#000000ff")
        let clear = UIColor.clear
        XCTAssertEqual(clear.toHexString(shouldIncludeAlpha: true), "#00000000")
        let color = UIColor(red: 17.0 / 255.0, green: 11.0 / 255, blue: 64.0 / 255.0, alpha: 32 / 255.0)
        XCTAssertEqual(color.toHexString(shouldIncludeAlpha: true), "#110b4020")
    }
}
