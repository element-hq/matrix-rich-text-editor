//
// Copyright 2025 Element Creations Ltd.
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Combine
import Testing
import UIKit
@testable import WysiwygComposer

@MainActor
struct WysiwygComposerViewModelTests {
    let viewModel = WysiwygComposerViewModel()

    init() {
        viewModel.clearContent()
    }

    @Test func isContentEmpty() async {
        #expect(viewModel.isContentEmpty)

        let emptyPublisher = viewModel.$isContentEmpty.removeDuplicates().dropFirst()
        let becameNonEmpty = await nextValue(of: emptyPublisher) {
            simulateTyping("Test", in: .zero)
        }
        #expect(becameNonEmpty == false)

        let becameEmpty = await nextValue(of: emptyPublisher) {
            simulateTyping("", in: NSRange(location: 0, length: viewModel.attributedContent.text.length))
        }
        #expect(becameEmpty == true)
    }

    @Test func isContentEmptyAfterDeletingSingleSpace() {
        // When typing a single space.
        _ = viewModel.replaceText(range: .zero, replacementText: " ")
        viewModel.textView.attributedText = NSAttributedString(string: " ")
        viewModel.didUpdateText()

        // And then deleting that space.
        _ = viewModel.replaceText(range: .init(location: 0, length: 1), replacementText: "")
        viewModel.textView.attributedText = NSAttributedString(string: "")
        viewModel.didUpdateText()

        // Then the content should be empty for the placeholder to be shown.
        #expect(viewModel.isContentEmpty)
    }

    @Test func isContentEmptyAfterDeletingMultilineContent() {
        // When typing a new line.
        _ = viewModel.replaceText(range: .zero, replacementText: "\n")
        viewModel.textView.attributedText = NSAttributedString(string: "\n")
        viewModel.didUpdateText()

        // And then deleting that new line.
        _ = viewModel.replaceText(range: .init(location: 0, length: 1), replacementText: "")
        viewModel.textView.attributedText = NSAttributedString(string: "")
        viewModel.didUpdateText()

        // Then the content should be empty for the placeholder to be shown.
        #expect(viewModel.isContentEmpty)
    }

    @Test func simpleTextInputIsAccepted() {
        let shouldChange = viewModel.replaceText(range: .zero,
                                                 replacementText: "A")
        #expect(shouldChange)
    }

    @Test func simpleTextInputIsNotAccepted() {
        viewModel.shouldReplaceText = false
        let shouldChange = viewModel.replaceText(range: .zero,
                                                 replacementText: "A")
        #expect(!shouldChange)
    }

    @Test func newlineIsNotAccepted() {
        // Enter is driven through Rust directly (model.enter()),
        // so replaceText returns false to prevent UIKit from also applying it.
        let shouldChange = viewModel.replaceText(range: .zero,
                                                 replacementText: "\n")
        #expect(!shouldChange)
    }

    @Test func reconciliateModel() {
        simulateTyping("wa", in: .zero)
        #expect(viewModel.attributedContent.text.string == "wa")
        #expect(viewModel.attributedContent.selection == NSRange(location: 2, length: 0))
        reconciliate(to: "わ", selectedRange: NSRange(location: 1, length: 0))
        #expect(viewModel.attributedContent.text.string == "わ")
        #expect(viewModel.attributedContent.selection == NSRange(location: 1, length: 0))
    }

    @Test func reconciliateRestoresSelection() {
        simulateTyping("I\'m", in: .zero)
        #expect(viewModel.attributedContent.selection == NSRange(location: 3, length: 0))
        reconciliate(to: "I’m", selectedRange: NSRange(location: 3, length: 0))
        #expect(viewModel.attributedContent.selection == NSRange(location: 3, length: 0))

        viewModel.clearContent()

        simulateTyping("Some text", in: .zero)
        viewModel.select(range: .zero)
        #expect(viewModel.attributedContent.selection == .zero)
        reconciliate(to: "Some test", selectedRange: .zero)
        #expect(viewModel.attributedContent.selection == .zero)
    }

    @Test func plainTextMode() {
        simulateTyping("Some bold text", in: .zero)
        viewModel.select(range: .init(location: 10, length: 4))
        viewModel.apply(.bold)

        #expect(viewModel.content.html == "Some bold <strong>text</strong>")

        viewModel.plainTextMode = true
        #expect(viewModel.content.markdown == "Some bold __text__")
        #expect(viewModel.content.html == "Some bold <strong>text</strong>")

        viewModel.plainTextMode = false
        #expect(viewModel.content.html == "Some bold <strong>text</strong>")
    }

    @Test func replaceTextAfterLink() {
        viewModel.applyLinkOperation(.createLink(urlString: "https://element.io", text: "test"))
        // UIKit owns the edit, reconciliation places text outside the link.
        simulateTyping("abc", in: .init(location: 4, length: 0))
        #expect(viewModel.content.html == "<a href=\"https://element.io\">test</a>abc")
    }

    @Test func replaceTextPartiallyInsideAndAfterLink() {
        viewModel.applyLinkOperation(.createLink(urlString: "https://element.io", text: "test"))
        simulateTyping("abc", in: .init(location: 3, length: 1))
        #expect(viewModel.content.html == "<a href=\"https://element.io\">tes</a>abc")
    }

    @Test func replaceTextInsideLink() {
        viewModel.applyLinkOperation(.createLink(urlString: "https://element.io", text: "test"))
        simulateTyping("abc", in: .init(location: 2, length: 0))
        #expect(viewModel.content.html == "<a href=\"https://element.io\">teabcst</a>")
    }

    @Test func crashRecoveryUsesLatestPlainText() {
        viewModel.setHtmlContent("<strong>Some <em>text</em></strong>")
        // Force a crash
        viewModel.setHtmlContent("<//strong>")
        #expect(viewModel.content.html == "Some text")
    }

    @Test func pendingFormatIsReapplied() {
        viewModel.apply(.orderedList)
        viewModel.apply(.bold)
        viewModel.apply(.italic)
        mockTrailingTyping("Formatted")
        // Enter
        mockTrailingTyping("\n")
        mockTrailingTyping("Still formatted")
        #expect(
            viewModel
                .textView
                .attributedText
                .fontSymbolicTraits(at: viewModel.textView.attributedText.length - 1)
                .contains([.traitBold, .traitItalic])
        )
    }

    @Test func pendingFormatFlagInNewList() {
        viewModel.apply(.bold)
        viewModel.apply(.italic)
        mockTrailingTyping("Text")
        viewModel.enter()
        // After creating a list, pending format flag is on
        viewModel.apply(.orderedList)
        #expect(viewModel.hasPendingFormats)
        // Typing consumes the flag
        mockTrailingTyping("Item")
        #expect(!viewModel.hasPendingFormats)
        // Creating a second list item re-enables the flag
        viewModel.enter()
        #expect(viewModel.hasPendingFormats)
    }

    @Test func pendingFormatFlagAfterReselectingListItem() {
        viewModel.apply(.bold)
        viewModel.apply(.italic)
        mockTrailingTyping("Text1")
        viewModel.enter()
        viewModel.apply(.orderedList)
        let inListSelection = viewModel.attributedContent.selection
        let insertedText = "Text2"
        mockTyping(insertedText, at: 0)
        // After re-selecting the empty list item, pending format flag is still on
        viewModel.select(range: NSRange(location: inListSelection.location + insertedText.utf16Length,
                                        length: inListSelection.length))
        #expect(viewModel.hasPendingFormats)
    }
}

// MARK: - Async helpers

extension WysiwygComposerViewModelTests {
    /// Awaits the next value published by `publisher` while `action` runs, then returns it.
    ///
    /// The view model publishes on the main actor, so the value typically arrives synchronously
    /// during `action`. Replaces the previous `XCTestExpectation` + Combine-sink helpers.
    func nextValue<P: Publisher>(of publisher: P, while action: () -> Void) async -> P.Output
        where P.Failure == Never {
        var result: P.Output?
        var cancellable: AnyCancellable?
        // Resume with `Void` and stash the value in `result`: the value (e.g. attributed content)
        // may be non-Sendable, so it must stay on the main actor rather than cross the continuation.
        await withCheckedContinuation { (continuation: CheckedContinuation<Void, Never>) in
            var resumed = false
            cancellable = publisher.sink { value in
                guard !resumed else { return }
                resumed = true
                result = value
                continuation.resume()
            }
            action()
        }
        cancellable?.cancel()
        return result!
    }
}

// MARK: - Helpers

extension WysiwygComposerViewModelTests {
    /// Simulate the full UIKit typing cycle: replaceText → text view update → didUpdateText.
    /// If replaceText returns false (e.g. backspace, enter — handled by Rust directly),
    /// skips the UIKit text mutation since Rust already updated the text view.
    ///
    /// - Parameters:
    ///   - text: Replacement text (use empty string for deletions).
    ///   - range: The range being replaced in the text view.
    func simulateTyping(_ text: String, in range: NSRange) {
        let shouldAcceptChange = viewModel.replaceText(range: range, replacementText: text)
        if shouldAcceptChange {
            let mutable = NSMutableAttributedString(attributedString: viewModel.textView.attributedText)
            mutable.replaceCharacters(in: range, with: text)
            viewModel.textView.attributedText = mutable
            viewModel.textView.selectedRange = NSRange(
                location: range.location + (text as NSString).length,
                length: 0
            )
            viewModel.didUpdateText()
        }
    }

    /// Mock typing at given location.
    ///
    /// - Parameters:
    ///   - text: text to type
    ///   - location: index in text view's attributed string
    func mockTyping(_ text: String, at location: Int) {
        guard location <= viewModel.textView.attributedText.length else {
            fatalError("Invalid location index")
        }
        simulateTyping(text, in: NSRange(location: location, length: 0))
    }

    /// Mock typing trailing text.
    ///
    /// - Parameter text: text to type
    func mockTrailingTyping(_ text: String) {
        mockTyping(text, at: viewModel.textView.attributedText.length)
    }

    /// Mock backspacing at given location.
    ///
    /// - Parameter location: index in text view's attributed string
    func mockBackspace(at location: Int) {
        guard location <= viewModel.textView.attributedText.length else {
            fatalError("Invalid location index")
        }

        let range: NSRange = location == 0 ? .zero : NSRange(location: location - 1, length: 1)
        let shouldAcceptChange = viewModel.replaceText(range: range, replacementText: "")
        if shouldAcceptChange {
            let mutable = NSMutableAttributedString(attributedString: viewModel.textView.attributedText)
            if range.length > 0 {
                mutable.replaceCharacters(in: range, with: "")
            }
            viewModel.textView.attributedText = mutable
            viewModel.textView.selectedRange = NSRange(location: range.location, length: 0)
            viewModel.didUpdateText()
        }
    }

    /// Mock backspacing from trailing position.
    func mockTrailingBackspace() {
        mockBackspace(at: viewModel.textView.attributedText.length)
    }
}

private extension WysiwygComposerViewModelTests {
    /// Fakes a trigger of the reconciliate mechanism of the view model.
    ///
    /// - Parameters:
    ///   - newText: New text to apply.
    ///   - selectedRange: Simulated selection in the text view.
    func reconciliate(to newText: String, selectedRange: NSRange) {
        viewModel.textView.attributedText = NSAttributedString(string: newText)
        // Set selection where we want it, as setting the content automatically moves cursor to the end.
        viewModel.textView.selectedRange = selectedRange
        viewModel.didUpdateText()
    }
}
