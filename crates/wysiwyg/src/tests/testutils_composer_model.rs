// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use widestring::Utf16String;

use crate::{ComposerModel, Location};

/// Short wrapper around [ComposerModel::from_example_format].
pub fn cm(text: &str) -> ComposerModel<Utf16String> {
    ComposerModel::<Utf16String>::from_example_format(text)
}

/// Short wrapper around [ComposerModel::to_example_format].
pub fn tx(model: &ComposerModel<Utf16String>) -> String {
    model.to_example_format()
}

#[allow(dead_code)]
pub(crate) fn sel(start: usize, end: usize) -> (Location, Location) {
    (Location::from(start), Location::from(end))
}

pub(crate) fn restore_whitespace(text: &str) -> String {
    text.replace("&nbsp;", " ").replace('\u{A0}', " ")
}

pub(crate) fn restore_whitespace_u16(text: &Utf16String) -> Utf16String {
    Utf16String::from(restore_whitespace(&text.to_string()))
}
