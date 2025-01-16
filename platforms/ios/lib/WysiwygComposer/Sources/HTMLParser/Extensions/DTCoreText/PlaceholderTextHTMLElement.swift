//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import DTCoreText

/// Defines a placeholder to be inserted during HTML parsing in order to have a valid
/// position for e.g. an empty paragraph.
final class PlaceholderTextHTMLElement: DTTextHTMLElement {
    /// Init.
    ///
    /// - Parameters:
    ///   - textNode: text node that should be copied into the element.
    init(from textNode: DTTextHTMLElement) {
        super.init()
        setText(textNode.text())
    }

    override init() {
        super.init()
        setText(.nbsp)
    }

    override func attributesForAttributedStringRepresentation() -> [AnyHashable: Any]! {
        var dict = super.attributesForAttributedStringRepresentation() ?? [AnyHashable: Any]()
        // Insert a key to mark this as discardable post-parsing.
        dict[NSAttributedString.Key.discardableText] = true
        return dict
    }
}

final class MentionTextNodeHTMLElement: DTTextHTMLElement {
    init(from textNode: DTTextHTMLElement) {
        super.init()
        setText(textNode.text())
    }

    override func attributesForAttributedStringRepresentation() -> [AnyHashable: Any]! {
        var dict = super.attributesForAttributedStringRepresentation() ?? [AnyHashable: Any]()
        // Insert a key to mark this as a mention post-parsing.
        dict[NSAttributedString.Key.mention] = MentionContent(
            rustLength: 1,
            url: parent().attributes["href"] as? String ?? ""
        )
        return dict
    }
}
