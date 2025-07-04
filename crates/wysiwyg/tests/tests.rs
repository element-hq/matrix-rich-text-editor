// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use widestring::Utf16String;
use wysiwyg::{ComposerModel, Location, TextUpdate};

#[test]
fn can_instantiate_a_model_and_call_methods() {
    let mut model = ComposerModel::new();
    model.replace_text(Utf16String::from_str("foo"));
    model.select(Location::from(1), Location::from(2));

    let update = model.bold();

    if let TextUpdate::ReplaceAll(r) = update.text_update {
        assert_eq!(r.replacement_html.to_string(), "f<strong>o</strong>o");
        assert_eq!(r.start, 1);
        assert_eq!(r.end, 2);
    } else {
        panic!("Expected to receive a ReplaceAll response");
    }
}
