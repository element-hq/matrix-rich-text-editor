// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::UnicodeString;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ListType {
    Ordered,
    Unordered,
}

impl ListType {
    pub(crate) fn tag(&self) -> &'static str {
        match self {
            ListType::Ordered => "ol",
            ListType::Unordered => "ul",
        }
    }
}

impl<S: UnicodeString> From<S> for ListType {
    fn from(value: S) -> Self {
        match value.to_string().as_str() {
            "ol" => ListType::Ordered,
            "ul" => ListType::Unordered,
            _ => {
                panic!("Unknown list type {}", value.to_string().as_str());
            }
        }
    }
}
