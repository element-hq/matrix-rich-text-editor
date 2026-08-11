// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::UnicodeString;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TableCellType {
    Data,
    Header,
}

impl TableCellType {
    pub(crate) fn tag(&self) -> &'static str {
        match self {
            TableCellType::Data => "td",
            TableCellType::Header => "th",
        }
    }
}

impl<S: UnicodeString> From<S> for TableCellType {
    fn from(value: S) -> Self {
        match value.to_string().as_str() {
            "td" => TableCellType::Data,
            "th" => TableCellType::Header,
            _ => {
                panic!(
                    "Unknown table cell type {}",
                    value.to_string().as_str()
                );
            }
        }
    }
}
