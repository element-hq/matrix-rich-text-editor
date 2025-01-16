// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use html5ever::QualName;

use super::PaDomHandle;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PaNodeContainer {
    pub(crate) name: QualName,
    pub(crate) attrs: Vec<(String, String)>,
    pub(crate) children: Vec<PaDomHandle>,
}
impl PaNodeContainer {
    pub(crate) fn get_attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(n, _v)| n == name)
            .map(|(_n, v)| v.as_str())
    }
}
