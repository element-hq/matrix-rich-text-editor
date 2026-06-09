//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Testing
@testable import WysiwygComposer

extension WysiwygComposerTests {
    @Test func createWithTextLinkAction() {
        ComposerModelWrapper()
            .assertLinkAction(.createWithText)
    }

    @Test func createLinkAction() {
        ComposerModelWrapper()
            .action { $0.replaceText(newText: "test") }
            .action { $0.select(startUtf16Codeunit: 0, endUtf16Codeunit: 4) }
            .assertLinkAction(.create)
    }

    @Test func editLinkAction() {
        let url = "test_url"
        ComposerModelWrapper()
            .action { $0.setLinkWithText(url: url, text: "test", attributes: []) }
            .assertLinkAction(.edit(url: "https://\(url)"))
    }

    @Test func setLinkWithText() {
        ComposerModelWrapper()
            .action { $0.setLinkWithText(url: "link", text: "text", attributes: []) }
            .assertTree(
                """
                
                └>a \"https://link\"
                  └>\"text\"
                
                """
            )
    }
    
    @Test func setLinkWithTextWithIncludedScheme() {
        ComposerModelWrapper()
            .action { $0.setLinkWithText(url: "http://link", text: "text", attributes: []) }
            .assertTree(
                """
                
                └>a \"http://link\"
                  └>\"text\"
                
                """
            )
    }
    
    @Test func setMailLinkWithText() {
        ComposerModelWrapper()
            .action { $0.setLinkWithText(url: "test@element.io", text: "text", attributes: []) }
            .assertTree(
                """
                
                └>a \"mailto:test@element.io\"
                  └>\"text\"
                
                """
            )
    }

    @Test func setLink() {
        ComposerModelWrapper()
            .action { $0.replaceText(newText: "text") }
            .action { $0.select(startUtf16Codeunit: 0, endUtf16Codeunit: 4) }
            .action { $0.setLink(url: "link", attributes: []) }
            .assertTree(
                """
                
                └>a \"https://link\"
                  └>\"text\"
                
                """
            )
    }

    @Test func removeLinks() {
        ComposerModelWrapper()
            .action { $0.setLinkWithText(url: "link", text: "text", attributes: []) }
            .assertTree(
                """
                
                └>a \"https://link\"
                  └>\"text\"
                
                """
            )
            .action { $0.removeLinks() }
            .assertTree(
                """
                
                └>"text"
                
                """
            )
    }
}
