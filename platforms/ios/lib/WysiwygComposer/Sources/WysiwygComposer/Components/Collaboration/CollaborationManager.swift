//
// Copyright 2026 The Matrix.org Foundation C.I.C
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
import OSLog

/// Manages the collaboration lifecycle for a single composer instance.
///
/// Wraps the Automerge document collaboration APIs exposed through the
/// FFI `ComposerModel` and adds debounced delta publishing so that
/// rapid keystroke-level edits are batched into a single delta before
/// being handed to the ``CollaborationDelegate``.
///
/// ## Typical usage
///
/// ```swift
/// let manager = CollaborationManager(model: model)
/// manager.delegate = self          // receive deltas to send via Matrix
/// manager.debounceInterval = 0.5   // half-second quiet period
///
/// // After any model mutation:
/// manager.notifyLocalChange()
///
/// // When a remote Matrix event arrives:
/// try manager.receiveRemoteChanges(data)
/// ```
public final class CollaborationManager {
    // MARK: - Public properties

    /// The delegate that receives debounced deltas for sending.
    public weak var delegate: CollaborationDelegate?

    /// How long (in seconds) to wait after the last local change before
    /// flushing a delta.  Defaults to 0.5 s — long enough to batch most
    /// rapid typing while keeping latency perceptible only in real-time
    /// collaborative scenarios.
    public var debounceInterval: TimeInterval = 0.5 {
        didSet {
            // Recreate the pipeline with the new interval.
            rebuildPipeline()
        }
    }

    // MARK: - Internal / private

    /// The underlying FFI model whose collaboration methods we call.
    private let model: ComposerModel

    /// Combine subject that drives debounce.
    /// Each call to `notifyLocalChange()` pushes a value through here.
    private let changeSubject = PassthroughSubject<Void, Never>()

    /// Storage for the Combine subscription.
    private var cancellable: AnyCancellable?

    /// Serial queue used for delta calculation to avoid locking the main
    /// thread when the document is large.
    private let deltaQueue = DispatchQueue(
        label: "org.matrix.WysiwygComposer.collaboration",
        qos: .userInitiated
    )

    // MARK: - Init

    /// Create a new collaboration manager.
    ///
    /// - Parameters:
    ///   - model: The FFI `ComposerModel` instance to manage.
    ///   - debounceInterval: Initial debounce interval (default 0.5 s).
    public init(model: ComposerModel, debounceInterval: TimeInterval = 0.5) {
        self.model = model
        self.debounceInterval = debounceInterval
        rebuildPipeline()
    }

    // MARK: - Local change notification

    /// Call this after every local model mutation (replaceText, bold,
    /// enter, etc.).  The actual delta will only be computed and
    /// delivered to the delegate after the debounce interval elapses
    /// with no further calls.
    public func notifyLocalChange() {
        changeSubject.send()
    }

    /// Immediately flush any pending delta without waiting for the
    /// debounce timer.  Useful when the user sends a message or
    /// navigates away.
    public func flushNow() {
        produceDelta()
    }

    // MARK: - Receiving remote changes

    /// Apply changes received from a remote participant.
    ///
    /// - Parameter data: Raw bytes from the Matrix event payload
    ///   (produced by the remote peer's `saveIncremental()` or `saveAfter()`).
    /// - Returns: The `ComposerUpdate` produced by applying the changes,
    ///   which the caller should feed into `applyUpdate(_:)`.
    /// - Throws: `CollaborationError` if the data is malformed.
    @discardableResult
    public func receiveRemoteChanges(_ data: Data) throws -> ComposerUpdate {
        try model.receiveChanges(data: data)
    }

    /// Merge a complete remote document snapshot into the local one.
    ///
    /// - Parameter data: Raw bytes of a full document save.
    /// - Returns: The `ComposerUpdate` for re-rendering.
    /// - Throws: `CollaborationError` if the data is malformed.
    @discardableResult
    public func mergeRemoteDocument(_ data: Data) throws -> ComposerUpdate {
        try model.mergeRemote(remoteBytes: data)
    }

    // MARK: - Full document save / load

    /// Serialise the full document for persistence or initial state.
    public func saveDocument() -> Data {
        model.saveDocument()
    }

    /// Replace the current document with a full snapshot.
    ///
    /// - Throws: `CollaborationError` if the data is invalid.
    public func loadDocument(_ data: Data) throws {
        try model.loadDocument(data: data)
    }

    // MARK: - Identity

    /// The current Automerge actor ID (hex string).
    public var actorId: String {
        model.getActorId()
    }

    /// Set the actor ID to a stable hex-encoded identifier.
    ///
    /// A good choice is the Matrix device ID (hex-encoded) or
    /// `"\(userId):\(deviceId)"` encoded as hex.
    ///
    /// Must be called **before** any mutations.
    public func setActorId(_ hexId: String) throws {
        try model.setActorId(actorHex: hexId)
    }

    // MARK: - Version tracking

    /// Current document heads as hex-encoded SHA-256 hashes.
    public var heads: [String] {
        model.getHeads()
    }

    // MARK: - Private

    private func rebuildPipeline() {
        cancellable = changeSubject
            .debounce(for: .seconds(debounceInterval), scheduler: DispatchQueue.main)
            .sink { [weak self] in
                self?.produceDelta()
            }
    }

    private func produceDelta() {
        let data = model.saveIncremental()
        let currentHeads = model.getHeads()
        let delta = CollaborationDelta(
            data: data,
            heads: currentHeads
        )

        // Only notify if there are actual changes.
        guard !delta.data.isEmpty else { return }

        DispatchQueue.main.async { [weak self] in
            guard let self else { return }
            self.delegate?.collaborationManager(self, didProduceDelta: delta)
        }
    }
}
