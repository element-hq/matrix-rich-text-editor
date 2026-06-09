//
// Copyright 2024 New Vector Ltd.
// Copyright 2024 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Testing
@testable import WysiwygComposer

struct StringLatinLangugesTests {
    @Test func latinLangugeCharacters() {
        #expect("hello".containsLatinAndCommonCharactersOnly)
        #expect("helló".containsLatinAndCommonCharactersOnly)
        #expect("helló, ".containsLatinAndCommonCharactersOnly)
        #expect("helló, ".containsLatinAndCommonCharactersOnly)
        #expect("😄🛴🤯❤️".containsLatinAndCommonCharactersOnly)
        // Test the object replacement character as defined in String+Character extension.
        #expect(String.object.containsLatinAndCommonCharactersOnly)
        #expect("!@££$%^&*()".containsLatinAndCommonCharactersOnly)

        #expect(!"你好".containsLatinAndCommonCharactersOnly)
        #expect(!"感^".containsLatinAndCommonCharactersOnly)
        #expect(!"Меня зовут Маша".containsLatinAndCommonCharactersOnly)
        #expect(!"ฉันชอบกินข้าวผัด แต่เธอชอบกินผัดไทย".containsLatinAndCommonCharactersOnly)
        #expect(!"ni3好^".containsLatinAndCommonCharactersOnly)
    }
}
