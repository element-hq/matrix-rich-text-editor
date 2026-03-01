//
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

import Combine
import Foundation
import HTMLParser
import OSLog
import SwiftUI
import UIKit

// swiftlint:disable file_length
/// Main view model for the composer. Forwards actions to the Rust model and publishes resulting states.
public class WysiwygComposerViewModel: WysiwygComposerViewModelProtocol, ObservableObject {
    // MARK: - Public

    /// The textView that the model currently manages, a default text view is provided, but you should always inject it from the UIWrapper
    public lazy var textView = {
        let textView = WysiwygTextView()
        textView.linkTextAttributes[.foregroundColor] = parserStyle.linkColor
        textView.mentionDisplayHelper = mentionDisplayHelper
        textView.apply(attributedContent, committed: &committedAttributedText)
        return textView
    }() {
        didSet {
            textView.linkTextAttributes[.foregroundColor] = parserStyle.linkColor
            textView.mentionDisplayHelper = mentionDisplayHelper
            textView.apply(attributedContent, committed: &committedAttributedText)
            textView.updateListMarkers(attributedContent.listMarkers)
        }
    }
        
    /// The composer minimal height.
    public let minHeight: CGFloat
    /// The mention replacer defined by the hosting application.
    public var mentionReplacer: MentionReplacer?

    // MARK: - Collaboration

    /// The collaboration manager for this composer instance.
    /// Created lazily on first access. Set its `delegate` to receive
    /// debounced deltas suitable for sending as Matrix events.
    public private(set) lazy var collaborationManager: CollaborationManager = {
        CollaborationManager(model: model.rawModel)
    }()

    /// Convenience: set the collaboration delegate directly on the
    /// view model instead of going through `collaborationManager`.
    public weak var collaborationDelegate: CollaborationDelegate? {
        get { collaborationManager.delegate }
        set { collaborationManager.delegate = newValue }
    }

    /// Published object for the composer attributed content.
    @Published public var attributedContent: WysiwygComposerAttributedContent = .init()
    /// Published value for the content of the text view in plain text mode.
    @Published public var plainTextContent = NSAttributedString()
    /// Published boolean for the composer empty content state.
    @Published public var isContentEmpty = true
    /// Published value for the composer ideal height to fit.
    /// Note: with SwiftUI & iOS > 16.0, the `UIViewRepresentable` will
    /// use `sizeThatFits` making registering to that publisher usually unnecessary.
    @Published public var idealHeight: CGFloat = .zero
    /// Published value for the composer current action states.
    @Published public var actionStates: [ComposerAction: ActionState] = [:]
    /// Published value for current detected suggestion pattern.
    @Published public var suggestionPattern: SuggestionPattern?
    /// Published value for the composer maximised state.
    @Published public var maximised = false {
        didSet {
            updateIdealHeight()
        }
    }

    /// Whether the composer should take any keyboard input.
    /// When set to `false`, `replaceText(range:replacementText:)` returns `false` as well.
    public var shouldReplaceText = true

    /// Published value for the composer plain text mode.
    @Published public var plainTextMode = false {
        didSet {
            updatePlainTextMode(plainTextMode)
        }
    }

    /// Style for the HTML parser.
    public var parserStyle: HTMLParserStyle {
        didSet {
            // In case of a color change, this will refresh the attributed text
            textView.linkTextAttributes[.foregroundColor] = parserStyle.linkColor
            let update = model.setContentFromHtml(html: content.html)
            applyUpdate(update)
            didUpdateText()
        }
    }
    
    /// The current max allowed height for the textView when maximised
    public var maxExpandedHeight: CGFloat {
        didSet {
            updateIdealHeight()
        }
    }
    
    /// The current max allowed height for the textView when minimised
    public var maxCompressedHeight: CGFloat {
        didSet {
            updateCompressedHeightIfNeeded()
        }
    }
    
    /// the current height of the textView when minimised
    public private(set) var compressedHeight: CGFloat = .zero {
        didSet {
            updateIdealHeight()
        }
    }

    /// The current composer content.
    public var content: WysiwygComposerContent {
        if plainTextMode {
            _ = model.setContentFromMarkdown(markdown: computeMarkdownContent())
        }
        return WysiwygComposerContent(markdown: model.getContentAsMessageMarkdown(),
                                      html: model.getContentAsMessageHtml())
    }
    
    /// The mention helper that will be used by the underlying textView
    public var mentionDisplayHelper: MentionDisplayHelper? {
        didSet {
            textView.mentionDisplayHelper = mentionDisplayHelper
        }
    }

    // MARK: - Private

    private let model: ComposerModelWrapper
    private var cancellables = Set<AnyCancellable>()
    private var defaultTextAttributes: [NSAttributedString.Key: Any] {
        [.font: UIFont.preferredFont(forTextStyle: .body),
         .foregroundColor: parserStyle.textColor]
    }

    private(set) var hasPendingFormats = false
    
    /// This is used to track the text commited to the editor by the user, as opposed to text
    /// that could be in the editor that is not yet committed (e.g. from inline predictive text or dictation ).
    private lazy var committedAttributedText = NSAttributedString(string: "", attributes: defaultTextAttributes)
    
    // MARK: - Public

    public init(minHeight: CGFloat = 22,
                maxCompressedHeight: CGFloat = 200,
                maxExpandedHeight: CGFloat = 300,
                parserStyle: HTMLParserStyle = .standard,
                mentionReplacer: MentionReplacer? = nil) {
        self.minHeight = minHeight
        idealHeight = minHeight
        self.maxCompressedHeight = maxCompressedHeight
        self.maxExpandedHeight = maxExpandedHeight
        self.parserStyle = parserStyle
        self.mentionReplacer = mentionReplacer

        model = ComposerModelWrapper()
        model.delegate = self
        // Publish composer empty state.
        $attributedContent.sink { [unowned self] content in
            isContentEmpty = content.text.length == 0 || content.plainText == "\n" // An empty <p> is left when deleting multi-line content.
        }
        .store(in: &cancellables)
        
        $idealHeight
            .removeDuplicates()
            .sink { [weak self] _ in
                guard let self = self else { return }
                // Improves a lot the user experience by keeping the selected range always visible when there are changes in the size.
                DispatchQueue.main.async {
                    self.textView.scrollRangeToVisible(self.textView.selectedRange)
                }
            }
            .store(in: &cancellables)
    }
}

// MARK: - Public

public extension WysiwygComposerViewModel {
    /// Apply any additional setup required.
    /// Should be called when the view appears.
    func setup() {
        clearContent()
    }

    /// Apply given action to the composer.
    ///
    /// - Parameters:
    ///   - action: Action to apply.
    func apply(_ action: ComposerAction) {
        Logger.viewModel.logDebug([attributedContent.logSelection,
                                   "Apply action: \(action)"],
                                  functionName: #function)
        let update = model.apply(action)
        if update.textUpdate() == .keep {
            hasPendingFormats = true
        } else if attributedContent.selection.length == 0, action.requiresReapplyFormattingOnEmptySelection {
            // Set pending format if current action requires it.
            hasPendingFormats = true
        }
        applyUpdate(update)
    }

    /// Sets given HTML as the current content of the composer.
    ///
    /// - Parameters:
    ///   - html: HTML content to apply
    func setHtmlContent(_ html: String) {
        let update = model.setContentFromHtml(html: html)
        applyUpdate(update)
        if plainTextMode {
            updatePlainTextMode(true)
        }
    }

    /// Sets given Markdown as the current content of the composer.
    ///
    /// - Parameters:
    ///   - markdown: Markdown content to apply
    func setMarkdownContent(_ markdown: String) {
        let update = model.setContentFromMarkdown(markdown: markdown)
        applyUpdate(update)
        if plainTextMode {
            updatePlainTextMode(true)
        }
    }

    /// Clear the content of the composer.
    func clearContent() {
        if plainTextMode {
            textView.attributedText = NSAttributedString(string: "", attributes: defaultTextAttributes)
            updateCompressedHeightIfNeeded()
        } else {
            applyUpdate(model.clear())
        }
    }

    /// Returns a textual representation of the composer model as a tree.
    func treeRepresentation() -> String {
        model.toTree()
    }

    /// Set a mention with given pattern. Usually used
    /// to mention a user (@) or a room/channel (#).
    ///
    /// - Parameters:
    ///   - url: The URL to the user/room.
    ///   - name: The display name of the user/room.
    ///   - mentionType: The type of mention.
    func setMention(url: String, name: String, mentionType: WysiwygMentionType) {
        let update: ComposerUpdate
        if let suggestionPattern, suggestionPattern.key == mentionType.patternKey {
            update = model.insertMentionAtSuggestion(url: url,
                                                     text: name,
                                                     suggestion: suggestionPattern,
                                                     attributes: mentionType.attributes)
        } else {
            update = model.insertMention(url: url,
                                         text: name,
                                         attributes: mentionType.attributes)
        }
        applyUpdate(update)
        hasPendingFormats = true
    }
    
    /// Sets the @room mention at the suggestion position
    func setAtRoomMention() {
        let update: ComposerUpdate
        if let suggestionPattern, suggestionPattern.key == .at {
            update = model.insertAtRoomMentionAtSuggestion(suggestionPattern)
        } else {
            update = model.insertAtRoomMention()
        }
        applyUpdate(update)
        hasPendingFormats = true
    }

    /// Set a command with `Slash` pattern.
    ///
    /// - Parameters:
    ///   - name: The name of the command.
    func setCommand(name: String) {
        guard let suggestionPattern, suggestionPattern.key == .slash else { return }
        let update = model.replaceTextSuggestion(newText: name, suggestion: suggestionPattern)
        applyUpdate(update)
    }

    // MARK: - Collaboration convenience

    /// Apply remote changes received from a Matrix event.
    ///
    /// - Parameter data: The raw delta bytes from the event payload.
    /// - Throws: `CollaborationError` if the data is invalid.
    func receiveRemoteChanges(_ data: Data) throws {
        let update = try collaborationManager.receiveRemoteChanges(data)
        applyUpdate(update)
    }

    /// Immediately flush any pending local delta (e.g. before sending
    /// a message or navigating away).
    func flushCollaborationDelta() {
        collaborationManager.flushNow()
    }
}

// MARK: - WysiwygComposerViewModelProtocol

public extension WysiwygComposerViewModel {
    func updateCompressedHeightIfNeeded() {
        let idealTextHeight = textView
            .sizeThatFits(CGSize(width: textView.bounds.size.width,
                                 height: CGFloat.greatestFiniteMagnitude))
            .height

        compressedHeight = min(maxCompressedHeight, max(minHeight, idealTextHeight))
    }

    func replaceText(range: NSRange, replacementText: String) -> Bool {
        guard shouldReplaceText else {
            return false
        }

        guard !plainTextMode else {
            return true
        }

        // Reset typing attributes when the text view is empty to prevent
        // stale link/code formatting from persisting after deletion.
        if textView.attributedText.length == 0 {
            textView.typingAttributes = defaultTextAttributes
        }

        // --- Structural operations must be driven through Rust directly ---
        // A text diff (reconcileNative) can't communicate the *intent* of these
        // operations to Rust, which needs them for list exit, item creation, etc.

        // Detect backspace: empty replacement deleting the character before the cursor,
        // or at cursor position with zero length (start-of-block backspace),
        // or deleting a selection.
        let uiSelection = textView.selectedRange
        let isSingleCharBackspace = replacementText.isEmpty
            && uiSelection.length == 0
            && (
                (range.length == 1 && range.upperBound == uiSelection.location)
                    || (range.length == 0 && range.location == uiSelection.location)
            )
        let isSelectionDeletion = replacementText.isEmpty && range.length > 0 && uiSelection.length > 0

        if isSingleCharBackspace {
            // Sync Rust cursor as a zero-length selection so list extraction
            // logic in backspace_single_cursor runs correctly.
            let cursorPos = uiSelection.location
            if attributedContent.selection.location != cursorPos || attributedContent.selection.length != 0 {
                let selUpdate = model.select(startUtf16Codeunit: UInt32(cursorPos),
                                             endUtf16Codeunit: UInt32(cursorPos))
                applyUpdate(selUpdate)
            }
            let update = model.backspace()
            applyUpdate(update, skipTextViewUpdate: false)
            return false
        }

        if isSelectionDeletion {
            // Sync Rust selection to match UIKit's selected range.
            if range != attributedContent.selection {
                select(range: range)
            }
            let update = model.backspace()
            applyUpdate(update, skipTextViewUpdate: false)
            return false
        }

        // Detect enter/newline.
        if replacementText.count == 1,
           replacementText[replacementText.startIndex].isNewline {
            // Sync Rust selection if needed.
            if range != attributedContent.selection {
                select(range: range)
            }
            let update = createEnterUpdate()
            applyUpdate(update, skipTextViewUpdate: false)
            return false
        }

        // --- Pending formats: Rust must handle insertion so it can apply the format ---
        // e.g. user tapped Bold/InlineCode before typing — Rust's replaceText()
        // applies the pending formatting that replaceTextIn() would miss.
        if hasPendingFormats {
            if range != attributedContent.selection {
                select(range: range)
            }
            let update = model.replaceText(newText: replacementText)
            applyUpdate(update, skipTextViewUpdate: false)
            hasPendingFormats = false
            return false
        }

        // --- Everything else: UIKit owns the mutation, reconcileNative() syncs to Rust ---
        return true
    }
    
    func select(range: NSRange) {
        guard !plainTextMode else { return }
        Logger.viewModel.logDebug(["Sel(att): \(range)", "select"],
                                  functionName: #function)
        let update = model.select(startUtf16Codeunit: UInt32(range.location),
                                  endUtf16Codeunit: UInt32(range.upperBound))
        applyUpdate(update)
    }

    func didUpdateText() {
        if plainTextMode {
            if textView.text.isEmpty != isContentEmpty {
                isContentEmpty = textView.text.isEmpty
            }
            plainTextContent = textView.attributedText
        } else {
            reconcileNative()
            applyPendingFormatsIfNeeded()
        }
        
        updateCompressedHeightIfNeeded()
    }
    
    func applyLinkOperation(_ linkOperation: WysiwygLinkOperation) {
        let update: ComposerUpdate
        switch linkOperation {
        case let .createLink(urlString, text):
            update = model.setLinkWithText(url: urlString, text: text, attributes: [])
        case let .setLink(urlString):
            update = model.setLink(url: urlString, attributes: [])
        case .removeLinks:
            update = model.removeLinks()
        }
        applyUpdate(update)
    }
    
    func getLinkAction() -> LinkAction {
        model.getLinkAction()
    }
    
    /// Get the current mentions present in the composer
    func getMentionsState() -> MentionsState {
        model.getMentionsState()
    }

    func enter() {
        applyUpdate(createEnterUpdate(), skipTextViewUpdate: false)
    }

    @available(iOS 16.0, *)
    func getIdealSize(_ proposal: ProposedViewSize) -> CGSize {
        guard let width = proposal.width else { return .zero }

        let idealHeight = textView
            .sizeThatFits(CGSize(width: width, height: CGFloat.greatestFiniteMagnitude))
            .height

        return CGSize(width: width,
                      height: maximised ? maxExpandedHeight : min(maxCompressedHeight, max(minHeight, idealHeight)))
    }
    
    func applyAttributedContent() {
        textView.apply(attributedContent, committed: &committedAttributedText)
    }
}

// MARK: - Private

private extension WysiwygComposerViewModel {
    /// Re-render list markers from the current model and push them to
    /// the text view.  Also updates `typingAttributes` so the cursor
    /// sits at the correct indented position for empty list items.
    func refreshListMarkers() {
        let projections = model.getBlockProjections()
        let (_, markers) = ProjectionRenderer(style: parserStyle, mentionReplacer: mentionReplacer)
            .render(projections: projections)
        attributedContent.listMarkers = markers
        textView.updateListMarkers(markers)
    }

    /// Apply given composer update to the composer.
    ///
    /// - Parameters:
    ///   - update: ComposerUpdate to apply.
    ///   - skipTextViewUpdate: A boolean indicating whether updating the text view should be skipped.
    func applyUpdate(_ update: ComposerUpdateProtocol, skipTextViewUpdate: Bool = false) {
        switch update.textUpdate() {
        case let .replaceAll(replacementHtml: codeUnits,
                             startUtf16Codeunit: start,
                             endUtf16Codeunit: end):
            applyReplaceAll(codeUnits: codeUnits, start: start, end: end)
            // Note: this makes replaceAll act like .keep on cases where we expect the text
            // view to be properly updated by the system.
            if skipTextViewUpdate {
                // We skip updating the text view as the system did that for us but that
                // is not reflected in committedAttributedText yet, so update it.
                committedAttributedText = attributedContent.text
            } else {
                applyAttributedContent()
                updateCompressedHeightIfNeeded()
            }
        case let .select(startUtf16Codeunit: start,
                         endUtf16Codeunit: end):
            applySelect(start: start, end: end)
        case .keep:
            break
        }

        // Always keep the text view's list markers in sync with the model
        // after ANY update (replaceAll, select, or keep). This handles:
        //  - indent/outdent on empty items (replaceAll where apply() guard skips)
        //  - exiting list mode via backspace on empty item (.keep update)
        //  - any other state change
        refreshListMarkers()

        // Notify the collaboration manager that local content changed.
        // The actual delta will be produced after the debounce interval.
        if case .replaceAll = update.textUpdate() {
            collaborationManager.notifyLocalChange()
        }

        switch update.menuState() {
        case let .update(actionStates: actionStates):
            self.actionStates = actionStates
        default:
            break
        }

        switch update.menuAction() {
        case .keep:
            break
        case .none:
            suggestionPattern = nil
        case let .suggestion(suggestionPattern: pattern):
            suggestionPattern = pattern
        }
    }

    /// Apply a replaceAll update to the composer
    ///
    /// - Parameters:
    ///   - codeUnits: Array of UTF16 code units representing the current HTML.
    ///   - start: Start location for the selection.
    ///   - end: End location for the selection.
    func applyReplaceAll(codeUnits: [UInt16], start: UInt32, end: UInt32) {
        let projections = model.getBlockProjections()
        let (attributed, listMarkers) = ProjectionRenderer(style: parserStyle, mentionReplacer: mentionReplacer)
            .render(projections: projections)
        let selection = NSRange(location: Int(start), length: Int(end - start))
        attributedContent = WysiwygComposerAttributedContent(
            text: attributed,
            selection: selection,
            plainText: model.getContentAsPlainText(),
            listMarkers: listMarkers
        )
        Logger.viewModel.logDebug(["Sel: {\(start), \(end - start)}",
                                   "Projections: \(projections.count) blocks",
                                   "replaceAll (projection)"],
                                  functionName: #function)
    }

    /// Apply a select update to the composer
    ///
    /// - Parameters:
    ///   - start: Start location for the selection.
    ///   - end: End location for the selection.
    func applySelect(start: UInt32, end: UInt32) {
        let selection = NSRange(location: Int(start), length: Int(end - start))
        if selection != attributedContent.selection {
            attributedContent.selection = selection
            hasPendingFormats = selection.length == 0 && !model.reversedActions.isEmpty
        }
        Logger.viewModel.logDebug(["Sel: {\(start), \(end - start)}", "applySelect"],
                                  functionName: #function)
    }
    
    /// Update the composer ideal height based on the maximised state.
    ///
    func updateIdealHeight() {
        if maximised {
            idealHeight = maxExpandedHeight
        } else {
            // This solves the slowdown caused by the "Publishing changes from within view updates" purple warning
            DispatchQueue.main.async {
                self.idealHeight = self.compressedHeight
            }
        }
    }

    /// Updates the view model content for given plain text mode setting.
    ///
    /// - Parameter enabled: whether plain text mode is enabled
    func updatePlainTextMode(_ enabled: Bool) {
        if enabled {
            var attributed = NSAttributedString(string: model.getContentAsMarkdown(),
                                                attributes: defaultTextAttributes)
            if let mentionReplacer {
                attributed = mentionReplacer.postProcessMarkdown(in: attributed)
            }
            textView.attributedText = attributed
            updateCompressedHeightIfNeeded()
        } else {
            let update = model.setContentFromMarkdown(markdown: computeMarkdownContent())
            applyUpdate(update)
            didUpdateText()
            plainTextContent = NSAttributedString()
        }
    }
    
    /// Reconcile the text view content back into the Rust model.
    ///
    /// Compares `committedAttributedText.string` (last known state from Rust)
    /// with `textView.attributedText.string` (current UIKit state) using a
    /// prefix/suffix diff, then feeds the minimal replacement into
    /// `model.replaceTextIn()`.
    ///
    /// Because ProjectionRenderer guarantees 1:1 UTF-16 offset mapping,
    /// the diff offsets can be sent directly to Rust without any HTML-range
    /// translation.
    func reconcileNative() {
        guard !textView.isDictationRunning else { return }
        let oldText = committedAttributedText.string
        let newText = textView.attributedText.string
        guard oldText != newText else { return }

        let diff = computePrefixSuffixDiff(old: oldText, new: newText)
        let update = model.replaceTextIn(
            newText: diff.replacement,
            start: UInt32(diff.replaceStart),
            end: UInt32(diff.replaceEnd)
        )
        applyUpdate(update, skipTextViewUpdate: true)

        // Resync cursor — 1:1 UTF-16 offsets thanks to ProjectionRenderer.
        let sel = textView.selectedRange
        let selUpdate = model.select(
            startUtf16Codeunit: UInt32(sel.location),
            endUtf16Codeunit: UInt32(sel.upperBound)
        )
        applyUpdate(selUpdate)
    }

    /// Updates the text view with the current content if we have some pending formats
    /// to apply (e.g. we hit the bold button with no selection).
    func applyPendingFormatsIfNeeded() {
        guard hasPendingFormats else { return }
        applyAttributedContent()
        updateCompressedHeightIfNeeded()
        hasPendingFormats = false
    }

    /// Compute the current content of the `UITextView`, as markdown.
    ///
    /// - Returns: A markdown string.
    func computeMarkdownContent() -> String {
        let markdownContent: String
        if let mentionReplacer,
           let attributedText = textView.attributedText {
            // `MentionReplacer` should restore altered content to valid markdown.
            markdownContent = mentionReplacer.restoreMarkdown(in: attributedText)
        } else {
            markdownContent = textView.text
        }

        return markdownContent
    }

    func createEnterUpdate() -> ComposerUpdate {
        let update = model.enter()
        // Pending formats need to be reapplied to the
        // NSAttributedString upon next character input if we
        // are in a structure that adds special formatting
        // (e.g. code blocks, quotes, list items).
        if !model
            .reversedActions
            .isDisjoint(with: [.codeBlock, .quote, .orderedList, .unorderedList]) {
            hasPendingFormats = true
        }
        return update
    }
}

// MARK: - ComposerModelWrapperDelegate

extension WysiwygComposerViewModel: ComposerModelWrapperDelegate {
    func fallbackContent() -> String {
        attributedContent.plainText
    }
}

// MARK: - Logger

private extension Logger {
    static let viewModel = Logger(subsystem: subsystem, category: "ViewModel")
}
