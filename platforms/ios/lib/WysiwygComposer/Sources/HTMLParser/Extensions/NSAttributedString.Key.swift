//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import DTCoreText
import Foundation

extension NSAttributedString.Key {
    // MARK: - DTCoreText Internal Keys

    static let DTTextBlocks: NSAttributedString.Key = .init(rawValue: DTTextBlocksAttribute)
    static let DTField: NSAttributedString.Key = .init(rawValue: DTFieldAttribute)
    static let DTTextLists: NSAttributedString.Key = .init(rawValue: DTTextListsAttribute)

    // MARK: - Custom Keys

    /// Attribute for parts of the string that require some custom drawing (e.g. code blocks, quotes).
    static let blockStyle: NSAttributedString.Key = .init(rawValue: "BlockStyleAttributeKey")
    /// Attribute for parts of the string that should be removed for HTML selection computation.
    /// Should include both placeholder characters such as NBSP and ZWSP, as well as list prefixes.
    static let discardableText: NSAttributedString.Key = .init(rawValue: "DiscardableAttributeKey")
    /// Attribute for a mention It contains data the composer requires in order to compute the expected HTML/attributed range properly.
    static let mention: NSAttributedString.Key = .init(rawValue: "mentionAttributeKey")
}
