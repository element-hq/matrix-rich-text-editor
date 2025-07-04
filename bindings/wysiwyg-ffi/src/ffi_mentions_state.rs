// Copyright 2025 New Vector Ltd.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

#[derive(uniffi::Record)]
pub struct MentionsState {
    pub user_ids: Vec<String>,
    pub room_ids: Vec<String>,
    pub room_aliases: Vec<String>,
    pub has_at_room_mention: bool,
}

impl From<wysiwyg::MentionsState> for MentionsState {
    fn from(value: wysiwyg::MentionsState) -> Self {
        Self {
            user_ids: value.user_ids.into_iter().collect(),
            room_ids: value.room_ids.into_iter().collect(),
            room_aliases: value.room_aliases.into_iter().collect(),
            has_at_room_mention: value.has_at_room_mention,
        }
    }
}
