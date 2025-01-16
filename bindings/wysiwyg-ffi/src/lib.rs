// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

uniffi_macros::include_scaffolding!("wysiwyg_composer");

mod ffi_action_state;
mod ffi_composer_action;
mod ffi_composer_model;
mod ffi_composer_state;
mod ffi_composer_update;
mod ffi_dom_creation_error;
mod ffi_link_actions;
mod ffi_mention_detector;
mod ffi_mentions_state;
mod ffi_menu_action;
mod ffi_menu_state;
mod ffi_pattern_key;
mod ffi_suggestion_pattern;
mod ffi_text_update;
mod into_ffi;

use std::sync::Arc;

pub use crate::ffi_action_state::ActionState;
pub use crate::ffi_composer_action::ComposerAction;
pub use crate::ffi_composer_model::Attribute;
pub use crate::ffi_composer_model::ComposerModel;
pub use crate::ffi_composer_state::ComposerState;
pub use crate::ffi_composer_update::ComposerUpdate;
pub use crate::ffi_dom_creation_error::DomCreationError;
pub use crate::ffi_link_actions::LinkAction;
use crate::ffi_mention_detector::MentionDetector;
pub use crate::ffi_mentions_state::MentionsState;
pub use crate::ffi_menu_action::MenuAction;
pub use crate::ffi_menu_state::MenuState;
pub use crate::ffi_pattern_key::PatternKey;
pub use crate::ffi_suggestion_pattern::SuggestionPattern;
pub use crate::ffi_text_update::TextUpdate;

#[uniffi::export]
pub fn new_composer_model() -> Arc<ComposerModel> {
    Arc::new(ComposerModel::new())
}

#[uniffi::export]
pub fn new_mention_detector() -> Arc<MentionDetector> {
    Arc::new(MentionDetector::new())
}
