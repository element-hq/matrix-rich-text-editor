//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

/// A struct that can be used as an attribute to persist the original content of a replaced part of an `NSAttributedString`.
struct MentionContent {
    /// The length of the replaced content in the Rust model.
    let rustLength: Int

    let url: String
}
