// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::SuggestionPattern;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuAction {
    Keep,
    None,
    Suggestion(SuggestionPattern),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuActionSuggestion {
    pub suggestion_pattern: SuggestionPattern,
}
