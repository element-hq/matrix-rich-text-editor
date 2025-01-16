// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

mod mention;

pub use crate::mention::{Mention, MentionKind, RoomIdentificationType};

pub fn is_mention(url: &str) -> bool {
    Mention::from_uri(url).is_some()
}
