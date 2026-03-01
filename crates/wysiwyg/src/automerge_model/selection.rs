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

//! Selection management.

use super::AutomergeModel;
use crate::ComposerUpdate;

impl AutomergeModel {
    /// Set the selection/cursor position (UTF-16 code unit offsets).
    pub fn select(
        &mut self,
        start: usize,
        end: usize,
    ) -> ComposerUpdate<String> {
        self.selection_start = start;
        self.selection_end = end;

        // Clear pending formats when selection changes
        self.pending_formats.clear();

        self.create_update_selection()
    }

    /// Get the current selection as (start, end) UTF-16 offsets.
    pub fn get_selection(&self) -> (usize, usize) {
        (self.selection_start, self.selection_end)
    }
}

#[cfg(test)]
mod tests {
    use crate::{AutomergeModel, Location, TextUpdate};

    fn model_with_text(text: &str) -> AutomergeModel {
        let mut m = AutomergeModel::new();
        m.replace_text(text);
        m
    }

    fn plain(m: &AutomergeModel) -> String {
        m.get_content_as_plain_text()
    }

    // ===================================================================
    // Basic selection (mapping test_selection.rs)
    // ===================================================================

    #[test]
    fn selecting_ascii_characters() {
        let mut model = model_with_text("abcdefgh");
        model.select(0, 1);
        assert_eq!(model.get_selection(), (0, 1));

        model.select(1, 3);
        assert_eq!(model.get_selection(), (1, 3));

        model.select(4, 8);
        assert_eq!(model.get_selection(), (4, 8));
    }

    #[test]
    fn selecting_past_end_stores_selection() {
        let mut model = model_with_text("abcdefgh");
        model.select(4, 9);
        // Selection beyond length is stored (Automerge will clamp on use)
        assert_eq!(model.get_selection(), (4, 9));
    }

    #[test]
    fn selecting_single_utf16_code_unit_characters() {
        // \u{03A9} is Omega, 1 UTF-16 code unit
        let mut model = model_with_text("\u{03A9}\u{03A9}\u{03A9}");
        model.select(0, 1);
        assert_eq!(model.get_selection(), (0, 1));

        model.select(0, 3);
        assert_eq!(model.get_selection(), (0, 3));

        model.select(1, 2);
        assert_eq!(model.get_selection(), (1, 2));
    }

    #[test]
    fn selecting_multiple_utf16_code_unit_characters() {
        // \u{1F4A9} is 💩, 2 UTF-16 code units
        let mut model = model_with_text("\u{1F4A9}\u{1F4A9}\u{1F4A9}");
        model.select(0, 2);
        assert_eq!(model.get_selection(), (0, 2));

        model.select(0, 6);
        assert_eq!(model.get_selection(), (0, 6));

        model.select(2, 4);
        assert_eq!(model.get_selection(), (2, 4));
    }

    #[test]
    fn selecting_complex_characters() {
        // Mix of ASCII, Omega(1), ASCII, Woman Astronaut(7), ASCII
        let mut model = model_with_text(
            "aaa\u{03A9}bbb\u{1F469}\u{1F3FF}\u{200D}\u{1F680}ccc",
        );

        model.select(0, 3);
        assert_eq!(model.get_selection(), (0, 3));

        model.select(0, 4);
        assert_eq!(model.get_selection(), (0, 4));
    }

    #[test]
    fn selecting_creates_a_selection_update() {
        let mut model = model_with_text("abcdef");
        let update = model.select(2, 6);
        if let TextUpdate::Select(s) = update.text_update {
            assert_eq!(s.start, Location::from(2));
            assert_eq!(s.end, Location::from(6));
        } else {
            panic!("TextUpdate should be a selection");
        }
    }

    #[test]
    fn select_clears_pending_formats() {
        let mut model = model_with_text("abc");
        model.bold();
        assert!(!model.pending_formats.is_empty());
        model.select(1, 1);
        assert!(model.pending_formats.is_empty());
    }

    #[test]
    fn select_same_position_is_collapsed_cursor() {
        let mut model = model_with_text("abc");
        model.select(2, 2);
        assert_eq!(model.get_selection(), (2, 2));
        assert!(!model.has_selection());
    }

    #[test]
    fn backward_selection() {
        let mut model = model_with_text("abcdef");
        model.select(5, 2);
        assert_eq!(model.get_selection(), (5, 2));
        // sel_start/sel_end normalise
        assert_eq!(model.sel_start(), 2);
        assert_eq!(model.sel_end(), 5);
    }

    #[test]
    fn select_after_typing_preserves_typed_text() {
        let mut model = model_with_text("abc");
        model.select(1, 1);
        model.replace_text("X");
        model.select(0, 5);
        assert_eq!(plain(&model), "aXbc");
        assert_eq!(model.get_selection(), (0, 5));
    }
}
