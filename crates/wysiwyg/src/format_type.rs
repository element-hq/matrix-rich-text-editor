// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::{ComposerAction, UnicodeString};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum InlineFormatType {
    Bold,
    Italic,
    StrikeThrough,
    Underline,
    InlineCode,
}

impl InlineFormatType {
    pub fn tag(&self) -> &'static str {
        match self {
            InlineFormatType::Bold => "strong",
            InlineFormatType::Italic => "em",
            InlineFormatType::StrikeThrough => "del",
            InlineFormatType::Underline => "u",
            InlineFormatType::InlineCode => "code",
        }
    }

    pub fn action(&self) -> ComposerAction {
        match self {
            InlineFormatType::Bold => ComposerAction::Bold,
            InlineFormatType::Italic => ComposerAction::Italic,
            InlineFormatType::StrikeThrough => ComposerAction::StrikeThrough,
            InlineFormatType::Underline => ComposerAction::Underline,
            InlineFormatType::InlineCode => ComposerAction::InlineCode,
        }
    }
}

impl<S: UnicodeString> From<S> for InlineFormatType {
    fn from(value: S) -> Self {
        match value.to_string().as_str() {
            "b" | "strong" => InlineFormatType::Bold,
            "i" | "em" => InlineFormatType::Italic,
            "del" => InlineFormatType::StrikeThrough,
            "u" => InlineFormatType::Underline,
            "code" => InlineFormatType::InlineCode,
            _ => {
                panic!("Unknown format type {}", value.to_string().as_str());
            }
        }
    }
}
