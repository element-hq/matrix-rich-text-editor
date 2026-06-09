//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Testing
@testable import WysiwygComposer

private enum Constants {
    static let fallbackContent = "Fallback content"
}

@MainActor
struct WysiwygComposerTests {
    @Test func composerEmptyState() {
        ComposerModelWrapper()
            .assertHtml("")
            .execute { #expect($0.getContentAsMarkdown() == "") }
            .assertSelection(start: 0, end: 0)
    }

    @Test func composerCrashRecovery() {
        class SomeDelegate: ComposerModelWrapperDelegate {
            func fallbackContent() -> String {
                Constants.fallbackContent
            }
        }

        let delegate = SomeDelegate()
        let model = ComposerModelWrapper()
        model.delegate = delegate

        model
            .action { $0.replaceText(newText: "Some text") }
            // Force a crash
            .action { $0.setContentFromHtml(html: "<//strong>") }
            .assertHtml(Constants.fallbackContent)
    }
}
