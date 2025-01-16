// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use widestring::Utf16String;

use crate::{
    tests::testutils_composer_model::tx, ComposerModel, MenuAction, PatternKey,
};

#[test]
fn can_do_plain_text_to_empji_replacement() {
    let mut model: ComposerModel<Utf16String> = ComposerModel::new();
    model.set_custom_suggestion_patterns(vec![":)".into()]);
    let update = model.replace_text("Hey That's great! :)".into());
    let MenuAction::Suggestion(suggestion) = update.menu_action else {
        panic!("No suggestion pattern found")
    };
    assert_eq!(suggestion.key, PatternKey::Custom(":)".into()),);
    model.replace_text_suggestion("🙂".into(), suggestion, false);

    assert_eq!(tx(&model), "Hey That's great! 🙂|");
}
