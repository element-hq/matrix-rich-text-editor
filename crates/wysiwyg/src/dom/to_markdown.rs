// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use super::UnicodeString;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum MarkdownError<S>
where
    S: UnicodeString,
{
    InvalidListItem(Option<S>),
}

impl<S> Error for MarkdownError<S> where S: UnicodeString {}

impl<S> fmt::Display for MarkdownError<S>
where
    S: UnicodeString,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidListItem(Some(node_name)) => write!(formatter, "A list expects a list item as immediate child, received `{node_name}`"),

            Self::InvalidListItem(None) => write!(formatter, "A list node expects a list item node as immediate child")
        }
    }
}

pub trait ToMarkdown<S>
where
    S: UnicodeString,
{
    fn fmt_markdown(
        &self,
        buffer: &mut S,
        options: &MarkdownOptions,
        as_message: bool,
    ) -> Result<(), MarkdownError<S>>;

    fn to_message_markdown(&self) -> Result<S, MarkdownError<S>> {
        let mut buffer = S::default();
        self.fmt_markdown(&mut buffer, &MarkdownOptions::empty(), true)?;

        Ok(buffer)
    }
    fn to_markdown(&self) -> Result<S, MarkdownError<S>> {
        let mut buffer = S::default();
        self.fmt_markdown(&mut buffer, &MarkdownOptions::empty(), false)?;

        Ok(buffer)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MarkdownOptions {
    bits: u8,
}

impl MarkdownOptions {
    pub const IGNORE_LINE_BREAK: Self = Self { bits: 0b0001 };

    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    /// Returns `true` if all of the flags in `other` are contained within `self`.
    pub const fn contains(&self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }

    /// Inserts the specified flags in-place.
    pub fn insert(&mut self, other: Self) {
        self.bits |= other.bits;
    }
}
