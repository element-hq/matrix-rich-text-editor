// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use std::{error::Error, fmt::Display};

#[derive(Debug, uniffi::Error)]
pub enum DomCreationError {
    HtmlParseError,
    MarkdownParseError,
}

impl Display for DomCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            DomCreationError::HtmlParseError => {
                "could not create dom from html"
            }
            DomCreationError::MarkdownParseError => {
                "could not create dom from markdown"
            }
        })
    }
}

impl From<wysiwyg::DomCreationError> for DomCreationError {
    fn from(error: wysiwyg::DomCreationError) -> Self {
        match error {
            wysiwyg::DomCreationError::HtmlParseError(_) => {
                Self::HtmlParseError
            }
            wysiwyg::DomCreationError::MarkdownParseError(_) => {
                Self::MarkdownParseError
            }
        }
    }
}

impl From<DomCreationError> for wysiwyg::DomCreationError {
    fn from(_: DomCreationError) -> Self {
        unimplemented!("Error is not needed as input")
    }
}

impl Error for DomCreationError {}
