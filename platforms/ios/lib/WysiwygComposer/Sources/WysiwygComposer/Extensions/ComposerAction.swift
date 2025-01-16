//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

extension ComposerAction {
    /// Returns `true` if action requires all current formatting to be re-applied on
    /// next character stroke when triggered on an empty selection.
    var requiresReapplyFormattingOnEmptySelection: Bool {
        switch self {
        case .bold, .italic, .strikeThrough, .underline, .inlineCode, .link, .undo, .redo:
            return false
        case .orderedList, .unorderedList, .indent, .unindent, .codeBlock, .quote:
            return true
        }
    }
}
