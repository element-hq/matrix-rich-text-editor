//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

enum TestConstants {
    /// Test string with emojis inputed both with codepoints and Xcode emoji insertion.
    /// String is actually 6 char long "abc🎉🎉👩🏿‍🚀" and represents 14 UTF-16 code units (3+2+2+7)
    static let testStringWithEmojis = "abc🎉\u{1f389}\u{1F469}\u{1F3FF}\u{200D}\u{1F680}"
    static let testStringAfterBackspace = "abc🎉🎉"
}
