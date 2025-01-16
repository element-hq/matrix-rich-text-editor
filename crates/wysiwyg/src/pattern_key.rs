// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatternKey {
    At,
    Hash,
    Slash,
    Custom(String),
}

impl PatternKey {
    pub(crate) fn is_static_pattern(&self) -> bool {
        matches!(self, Self::At | Self::Hash | Self::Slash)
    }

    pub(crate) fn from_string_and_suggestions(
        string: String,
        custom_suggestion_patterns: &HashSet<String>,
    ) -> Option<Self> {
        if custom_suggestion_patterns.contains(&string) {
            return Some(Self::Custom(string));
        }
        let first_char = string.chars().nth(0)?;
        match first_char {
            '\u{0040}' => Some(Self::At),
            '\u{0023}' => Some(Self::Hash),
            '\u{002F}' => Some(Self::Slash),
            _ => None,
        }
    }
}
