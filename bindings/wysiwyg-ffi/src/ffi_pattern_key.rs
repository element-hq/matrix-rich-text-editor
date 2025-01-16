// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

#[derive(Debug, PartialEq, Eq, uniffi::Enum)]
pub enum PatternKey {
    At,
    Hash,
    Slash,
    Custom(String),
}

impl From<wysiwyg::PatternKey> for PatternKey {
    fn from(inner: wysiwyg::PatternKey) -> Self {
        match inner {
            wysiwyg::PatternKey::At => Self::At,
            wysiwyg::PatternKey::Hash => Self::Hash,
            wysiwyg::PatternKey::Slash => Self::Slash,
            wysiwyg::PatternKey::Custom(key) => Self::Custom(key),
        }
    }
}

impl From<PatternKey> for wysiwyg::PatternKey {
    fn from(key: PatternKey) -> Self {
        match key {
            PatternKey::At => Self::At,
            PatternKey::Hash => Self::Hash,
            PatternKey::Slash => Self::Slash,
            PatternKey::Custom(key) => Self::Custom(key),
        }
    }
}
