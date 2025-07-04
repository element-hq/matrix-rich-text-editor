// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::UnicodeString;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkActionUpdate<S: UnicodeString> {
    Keep,
    Update(LinkAction<S>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkAction<S: UnicodeString> {
    CreateWithText,
    Create,
    Edit(S),
    Disabled,
}
