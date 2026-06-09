//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Combine
import Testing
@testable import WysiwygComposer

private enum Constants {
    static let sampleHtml = "some <strong>bold</strong> text"
    static let sampleMarkdown = "some __bold__ text"
    static let samplePlainText = "some bold text"
    static let sampleHtml2 = "<ol><li><strong>A</strong></li><li><em>B</em></li></ol>"
    static let sampleMarkdown2 = "1. __A__\n2. *B*"
}

extension WysiwygComposerViewModelTests {
    @Test func setHtmlContent() {
        viewModel.setHtmlContent(Constants.sampleHtml)
        #expect(viewModel.content.html == Constants.sampleHtml)
        #expect(viewModel.content.markdown == Constants.sampleMarkdown)

        viewModel.setHtmlContent(Constants.sampleHtml2)
        #expect(viewModel.content.html == Constants.sampleHtml2)
        #expect(viewModel.content.markdown == Constants.sampleMarkdown2)
    }

    @Test func setMarkdownContent() {
        viewModel.setMarkdownContent(Constants.sampleMarkdown)
        #expect(viewModel.content.html == Constants.sampleHtml)
        #expect(viewModel.content.markdown == Constants.sampleMarkdown)

        viewModel.setMarkdownContent(Constants.sampleMarkdown2)
        #expect(viewModel.content.html == Constants.sampleHtml2)
        #expect(viewModel.content.markdown == Constants.sampleMarkdown2)
    }

    @Test func setHtmlContentTriggersPublish() async {
        // The plain text is asserted, as it's way easier to build than the attributed string.
        let publisher = viewModel.$attributedContent.removeDuplicates { $0.text == $1.text }.dropFirst()
        let content = await nextValue(of: publisher) {
            viewModel.setHtmlContent(Constants.sampleHtml)
        }
        #expect(content.plainText == Constants.samplePlainText)
    }

    @Test func setMarkdownContentTriggersPublish() async {
        let publisher = viewModel.$attributedContent.removeDuplicates { $0.text == $1.text }.dropFirst()
        let content = await nextValue(of: publisher) {
            viewModel.setMarkdownContent(Constants.sampleMarkdown)
        }
        #expect(content.plainText == Constants.samplePlainText)
    }
}
