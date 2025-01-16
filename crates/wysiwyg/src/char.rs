// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

pub trait CharExt: Sized {
    fn nbsp() -> Self;
}

impl CharExt for char {
    fn nbsp() -> Self {
        '\u{A0}'
    }
}
