// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::dom::{Dom, UnicodeString};
use crate::{InlineFormatType, Location};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ComposerState<S>
where
    S: UnicodeString,
{
    pub dom: Dom<S>,
    pub start: Location,
    pub end: Location,
    pub toggled_format_types: Vec<InlineFormatType>,
}

impl<S> ComposerState<S>
where
    S: UnicodeString,
{
    pub fn new() -> Self {
        Self {
            dom: Dom::default(),
            start: Location::default(),
            end: Location::default(),
            toggled_format_types: Vec::new(),
        }
    }

    pub fn advance_selection(&mut self) {
        self.start += 1;
        self.end += 1;
    }

    /// Extends the selection by the given number of code points by moving the
    /// greater of the two selection points.
    ///
    pub(crate) fn extend_selection(&mut self, length: isize) {
        if self.start > self.end {
            self.start += length;
        } else {
            self.end += length;
        }
    }
}
