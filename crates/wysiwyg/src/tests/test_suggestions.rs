// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::MenuAction;

use super::testutils_composer_model::{cm, tx};

#[test]
fn test_replace_text_suggestion() {
    let mut model = cm("|");
    let update = model.replace_text("/".into());
    let MenuAction::Suggestion(suggestion) = update.menu_action else {
        panic!("No suggestion pattern found")
    };
    model.replace_text_suggestion("/invite".into(), suggestion, true);
    assert_eq!(tx(&model), "/invite&nbsp;|");
}
