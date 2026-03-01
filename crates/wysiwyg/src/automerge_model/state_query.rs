// Copyright 2026 The Matrix.org Foundation C.I.C.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! State queries: action_states for toolbar button states.

use std::collections::HashMap;

use super::AutomergeModel;
use crate::{ActionState, ComposerAction};

impl AutomergeModel {
    /// Get the current action states for all toolbar buttons.
    pub fn action_states(&self) -> HashMap<ComposerAction, ActionState> {
        self.compute_action_states()
    }
}

#[cfg(test)]
mod tests {
    use crate::{ActionState, AutomergeModel, ComposerAction};

    fn new_model() -> AutomergeModel {
        AutomergeModel::new()
    }

    fn model_with_text(text: &str) -> AutomergeModel {
        let mut m = AutomergeModel::new();
        m.replace_text(text);
        m
    }

    // ===================================================================
    // Action states (mapping test_menu_state.rs)
    // ===================================================================

    #[test]
    fn bold_action_is_enabled_when_not_active() {
        let model = model_with_text("hello");
        let states = model.action_states();
        assert_eq!(
            states.get(&ComposerAction::Bold),
            Some(&ActionState::Enabled)
        );
    }

    #[test]
    fn bold_action_is_reversed_when_bold_is_active() {
        let mut model = model_with_text("hello");
        model.select(0, 5);
        model.bold();
        let states = model.action_states();
        assert_eq!(
            states.get(&ComposerAction::Bold),
            Some(&ActionState::Reversed)
        );
    }

    #[test]
    fn italic_action_is_reversed_when_italic_is_active() {
        let mut model = model_with_text("hello");
        model.select(0, 5);
        model.italic();
        let states = model.action_states();
        assert_eq!(
            states.get(&ComposerAction::Italic),
            Some(&ActionState::Reversed)
        );
    }

    #[test]
    fn formatting_updates_reversed_actions() {
        let mut model = model_with_text("hello");
        model.select(0, 5);
        model.bold();
        model.italic();
        model.underline();
        let states = model.action_states();
        assert_eq!(
            states.get(&ComposerAction::Bold),
            Some(&ActionState::Reversed)
        );
        assert_eq!(
            states.get(&ComposerAction::Italic),
            Some(&ActionState::Reversed)
        );
        assert_eq!(
            states.get(&ComposerAction::Underline),
            Some(&ActionState::Reversed)
        );
    }

    #[test]
    fn undo_action_disabled_initially() {
        let model = new_model();
        let states = model.action_states();
        assert_eq!(
            states.get(&ComposerAction::Undo),
            Some(&ActionState::Disabled)
        );
    }

    #[test]
    fn undo_action_enabled_after_mutation() {
        let mut model = new_model();
        model.replace_text("hello");
        let states = model.action_states();
        assert_eq!(
            states.get(&ComposerAction::Undo),
            Some(&ActionState::Enabled)
        );
    }

    #[test]
    fn redo_action_disabled_initially() {
        let model = new_model();
        let states = model.action_states();
        assert_eq!(
            states.get(&ComposerAction::Redo),
            Some(&ActionState::Disabled)
        );
    }

    #[test]
    fn redo_action_enabled_after_undo() {
        let mut model = new_model();
        model.replace_text("hello");
        model.undo();
        let states = model.action_states();
        assert_eq!(
            states.get(&ComposerAction::Redo),
            Some(&ActionState::Enabled)
        );
    }

    #[test]
    fn redo_disabled_after_redo() {
        let mut model = new_model();
        model.replace_text("hello");
        model.undo();
        model.redo();
        let states = model.action_states();
        assert_eq!(
            states.get(&ComposerAction::Redo),
            Some(&ActionState::Disabled)
        );
    }

    #[test]
    fn undo_disabled_after_all_undone() {
        let mut model = new_model();
        model.replace_text("hello");
        model.undo();
        let states = model.action_states();
        assert_eq!(
            states.get(&ComposerAction::Undo),
            Some(&ActionState::Disabled)
        );
    }

    #[test]
    fn link_action_enabled_by_default() {
        let model = new_model();
        let states = model.action_states();
        assert_eq!(
            states.get(&ComposerAction::Link),
            Some(&ActionState::Enabled)
        );
    }

    #[test]
    fn all_inline_formats_default_to_enabled() {
        let model = new_model();
        let states = model.action_states();
        for action in [
            ComposerAction::Bold,
            ComposerAction::Italic,
            ComposerAction::StrikeThrough,
            ComposerAction::Underline,
            ComposerAction::InlineCode,
        ] {
            assert_eq!(
                states.get(&action),
                Some(&ActionState::Enabled),
                "expected {:?} to be Enabled",
                action
            );
        }
    }

    #[test]
    fn formatting_zero_length_selection_updates_actions_via_pending() {
        let mut model = model_with_text("aaabbb");
        model.select(3, 3); // collapsed cursor
        model.bold();
        model.underline();
        let states = model.action_states();
        assert_eq!(
            states.get(&ComposerAction::Bold),
            Some(&ActionState::Reversed),
            "bold should be reversed via pending"
        );
        assert_eq!(
            states.get(&ComposerAction::Underline),
            Some(&ActionState::Reversed),
            "underline should be reversed via pending"
        );
    }

    #[test]
    fn selecting_restores_action_states() {
        let mut model = model_with_text("aaabbb");
        model.select(3, 3);
        model.bold();
        model.underline();
        // Selecting elsewhere clears pending
        model.select(1, 1);
        let states = model.action_states();
        assert_eq!(
            states.get(&ComposerAction::Bold),
            Some(&ActionState::Enabled),
            "bold should be enabled after selection change"
        );
    }

    #[test]
    fn inline_code_reversed_when_active() {
        let mut model = model_with_text("hello");
        model.select(0, 5);
        model.inline_code();
        let states = model.action_states();
        assert_eq!(
            states.get(&ComposerAction::InlineCode),
            Some(&ActionState::Reversed)
        );
    }

    #[test]
    fn strikethrough_reversed_when_active() {
        let mut model = model_with_text("hello");
        model.select(0, 5);
        model.strike_through();
        let states = model.action_states();
        assert_eq!(
            states.get(&ComposerAction::StrikeThrough),
            Some(&ActionState::Reversed)
        );
    }
}
