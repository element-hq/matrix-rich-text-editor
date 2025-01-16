// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::{dom::UnicodeString, Location};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextUpdate<S>
where
    S: UnicodeString,
{
    Keep,
    ReplaceAll(ReplaceAll<S>),
    Select(Selection),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceAll<S>
where
    S: UnicodeString,
{
    pub replacement_html: S,
    pub start: Location,
    pub end: Location,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub start: Location,
    pub end: Location,
}
