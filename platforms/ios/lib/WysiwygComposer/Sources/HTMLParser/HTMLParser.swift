//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import DTCoreText
import UIKit

/// Provides tools to parse from HTML to NSAttributedString with a standard style.
public final class HTMLParser {
    // MARK: - Private
    
    private static var defaultCSS: String {
        """
        blockquote {
            background-color: \(TempColor.quote.toHexString());
            display: block;
        }
        pre {
            background-color: \(TempColor.codeBlock.toHexString());
            display: block;
            white-space: pre;
            -coretext-fontname: .AppleSystemUIFontMonospaced-Regular;
            font-size: inherit;
        }
        code {
            background-color: \(TempColor.inlineCode.toHexString());
            display: inline;
            white-space: pre;
            -coretext-fontname: .AppleSystemUIFontMonospaced-Regular;
            font-size: inherit;
        }
        h1,h2,h3 {
            font-size: 1.2em;
        }
        """
    }
    
    private init() { }
    
    // MARK: - Public
    
    /// Parse given HTML to NSAttributedString with a standard style.
    ///
    /// - Parameters:
    ///   - html: HTML to parse
    ///   - encoding: String encoding to use
    ///   - style: Style to apply for HTML parsing
    ///   - mentionReplacer:An object that might replace detected mentions.
    /// - Returns: An attributed string representation of the HTML content
    public static func parse(html: String,
                             encoding: String.Encoding = .utf16,
                             style: HTMLParserStyle = .standard,
                             mentionReplacer: HTMLMentionReplacer? = nil) throws -> NSAttributedString {
        guard !html.isEmpty else {
            return NSAttributedString(string: "")
        }

        guard let data = html.data(using: encoding) else {
            throw BuildHtmlAttributedError.dataError(encoding: encoding)
        }
        
        let defaultFont = UIFont.preferredFont(forTextStyle: .body)
        
        let parsingOptions: [String: Any] = [
            DTUseiOS6Attributes: true,
            DTDefaultFontFamily: defaultFont.familyName,
            DTDefaultFontName: defaultFont.fontName,
            DTDefaultFontSize: defaultFont.pointSize,
            DTDefaultStyleSheet: DTCSSStylesheet(styleBlock: defaultCSS) as Any,
            DTDocumentPreserveTrailingSpaces: true,
        ]

        guard let builder = DTHTMLAttributedStringBuilder(html: data, options: parsingOptions, documentAttributes: nil) else {
            throw BuildHtmlAttributedError.dataError(encoding: encoding)
        }

        builder.willFlushCallback = { element in
            guard let element else { return }
            element.sanitize()
        }
        
        guard let attributedString = builder.generatedAttributedString() else {
            throw BuildHtmlAttributedError.dataError(encoding: encoding)
        }
        
        let mutableAttributedString = NSMutableAttributedString(attributedString: attributedString)
        mutableAttributedString.applyPostParsingCustomAttributes(style: style)
        mutableAttributedString.replaceMentions(with: mentionReplacer)

        removeTrailingNewlineIfNeeded(from: mutableAttributedString, given: html)

        return mutableAttributedString
    }
    
    private static func removeTrailingNewlineIfNeeded(from mutableAttributedString: NSMutableAttributedString, given html: String) {
        // DTCoreText always adds a \n at the end of the document, which we need to remove
        // however it does not add it if </code> </a> are the last nodes.
        // It should give also issues with codeblock and blockquote when they contain newlines
        // but the usage of nbsp and zwsp solves that
        if mutableAttributedString.string.last == "\n",
           !html.hasSuffix("</code>"),
           !html.hasSuffix("</a>") {
            mutableAttributedString.deleteCharacters(
                in: NSRange(
                    location: mutableAttributedString.length - 1,
                    length: 1
                )
            )
        }
    }
}
