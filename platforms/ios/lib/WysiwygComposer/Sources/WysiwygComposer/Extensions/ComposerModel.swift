//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

extension ComposerModel {
    // swiftlint:disable cyclomatic_complexity
    /// Apply given action to the composer model.
    ///
    /// - Parameters:
    ///   - action: Action to apply.
    func apply(_ action: ComposerAction) throws -> ComposerUpdate {
        let update: ComposerUpdate
        switch action {
        case .bold:
            update = try bold()
        case .italic:
            update = try italic()
        case .strikeThrough:
            update = try strikeThrough()
        case .underline:
            update = try underline()
        case .inlineCode:
            update = try inlineCode()
        case .undo:
            update = try undo()
        case .redo:
            update = try redo()
        case .orderedList:
            update = try orderedList()
        case .unorderedList:
            update = try unorderedList()
        case .indent:
            update = try indent()
        case .unindent:
            update = try unindent()
        case .codeBlock:
            update = try codeBlock()
        case .quote:
            update = try quote()
        case .link:
            fatalError()
        }

        return update
    }

    // swiftlint:enable cyclomatic_complexity

    /// Returns currently reversed (active) actions on the composer model.
    var reversedActions: Set<ComposerAction> {
        Set(actionStates().compactMap { (key: ComposerAction, value: ActionState) in
            value == .reversed ? key : nil
        })
    }
}
