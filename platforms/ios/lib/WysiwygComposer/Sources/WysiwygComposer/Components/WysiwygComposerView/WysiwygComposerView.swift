//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import OSLog
import SwiftUI

/// A protocol to implement in order to inform the composer if a specific item
/// can be pasted into the application.
public protocol WysiwygItemProviderHelper {
    /// Determine if the item attached to given item provider can be pasted into the application.
    ///
    /// - Parameter itemProvider: The item provider.
    /// - Returns: True if it can be pasted, false otherwise.
    func isPasteSupported(for itemProvider: NSItemProvider) -> Bool
}

/// Handler for paste events.
public typealias PasteHandler = (NSItemProvider) -> Void

/// Main component of the Rich Text Editor, this can be added anywhere into the
/// SwiftUI hierarchy. Using the same instance of the provided view model, it's
/// possible to trigger specific actions on the composer such as formatting
/// selection, clearing content, etc
public struct WysiwygComposerView: View {
    // MARK: - Private

    private let placeholder: String
    private let placeholderColor: Color
    private let viewModel: WysiwygComposerViewModelProtocol
    private let itemProviderHelper: WysiwygItemProviderHelper?
    private let keyCommands: [WysiwygKeyCommand]?
    private let pasteHandler: PasteHandler?

    // MARK: - Public

    /// Init a `WysiwygComposerView`.
    ///
    /// - Parameters:
    ///   - placeholder: Placeholder for empty composer.
    ///   - placeholderColor: The color of the placeholder.
    ///   - viewModel: The main view model of the composer.
    ///   See `WysiwygComposerViewModel.swift` for triggerable actions.
    ///   - itemProviderHelper: A helper to determine if an item can be pasted into the hosting application.
    ///   If omitted, most non-text paste events will be ignored.
    ///   - keyCommands: The supported key commands that can be provided with their associated action
    ///   - pasteHandler: A handler for paste events.
    ///   If omitted, the composer will try to paste content as raw text.
    public init(placeholder: String,
                placeholderColor: Color = .init(UIColor.placeholderText),
                viewModel: WysiwygComposerViewModelProtocol,
                itemProviderHelper: WysiwygItemProviderHelper?,
                keyCommands: [WysiwygKeyCommand]?,
                pasteHandler: PasteHandler?) {
        self.placeholder = placeholder
        self.placeholderColor = placeholderColor
        self.viewModel = viewModel
        self.itemProviderHelper = itemProviderHelper
        self.keyCommands = keyCommands
        self.pasteHandler = pasteHandler
    }

    public var body: some View {
        UITextViewWrapper(viewModel: viewModel,
                          itemProviderHelper: itemProviderHelper,
                          keyCommands: keyCommands,
                          pasteHandler: pasteHandler)
            .accessibilityLabel(placeholder)
            .background(placeholderView, alignment: .topLeading)
    }

    // MARK: - Private

    @ViewBuilder
    private var placeholderView: some View {
        // The content can be empty but the textview not, e.g. if you start dictation
        // but have not committed the text yet.
        if viewModel.textView.attributedText.length == 0 {
            Text(placeholder)
                .font(Font(UIFont.preferredFont(forTextStyle: .body)))
                .foregroundColor(placeholderColor)
                .accessibilityHidden(true)
        }
    }
}

/// Provides a SwiftUI displayable view for the composer UITextView component.
struct UITextViewWrapper: UIViewRepresentable {
    // MARK: - Private

    /// A helper to determine if an item can be pasted into the hosting application.
    /// If omitted, most non-text paste events will be ignored.
    private let itemProviderHelper: WysiwygItemProviderHelper?
    /// A handler for key commands. If omitted, default behaviour will be applied. See `WysiwygKeyCommand.swift`.
    private let keyCommands: [WysiwygKeyCommand]?
    /// A handler for paste events. If omitted, the composer will try to paste content as raw text.
    private let pasteHandler: PasteHandler?

    private var viewModel: WysiwygComposerViewModelProtocol

    // MARK: - Internal

    init(viewModel: WysiwygComposerViewModelProtocol,
         itemProviderHelper: WysiwygItemProviderHelper?,
         keyCommands: [WysiwygKeyCommand]?,
         pasteHandler: PasteHandler?) {
        self.itemProviderHelper = itemProviderHelper
        self.keyCommands = keyCommands
        self.pasteHandler = pasteHandler
        self.viewModel = viewModel
    }
    
    func makeUIView(context: Context) -> WysiwygTextView {
        let textView = WysiwygTextView()
        // Assign the textView to the view model ASAP
        viewModel.textView = textView
        textView.accessibilityIdentifier = "WysiwygComposer"
        textView.font = UIFont.preferredFont(forTextStyle: .body)
        textView.autocapitalizationType = .sentences
        textView.isSelectable = true
        textView.isUserInteractionEnabled = true
        textView.delegate = context.coordinator
        textView.textStorage.delegate = context.coordinator
        textView.textContainerInset = .zero
        textView.textContainer.lineFragmentPadding = 0
        textView.adjustsFontForContentSizeCategory = true
        textView.backgroundColor = .clear
        textView.clipsToBounds = false
        textView.tintColor = UIColor.tintColor
        textView.wysiwygDelegate = context.coordinator
        viewModel.updateCompressedHeightIfNeeded()
        return textView
    }

    func updateUIView(_ uiView: WysiwygTextView, context: Context) {
        if uiView !== viewModel.textView {
            viewModel.textView = uiView
        }
    }

    func makeCoordinator() -> Coordinator {
        Coordinator(viewModel.replaceText,
                    viewModel.select,
                    viewModel.didUpdateText,
                    itemProviderHelper: itemProviderHelper,
                    keyCommands: keyCommands,
                    pasteHandler: pasteHandler)
    }

    @available(iOS 16.0, *)
    func sizeThatFits(_ proposal: ProposedViewSize, uiView: WysiwygTextView, context: Context) -> CGSize? {
        viewModel.getIdealSize(proposal)
    }

    /// Coordinates UIKit communication.
    class Coordinator: NSObject, UITextViewDelegate, NSTextStorageDelegate, WysiwygTextViewDelegate {
        var replaceText: (NSRange, String) -> Bool
        var select: (NSRange) -> Void
        var didUpdateText: () -> Void
        let keyCommands: [WysiwygKeyCommand]?

        private let itemProviderHelper: WysiwygItemProviderHelper?
        private let pasteHandler: PasteHandler?

        init(_ replaceText: @escaping (NSRange, String) -> Bool,
             _ select: @escaping (NSRange) -> Void,
             _ didUpdateText: @escaping () -> Void,
             itemProviderHelper: WysiwygItemProviderHelper?,
             keyCommands: [WysiwygKeyCommand]?,
             pasteHandler: PasteHandler?) {
            self.replaceText = replaceText
            self.select = select
            self.didUpdateText = didUpdateText
            self.itemProviderHelper = itemProviderHelper
            self.keyCommands = keyCommands
            self.pasteHandler = pasteHandler
        }

        func textView(_ textView: UITextView, shouldChangeTextIn range: NSRange, replacementText text: String) -> Bool {
            Logger.textView.logDebug(["Sel(att): \(range)",
                                      textView.logText,
                                      "Replacement: \"\(text)\""],
                                     functionName: #function)
            let change = replaceText(range, text)
            Logger.textView.logDebug(["change: \(change)"],
                                     functionName: #function)
            return change
        }
        
        func textViewDidChange(_ textView: UITextView) {
            Logger.textView.logDebug(
                [
                    textView.logSelection,
                    textView.logText,
                ],
                functionName: #function
            )
            didUpdateText()
            textView.toggleAutocorrectionIfNeeded()
            textView.invalidateIntrinsicContentSize()
        }

        func textViewDidChangeSelection(_ textView: UITextView) {
            Logger.textView.logDebug([textView.logSelection],
                                     functionName: #function)
            select(textView.selectedRange)
        }
        
        func textView(_ textView: UITextView,
                      shouldInteractWith URL: URL,
                      in characterRange: NSRange,
                      interaction: UITextItemInteraction) -> Bool {
            guard interaction == .invokeDefaultAction else {
                return true
            }
            textView.selectedRange = characterRange
            return false
        }

        func isPasteSupported(for itemProvider: NSItemProvider) -> Bool {
            guard let itemProviderHelper else {
                return false
            }
            
            return itemProviderHelper.isPasteSupported(for: itemProvider)
        }

        func textView(_ textView: UITextView, didReceivePasteWith provider: NSItemProvider) {
            pasteHandler?(provider)
        }
    }
}

// MARK: - Logger

private extension Logger {
    static let textView = Logger(subsystem: subsystem, category: "TextView")
}
