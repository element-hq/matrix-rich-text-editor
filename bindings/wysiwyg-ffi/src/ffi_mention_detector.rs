// Copyright 2025 New Vector Ltd.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use std::sync::Arc;

#[derive(Default, uniffi::Object)]
pub struct MentionDetector {}

impl MentionDetector {
    pub fn new() -> Self {
        Self {}
    }
}

#[uniffi::export]
impl MentionDetector {
    pub fn is_mention(self: &Arc<Self>, url: String) -> bool {
        matrix_mentions::is_mention(&url)
    }
}
