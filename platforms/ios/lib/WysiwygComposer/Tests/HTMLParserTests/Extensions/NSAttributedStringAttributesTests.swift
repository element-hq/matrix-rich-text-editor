//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

@testable import HTMLParser
import XCTest

final class NSAttributedStringTests: XCTestCase {
    func testEnumerateTypedAttribute() {
        let attributed = NSMutableAttributedString(string: "Test",
                                                   attributes: [.font: UIFont.boldSystemFont(ofSize: 15)])
        attributed.enumerateTypedAttribute(.font) { (font: UIFont, range: NSRange, _) in
            XCTAssertTrue(font.fontDescriptor.symbolicTraits.contains(.traitBold))
            XCTAssertTrue(range == .init(location: 0, length: attributed.length))
        }
        attributed.addAttribute(.font, value: "bad type", range: .init(location: 2, length: 2))
        attributed.enumerateTypedAttribute(.font) { (font: UIFont, range: NSRange, _) in
            XCTAssertTrue(font.fontDescriptor.symbolicTraits.contains(.traitBold))
            XCTAssertTrue(range == .init(location: 0, length: 2))
        }
    }
}
