//
// Copyright 2024 New Vector Ltd.
// Copyright 2024 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

@testable import WysiwygComposer
import XCTest

final class StringLatinLangugesTests: XCTestCase {
    func testLatinLangugeCharacters() {
        XCTAssertTrue("hello".containsLatinAndCommonCharactersOnly)
        XCTAssertTrue("helló".containsLatinAndCommonCharactersOnly)
        XCTAssertTrue("helló, ".containsLatinAndCommonCharactersOnly)
        XCTAssertTrue("helló, ".containsLatinAndCommonCharactersOnly)
        XCTAssertTrue("😄🛴🤯❤️".containsLatinAndCommonCharactersOnly)
        // Test the object replacement character as defined in String+Character extension.
        XCTAssertTrue(String.object.containsLatinAndCommonCharactersOnly)
        XCTAssertTrue("!@££$%^&*()".containsLatinAndCommonCharactersOnly)
        
        XCTAssertFalse("你好".containsLatinAndCommonCharactersOnly)
        XCTAssertFalse("感^".containsLatinAndCommonCharactersOnly)
        XCTAssertFalse("Меня зовут Маша".containsLatinAndCommonCharactersOnly)
        XCTAssertFalse("ฉันชอบกินข้าวผัด แต่เธอชอบกินผัดไทย".containsLatinAndCommonCharactersOnly)
        XCTAssertFalse("ni3好^".containsLatinAndCommonCharactersOnly)
    }
}
