//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Foundation
import UIKit
import WysiwygComposer

final class WysiwygMentionReplacer: MentionReplacer {
    func replacementForMention(_ url: String, text: String) -> NSAttributedString? {
        if #available(iOS 15.0, *),
           url.starts(with: "https://matrix.to/#/"),
           let attachment = WysiwygTextAttachment(displayName: text,
                                                  url: url,
                                                  font: UIFont.preferredFont(forTextStyle: .body)) {
            return NSAttributedString(attachment: attachment)
        } else {
            return nil
        }
    }

    func postProcessMarkdown(in attributedString: NSAttributedString) -> NSAttributedString {
        // Create a regexp that detects markdown links.
        let pattern = "\\[([^\\]]+)\\]\\(([^\\)\"\\s]+)(?:\\s+\"(.*)\")?\\)"
        guard #available(iOS 15.0, *),
              let regExp = try? NSRegularExpression(pattern: pattern) else {
            return attributedString
        }

        let matches = regExp.matches(in: attributedString.string,
                                     range: .init(location: 0, length: attributedString.length))

        // If we have some matches, replace permalinks by a pill version.
        let mutable = NSMutableAttributedString(attributedString: attributedString)
        for match in matches.reversed() {
            let displayNameRange = match.range(at: 1)
            let urlRange = match.range(at: 2)
            let displayName = attributedString.attributedSubstring(from: displayNameRange).string
            var url = attributedString.attributedSubstring(from: urlRange).string

            // Note: a valid markdown link can be written with
            // enclosing <..>, remove them for url check.
            if url.first == "<", url.last == ">" {
                url = String(url[url.index(after: url.startIndex)...url.index(url.endIndex, offsetBy: -2)])
            }

            if url.starts(with: "https://matrix.to/#/"),
               let attachment = WysiwygTextAttachment(displayName: displayName,
                                                      url: url,
                                                      font: UIFont.preferredFont(forTextStyle: .body)) {
                mutable.replaceCharacters(in: match.range, with: NSAttributedString(attachment: attachment))
            }
        }

        return mutable
    }

    func restoreMarkdown(in attributedString: NSAttributedString) -> String {
        guard #available(iOS 15.0, *) else { return attributedString.string }

        let newAttr = NSMutableAttributedString(attributedString: attributedString)
        newAttr.enumerateTypedAttribute(.attachment) { (attachment: WysiwygTextAttachment, range: NSRange, _) in
            if let displayName = attachment.data?.displayName,
               let url = attachment.data?.url {
                let markdownString = "[\(displayName)](\(url))"
                newAttr.replaceCharacters(in: range, with: markdownString)
            }
        }

        return newAttr.string
    }
}
