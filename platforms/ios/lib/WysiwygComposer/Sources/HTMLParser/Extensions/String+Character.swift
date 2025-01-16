//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

public extension String {
    /// String containing a single NBSP character (`\u{00A0}`)
    static let nbsp = "\u{00A0}"
    /// String containing a single ZWSP character (`\u{200B}`)
    static let zwsp = "\u{200B}"
    /// String containing a single line separator character (`\u{2028}`)
    static let lineSeparator = "\u{2028}"
    /// String containing a single carriage return character (`\r`)
    static let carriageReturn = "\r"
    /// String containing a single line feed character (`\n`)
    static let lineFeed = "\n"
    /// String containing a single slash character(`/`)
    static let slash = "/"
    /// String containing an object replacement character.
    static let object = "\u{FFFC}"
}

public extension Character {
    /// NBSP character (`\u{00A0}`)
    static let nbsp = Character(.nbsp)
    /// ZWSP character (`\u{200B}`)
    static let zwsp = Character(.zwsp)
    /// Line feed character (`\n`)
    static let lineFeed = Character(.lineFeed)
    /// Slash character(`/`)
    static let slash = Character(.slash)
    /// Object replacement character.
    static let object = Character(.object)
}
