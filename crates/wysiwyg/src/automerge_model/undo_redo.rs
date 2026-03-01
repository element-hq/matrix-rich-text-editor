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

//! Undo / Redo using Automerge heads snapshots.
//!
//! Before each mutating operation, the current `get_heads()` and selection
//! are pushed onto the undo stack. `undo()` forks at the saved heads,
//! effectively reverting to that state. `redo()` does the reverse.

use automerge::ReadDoc;

use super::AutomergeModel;
use crate::ComposerUpdate;

impl AutomergeModel {
    /// Undo the last editing operation.
    pub fn undo(&mut self) -> ComposerUpdate<String> {
        if let Some((heads, sel_start, sel_end)) = self.undo_stack.pop()
        {
            // Save current state for redo
            let current_heads = self.doc.get_heads();
            self.redo_stack.push((
                current_heads,
                self.selection_start,
                self.selection_end,
            ));

            // Fork the document at the saved heads to revert
            // AutoCommit doesn't have fork_at directly, so we
            // isolate at the saved heads instead.
            // TODO: Investigate if isolate() or a different approach
            // is better for undo semantics.
            self.doc.isolate(&heads);

            self.selection_start = sel_start;
            self.selection_end = sel_end;

            self.create_update_replace_all()
        } else {
            ComposerUpdate::keep()
        }
    }

    /// Redo a previously undone operation.
    pub fn redo(&mut self) -> ComposerUpdate<String> {
        if let Some((heads, sel_start, sel_end)) = self.redo_stack.pop()
        {
            // Save current state for undo
            let current_heads = self.doc.get_heads();
            self.undo_stack.push((
                current_heads,
                self.selection_start,
                self.selection_end,
            ));

            self.doc.isolate(&heads);

            self.selection_start = sel_start;
            self.selection_end = sel_end;

            self.create_update_replace_all()
        } else {
            ComposerUpdate::keep()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{AutomergeModel, TextUpdate};

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
    // Undo (mapping test_undo_redo.rs)
    // ===================================================================

    #[test]
    fn inserting_text_creates_undo_entry() {
        let mut model = new_model();
        assert!(model.undo_stack.is_empty());
        model.replace_text("hello world!");
        assert!(!model.undo_stack.is_empty());
    }

    #[test]
    fn backspacing_text_creates_undo_entry() {
        let mut model = model_with_text("hello world!");
        let old_len = model.undo_stack.len();
        model.backspace();
        assert!(model.undo_stack.len() > old_len);
    }

    #[test]
    fn deleting_text_creates_undo_entry() {
        let mut model = model_with_text("hello world!");
        model.select(5, 5);
        let old_len = model.undo_stack.len();
        model.delete();
        assert!(model.undo_stack.len() > old_len);
    }

    #[test]
    fn formatting_text_creates_undo_entry() {
        let mut model = model_with_text("hello world!");
        model.select(0, 5);
        let old_len = model.undo_stack.len();
        model.bold();
        assert!(model.undo_stack.len() > old_len);
    }

    #[test]
    fn undo_restores_previous_text() {
        let mut model = new_model();
        model.replace_text("hello");
        assert_eq!(plain(&model), "hello");
        model.undo();
        assert_eq!(plain(&model), "");
    }

    #[test]
    fn undo_on_empty_model_is_noop() {
        let mut model = new_model();
        let update = model.undo();
        assert_eq!(update.text_update, TextUpdate::Keep);
    }

    #[test]
    fn undo_removes_last_undo_entry() {
        let mut model = model_with_text("hello");
        assert!(!model.undo_stack.is_empty());
        model.undo();
        assert!(model.undo_stack.is_empty());
    }

    #[test]
    fn undo_adds_to_redo_stack() {
        let mut model = model_with_text("hello");
        assert!(model.redo_stack.is_empty());
        model.undo();
        assert!(!model.redo_stack.is_empty());
    }

    #[test]
    fn can_undo_pressing_enter() {
        let mut model = model_with_text("Test");
        model.enter();
        model.undo();
        assert_eq!(plain(&model), "Test");
    }

    #[test]
    fn can_undo_with_selection() {
        let mut model = model_with_text("Testfoobar");
        model.select(4, 7); // "foo"
        model.enter();
        model.undo();
        assert_eq!(plain(&model), "Testfoobar");
    }

    // ===================================================================
    // Redo
    // ===================================================================

    #[test]
    fn redo_restores_undone_text() {
        let mut model = new_model();
        model.replace_text("hello");
        model.undo();
        assert_eq!(plain(&model), "");
        model.redo();
        assert_eq!(plain(&model), "hello");
    }

    #[test]
    fn redo_on_empty_model_is_noop() {
        let mut model = new_model();
        let update = model.redo();
        assert_eq!(update.text_update, TextUpdate::Keep);
    }

    #[test]
    fn redo_pops_from_redo_stack() {
        let mut model = model_with_text("hello");
        model.undo();
        assert!(!model.redo_stack.is_empty());
        model.redo();
        assert!(model.redo_stack.is_empty());
    }

    #[test]
    fn redo_pushes_to_undo_stack() {
        let mut model = model_with_text("hello");
        model.undo();
        assert!(model.undo_stack.is_empty());
        model.redo();
        assert!(!model.undo_stack.is_empty());
    }

    // ===================================================================
    // Multiple undo/redo steps
    // ===================================================================

    #[test]
    fn multiple_undo_steps() {
        let mut model = new_model();
        model.replace_text("a");
        model.replace_text("b");
        model.replace_text("c");
        assert_eq!(plain(&model), "abc");

        model.undo(); // undo "c"
        assert_eq!(plain(&model), "ab");

        model.undo(); // undo "b"
        assert_eq!(plain(&model), "a");

        model.undo(); // undo "a"
        assert_eq!(plain(&model), "");
    }

    #[test]
    fn undo_redo_undo_cycle() {
        let mut model = new_model();
        model.replace_text("hello");
        model.undo();
        assert_eq!(plain(&model), "");
        model.redo();
        assert_eq!(plain(&model), "hello");
        model.undo();
        assert_eq!(plain(&model), "");
    }

    #[test]
    fn new_edit_after_undo_clears_redo() {
        let mut model = new_model();
        model.replace_text("hello");
        model.undo();
        // Now make a new edit
        model.replace_text("world");
        assert!(model.redo_stack.is_empty());
        assert_eq!(plain(&model), "world");
    }

    #[test]
    fn undo_enter() {
        let mut model = model_with_text("Test");
        model.enter();
        let p = plain(&model);
        assert!(p.contains('\n'), "expected newline after enter");
        model.undo();
        assert_eq!(plain(&model), "Test");
    }

    #[test]
    fn undoing_enter_only_undoes_one() {
        let mut model = model_with_text("Test");
        model.enter();
        model.enter();
        model.undo();
        // Should still have "Test\n" (one enter remaining)
        let p = plain(&model);
        assert_eq!(p, "Test\n");
    }

    #[test]
    fn replacing_text_with_newlines_only_adds_one_to_undo_stack() {
        let mut model = model_with_text("abc");
        let stack_before = model.undo_stack.len();
        model.replace_text("def\nghi");
        assert_eq!(model.undo_stack.len(), stack_before + 1);
        model.replace_text("\njkl\n");
        assert_eq!(model.undo_stack.len(), stack_before + 2);
        model.undo();
        model.undo();
        assert_eq!(plain(&model), "abc");
    }
}
