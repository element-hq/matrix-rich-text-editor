// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::PatternKey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuggestionPattern {
    pub key: PatternKey,
    pub text: String,
    pub start: usize,
    pub end: usize,
}
