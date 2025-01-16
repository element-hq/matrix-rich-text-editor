// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::PatternKey;

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct SuggestionPattern {
    pub key: PatternKey,
    pub text: String,
    pub start: u32,
    pub end: u32,
}

impl From<wysiwyg::SuggestionPattern> for SuggestionPattern {
    fn from(inner: wysiwyg::SuggestionPattern) -> Self {
        Self {
            key: PatternKey::from(inner.key),
            text: inner.text,
            start: u32::try_from(inner.start).unwrap(),
            end: u32::try_from(inner.end).unwrap(),
        }
    }
}

impl From<SuggestionPattern> for wysiwyg::SuggestionPattern {
    fn from(pattern: SuggestionPattern) -> Self {
        Self {
            key: wysiwyg::PatternKey::from(pattern.key),
            text: pattern.text,
            start: usize::try_from(pattern.start).unwrap(),
            end: usize::try_from(pattern.end).unwrap(),
        }
    }
}
