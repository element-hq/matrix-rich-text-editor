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

import Foundation

/// Metadata included with each delta so recipients can compute minimal
/// diffs and detect missing changes.
public struct CollaborationDelta {
    /// The binary Automerge delta (output of `saveIncremental()` or `saveAfter()`).
    /// Send this as the event payload. An empty `Data` means no changes since
    /// the last flush.
    public let data: Data

    /// The document heads after this delta was produced.
    /// Hex-encoded SHA-256 hashes — include these in the Matrix event so
    /// recipients can request only the changes they are missing.
    public let heads: [String]
}

/// Protocol that the host application implements to send collaboration
/// deltas over Matrix (or any other transport).
///
/// The composer calls `collaborationManager(_:didProduceDelta:)` after
/// a debounce period with no further edits, giving the host a chance
/// to send the accumulated changes as a Matrix event.
public protocol CollaborationDelegate: AnyObject {
    /// Called on the main queue when a debounced batch of local edits
    /// is ready to be sent to remote participants.
    ///
    /// - Parameters:
    ///   - manager: The collaboration manager that produced the delta.
    ///   - delta: The incremental changes and associated metadata.
    func collaborationManager(
        _ manager: CollaborationManager,
        didProduceDelta delta: CollaborationDelta
    )
}
