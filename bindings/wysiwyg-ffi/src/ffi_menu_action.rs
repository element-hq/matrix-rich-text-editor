// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::SuggestionPattern;

#[derive(Debug, PartialEq, Eq, uniffi::Enum)]
pub enum MenuAction {
    Keep,
    None,
    Suggestion {
        suggestion_pattern: SuggestionPattern,
    },
}

impl MenuAction {
    pub fn from(inner: wysiwyg::MenuAction) -> Self {
        match inner {
            wysiwyg::MenuAction::Keep => Self::Keep,
            wysiwyg::MenuAction::None => Self::None,
            wysiwyg::MenuAction::Suggestion(suggestion_pattern) => {
                Self::Suggestion {
                    suggestion_pattern: SuggestionPattern::from(
                        suggestion_pattern,
                    ),
                }
            }
        }
    }
}
