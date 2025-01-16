// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use core::fmt;

#[derive(Debug, Eq, PartialEq)]
pub enum DomCreationError {
    HtmlParseError(HtmlParseError),
    MarkdownParseError(MarkdownParseError),
}

#[derive(Debug, Eq, PartialEq)]
pub struct HtmlParseError {
    pub parse_errors: Vec<String>,
}

impl HtmlParseError {
    pub fn new(parse_errors: Vec<String>) -> Self {
        Self { parse_errors }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MarkdownParseError {
    InvalidMarkdownError,
}

impl fmt::Display for MarkdownParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            Self::InvalidMarkdownError => "unable to parse markdown",
        };
        write!(f, "{message}")
    }
}
