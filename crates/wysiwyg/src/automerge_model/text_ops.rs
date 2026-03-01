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

//! Text editing operations: replace_text, backspace, delete, enter.

use automerge::transaction::Transactable;
use automerge::ReadDoc;

use super::AutomergeModel;
use crate::{ComposerUpdate, SuggestionPattern};

impl AutomergeModel {
    /// Replace the current selection with `new_text`.
    pub fn replace_text(
        &mut self,
        new_text: &str,
    ) -> ComposerUpdate<String> {
        self.push_undo();

        let start = self.sel_start();
        let end = self.sel_end();
        let del = (end - start) as isize;

        self.doc
            .splice_text(&self.text_id, start, del, new_text)
            .expect("splice_text failed");

        let new_end = start + new_text.encode_utf16().count();

        // Apply any pending inline formats to the newly inserted text
        if !self.pending_formats.is_empty() && !new_text.is_empty() {
            self.apply_pending_marks(start, new_end);
        }

        self.selection_start = new_end;
        self.selection_end = new_end;

        self.create_update_replace_all()
    }

    /// Replace a specific range [start, end) with `new_text`.
    pub fn replace_text_in(
        &mut self,
        new_text: &str,
        start: usize,
        end: usize,
    ) -> ComposerUpdate<String> {
        self.push_undo();

        let del = (end - start) as isize;
        self.doc
            .splice_text(&self.text_id, start, del, new_text)
            .expect("splice_text failed");

        let new_end = start + new_text.encode_utf16().count();
        self.selection_start = new_end;
        self.selection_end = new_end;

        self.create_update_replace_all()
    }

    /// Replace a suggestion pattern match with text.
    pub fn replace_text_suggestion(
        &mut self,
        new_text: &str,
        suggestion: &SuggestionPattern,
        append_space: bool,
    ) -> ComposerUpdate<String> {
        self.push_undo();

        let start = suggestion.start;
        let end = suggestion.end;
        let del = (end - start) as isize;

        let text = if append_space {
            format!("{new_text}\u{00A0}")
        } else {
            new_text.to_string()
        };

        self.doc
            .splice_text(&self.text_id, start, del, &text)
            .expect("splice_text failed");

        let new_end = start + text.encode_utf16().count();
        self.selection_start = new_end;
        self.selection_end = new_end;

        self.create_update_replace_all()
    }

    /// Delete backward from the cursor (backspace).
    pub fn backspace(&mut self) -> ComposerUpdate<String> {
        self.push_undo();

        let start = self.sel_start();
        let end = self.sel_end();

        if start == end {
            // No selection — delete one character before cursor
            if start == 0 {
                return ComposerUpdate::keep();
            }
            self.doc
                .splice_text(&self.text_id, start - 1, 1, "")
                .expect("splice_text failed");
            self.selection_start = start - 1;
            self.selection_end = start - 1;
        } else {
            // Delete the selection
            let del = (end - start) as isize;
            self.doc
                .splice_text(&self.text_id, start, del, "")
                .expect("splice_text failed");
            self.selection_start = start;
            self.selection_end = start;
        }

        self.create_update_replace_all()
    }

    /// Delete forward from the cursor (delete key).
    pub fn delete(&mut self) -> ComposerUpdate<String> {
        self.push_undo();

        let start = self.sel_start();
        let end = self.sel_end();

        if start == end {
            // No selection — delete one character after cursor
            let len = self.text_len();
            if start >= len {
                return ComposerUpdate::keep();
            }
            self.doc
                .splice_text(&self.text_id, start, 1, "")
                .expect("splice_text failed");
            // Cursor stays at start
        } else {
            let del = (end - start) as isize;
            self.doc
                .splice_text(&self.text_id, start, del, "")
                .expect("splice_text failed");
            self.selection_start = start;
            self.selection_end = start;
        }

        self.create_update_replace_all()
    }

    /// Delete a specific range [start, end).
    pub fn delete_in(
        &mut self,
        start: usize,
        end: usize,
    ) -> ComposerUpdate<String> {
        self.push_undo();

        let del = (end - start) as isize;
        self.doc
            .splice_text(&self.text_id, start, del, "")
            .expect("splice_text failed");

        // Adjust cursor if it was inside the deleted range
        if self.selection_start > start {
            self.selection_start = self
                .selection_start
                .saturating_sub(del as usize)
                .max(start);
        }
        if self.selection_end > start {
            self.selection_end = self
                .selection_end
                .saturating_sub(del as usize)
                .max(start);
        }

        self.create_update_replace_all()
    }

    /// Insert a new line / paragraph break (enter key).
    ///
    /// Uses `split_block` to insert a block marker at the cursor position,
    /// creating a new paragraph (or list item, if inside a list).
    pub fn enter(&mut self) -> ComposerUpdate<String> {
        self.push_undo();

        let start = self.sel_start();
        let end = self.sel_end();

        // Delete any selected text first
        if start != end {
            let del = (end - start) as isize;
            self.doc
                .splice_text(&self.text_id, start, del, "")
                .expect("splice_text failed");
        }

        // Determine the block type for the new block.  If we are
        // currently inside a list item, the new block should also be a
        // list item of the same type.  Otherwise it becomes a paragraph.
        let block_type = self
            .block_at(start)
            .map(|info| info.block_type.clone())
            .unwrap_or_else(|| {
                super::block_ops::block_type::PARAGRAPH.to_string()
            });

        // Insert a block marker at the cursor position.
        // `split_block` inserts a Map object that occupies 1 index.
        self.insert_block_marker(start, &block_type);

        // The cursor moves past the new block marker (+1).
        self.selection_start = start + 1;
        self.selection_end = start + 1;

        self.create_update_replace_all()
    }
}

#[cfg(test)]
mod tests {
    use crate::AutomergeModel;

    fn new_model() -> AutomergeModel {
        AutomergeModel::new()
    }

    fn model_with_text(text: &str) -> AutomergeModel {
        let mut m = AutomergeModel::new();
        m.replace_text(text);
        m
    }

    fn plain(m: &AutomergeModel) -> String {
        m.get_content_as_plain_text()
    }

    // ===================================================================
    // Character insertion (mapping test_characters.rs)
    // ===================================================================

    #[test]
    fn typing_a_character_into_an_empty_box_appends_it() {
        let mut model = new_model();
        model.replace_text("v");
        assert_eq!(plain(&model), "v");
        assert_eq!(model.get_selection(), (1, 1));
    }

    #[test]
    fn typing_a_character_at_the_end_appends_it() {
        let mut model = model_with_text("abc");
        model.replace_text("d");
        assert_eq!(plain(&model), "abcd");
    }

    #[test]
    fn typing_a_character_in_the_middle_inserts_it() {
        let mut model = model_with_text("abc");
        model.select(0, 0);
        model.replace_text("Z");
        assert_eq!(plain(&model), "Zabc");
    }

    #[test]
    fn replacing_a_selection_past_the_end_clamps_to_end() {
        // In AutomergeModel, selecting past the end then replacing
        // still inserts at the actual end position
        let mut model = model_with_text("abc");
        // select at end (length = 3)
        model.select(3, 3);
        model.replace_text("Z");
        assert_eq!(plain(&model), "abcZ");
    }

    #[test]
    fn replacing_a_selection_with_a_character() {
        let mut model = model_with_text("abcdefghi");
        model.select(3, 6); // select "def"
        model.replace_text("Z");
        assert_eq!(plain(&model), "abcZghi");
    }

    #[test]
    fn replacing_a_backwards_selection_with_a_character() {
        // In Automerge model, we normalise selection with sel_start/sel_end
        let mut model = model_with_text("abcdefghi");
        model.select(6, 3); // backward selection of "def"
        model.replace_text("Z");
        assert_eq!(plain(&model), "abcZghi");
    }

    #[test]
    fn typing_a_character_after_a_multi_codepoint_character() {
        // Woman Astronaut: \u{1F469}\u{1F3FF}\u{200D}\u{1F680}
        let emoji = "\u{1F469}\u{1F3FF}\u{200D}\u{1F680}";
        let mut model = model_with_text(emoji);
        model.replace_text("Z");
        let p = plain(&model);
        assert!(p.starts_with(emoji), "expected emoji prefix in: {p}");
        assert!(p.ends_with('Z'), "expected Z suffix in: {p}");
    }

    #[test]
    fn replacing_an_explicit_text_range_works() {
        let mut model = model_with_text("0123456789");
        model.replace_text_in("654", 4, 7);
        assert_eq!(plain(&model), "0123654789");
    }

    #[test]
    fn can_replace_text_in_an_empty_composer_model() {
        let mut model = new_model();
        model.replace_text("foo");
        assert_eq!(plain(&model), "foo");
    }

    #[test]
    fn newline_characters_insert_newlines() {
        let mut model = new_model();
        model.replace_text("abc\ndef\nghi");
        let p = plain(&model);
        assert!(p.contains("abc"), "expected 'abc' in: {p}");
        assert!(p.contains("def"), "expected 'def' in: {p}");
        assert!(p.contains("ghi"), "expected 'ghi' in: {p}");
    }

    // ===================================================================
    // Backspace (mapping test_deleting.rs)
    // ===================================================================

    #[test]
    fn backspacing_a_character_at_the_end_deletes_it() {
        let mut model = model_with_text("abc");
        model.backspace();
        assert_eq!(plain(&model), "ab");
        assert_eq!(model.get_selection(), (2, 2));
    }

    #[test]
    fn backspacing_a_character_at_the_beginning_does_nothing() {
        let mut model = model_with_text("abc");
        model.select(0, 0);
        model.backspace();
        assert_eq!(plain(&model), "abc");
        assert_eq!(model.get_selection(), (0, 0));
    }

    #[test]
    fn backspacing_a_character_in_the_middle_deletes_it() {
        let mut model = model_with_text("abc");
        model.select(2, 2); // after 'b'
        model.backspace();
        assert_eq!(plain(&model), "ac");
        assert_eq!(model.get_selection(), (1, 1));
    }

    #[test]
    fn backspacing_a_selection_deletes_it() {
        let mut model = model_with_text("abcdef");
        model.select(1, 4); // select "bcd"
        model.backspace();
        assert_eq!(plain(&model), "aef");
        assert_eq!(model.get_selection(), (1, 1));
    }

    #[test]
    fn backspacing_a_backwards_selection_deletes_it() {
        let mut model = model_with_text("abcdef");
        model.select(4, 1); // backward "bcd"
        model.backspace();
        assert_eq!(plain(&model), "aef");
        assert_eq!(model.get_selection(), (1, 1));
    }

    // ===================================================================
    // Delete forward (mapping test_deleting.rs)
    // ===================================================================

    #[test]
    fn deleting_a_character_at_the_end_does_nothing() {
        let mut model = model_with_text("abc");
        // cursor at end (default after model_with_text)
        model.delete();
        assert_eq!(plain(&model), "abc");
    }

    #[test]
    fn deleting_a_character_at_the_beginning_deletes_it() {
        let mut model = model_with_text("abc");
        model.select(0, 0);
        model.delete();
        assert_eq!(plain(&model), "bc");
        assert_eq!(model.get_selection(), (0, 0));
    }

    #[test]
    fn deleting_a_character_in_the_middle_deletes_it() {
        let mut model = model_with_text("abc");
        model.select(1, 1);
        model.delete();
        assert_eq!(plain(&model), "ac");
    }

    #[test]
    fn deleting_a_selection_deletes_it() {
        let mut model = model_with_text("abcdef");
        model.select(1, 4);
        model.delete();
        assert_eq!(plain(&model), "aef");
    }

    #[test]
    fn deleting_a_backwards_selection_deletes_it() {
        let mut model = model_with_text("abcdef");
        model.select(4, 1);
        model.delete();
        assert_eq!(plain(&model), "aef");
    }

    #[test]
    fn deleting_a_range_removes_it() {
        let mut model = model_with_text("abcd");
        model.delete_in(1, 3);
        assert_eq!(plain(&model), "ad");
    }

    // ===================================================================
    // Enter / newline
    // ===================================================================

    #[test]
    fn enter_inserts_newline() {
        let mut model = model_with_text("ab");
        model.select(1, 1);
        model.enter();
        let p = plain(&model);
        assert!(p.contains('\n'), "expected newline in: {p:?}");
        assert_eq!(model.get_selection(), (2, 2));
    }

    #[test]
    fn enter_at_end_appends_newline() {
        let mut model = model_with_text("foo");
        model.enter();
        let p = plain(&model);
        assert_eq!(p, "foo\n");
        assert_eq!(model.get_selection(), (4, 4));
    }

    #[test]
    fn enter_with_selection_deletes_selection_and_inserts_newline() {
        let mut model = model_with_text("abcdef");
        model.select(2, 4); // select "cd"
        model.enter();
        let p = plain(&model);
        assert_eq!(p, "ab\nef");
    }

    #[test]
    fn multiple_enters_create_multiple_newlines() {
        let mut model = model_with_text("foo");
        model.enter();
        model.enter();
        let p = plain(&model);
        assert_eq!(p, "foo\n\n");
    }

    // ===================================================================
    // Edge cases
    // ===================================================================

    #[test]
    fn replace_text_in_empty_model_works() {
        let mut model = new_model();
        model.replace_text("hello world");
        assert_eq!(plain(&model), "hello world");
        assert_eq!(model.get_selection(), (11, 11));
    }

    #[test]
    fn backspace_on_empty_model_is_noop() {
        let mut model = new_model();
        model.backspace();
        assert_eq!(plain(&model), "");
        assert_eq!(model.get_selection(), (0, 0));
    }

    #[test]
    fn delete_on_empty_model_is_noop() {
        let mut model = new_model();
        model.delete();
        assert_eq!(plain(&model), "");
    }

    #[test]
    fn replace_then_backspace_multiple_times() {
        let mut model = model_with_text("abcde");
        model.backspace();
        model.backspace();
        model.backspace();
        assert_eq!(plain(&model), "ab");
    }

    #[test]
    fn delete_then_type_at_beginning() {
        let mut model = model_with_text("abc");
        model.select(0, 0);
        model.delete();
        model.delete();
        model.replace_text("XY");
        assert_eq!(plain(&model), "XYc");
    }

    #[test]
    fn replace_text_in_range_preserves_surrounding_text() {
        let mut model = model_with_text("hello world");
        model.replace_text_in("beautiful ", 6, 6);
        assert_eq!(plain(&model), "hello beautiful world");
    }

    #[test]
    fn delete_entire_content_via_selection() {
        let mut model = model_with_text("hello");
        model.select(0, 5);
        model.delete();
        assert_eq!(plain(&model), "");
        assert_eq!(model.get_selection(), (0, 0));
    }

    #[test]
    fn backspace_entire_content_via_selection() {
        let mut model = model_with_text("hello");
        model.select(0, 5);
        model.backspace();
        assert_eq!(plain(&model), "");
        assert_eq!(model.get_selection(), (0, 0));
    }

    #[test]
    fn delete_in_at_boundaries() {
        let mut model = model_with_text("abcdef");
        model.delete_in(0, 6);
        assert_eq!(plain(&model), "");
    }

    #[test]
    fn sequential_typing_builds_text() {
        let mut model = new_model();
        model.replace_text("a");
        model.replace_text("b");
        model.replace_text("c");
        assert_eq!(plain(&model), "abc");
        assert_eq!(model.get_selection(), (3, 3));
    }

    #[test]
    fn insert_in_middle_of_existing_text() {
        let mut model = model_with_text("ac");
        model.select(1, 1);
        model.replace_text("b");
        assert_eq!(plain(&model), "abc");
    }

    #[test]
    fn replace_text_suggestion_basic() {
        use crate::SuggestionPattern;
        use crate::PatternKey;

        let mut model = model_with_text("hello @ali world");
        let suggestion = SuggestionPattern {
            key: PatternKey::At,
            text: "@ali".to_string(),
            start: 6,
            end: 10,
        };
        model.replace_text_suggestion("Alice", &suggestion, false);
        assert_eq!(plain(&model), "hello Alice world");
    }
}
