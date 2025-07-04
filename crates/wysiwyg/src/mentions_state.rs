// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use std::collections::HashSet;

#[derive(Default, Debug, PartialEq, Eq)]
pub struct MentionsState {
    pub user_ids: HashSet<String>,
    pub room_ids: HashSet<String>,
    pub room_aliases: HashSet<String>,
    pub has_at_room_mention: bool,
}
