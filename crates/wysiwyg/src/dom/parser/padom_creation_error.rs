// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use super::PaDom;

#[derive(Debug, PartialEq)]
pub(crate) struct PaDomCreationError {
    pub(crate) dom: PaDom,
    pub(crate) parse_errors: Vec<String>,
}

impl PaDomCreationError {
    pub(crate) fn new() -> Self {
        Self {
            dom: PaDom::new(),
            parse_errors: Vec::new(),
        }
    }
}
