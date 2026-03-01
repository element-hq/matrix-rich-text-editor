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

//! Content access: serialization to HTML, Markdown, and plain text.

use automerge::transaction::Transactable;
use automerge::ReadDoc;

use super::AutomergeModel;
use crate::ComposerUpdate;

impl AutomergeModel {
    /// Set content from HTML.
    pub fn set_content_from_html(
        &mut self,
        html: &str,
    ) -> ComposerUpdate<String> {
        // Clear existing content
        let len = self.text_len();
        if len > 0 {
            let _ = self
                .doc
                .splice_text(&self.text_id, 0, len as isize, "");
        }

        self.set_content_from_html_internal(html);

        self.selection_start = 0;
        self.selection_end = 0;
        self.undo_stack.clear();
        self.redo_stack.clear();

        self.create_update_replace_all()
    }

    /// Set content from Markdown.
    pub fn set_content_from_markdown(
        &mut self,
        _markdown: &str,
    ) -> ComposerUpdate<String> {
        // TODO: Parse Markdown → intermediate → Automerge spans
        // For now, treat as plain text
        let len = self.text_len();
        if len > 0 {
            let _ = self
                .doc
                .splice_text(&self.text_id, 0, len as isize, "");
        }

        // Stub: just insert markdown as text
        let _ = self.doc.splice_text(&self.text_id, 0, 0, _markdown);

        self.selection_start = 0;
        self.selection_end = 0;
        self.undo_stack.clear();
        self.redo_stack.clear();

        self.create_update_replace_all()
    }

    /// Return internal HTML representation of the document.
    pub fn get_content_as_html(&self) -> String {
        self.spans_to_html()
    }

    /// Return clean HTML suitable for a Matrix message.
    pub fn get_content_as_message_html(&self) -> String {
        // TODO: Additional cleanup (strip internal attributes, etc.)
        self.spans_to_html()
    }

    /// Return Markdown representation.
    pub fn get_content_as_markdown(&self) -> String {
        // TODO: Implement proper spans → Markdown conversion
        self.get_content_as_plain_text()
    }

    /// Return clean Markdown suitable for a Matrix message.
    pub fn get_content_as_message_markdown(&self) -> String {
        // TODO: Full markdown serialization
        self.get_content_as_plain_text()
    }

    /// Return plain text (all formatting stripped).
    ///
    /// Block markers (which Automerge represents as `\u{fffc}`) are
    /// converted to newline characters.
    pub fn get_content_as_plain_text(&self) -> String {
        self.doc
            .text(&self.text_id)
            .unwrap_or_default()
            .replace('\u{fffc}', "\n")
    }

    /// Clear all content and return to empty state.
    pub fn clear(&mut self) -> ComposerUpdate<String> {
        self.push_undo();

        let len = self.text_len();
        if len > 0 {
            let _ = self
                .doc
                .splice_text(&self.text_id, 0, len as isize, "");
        }

        self.selection_start = 0;
        self.selection_end = 0;
        self.pending_formats.clear();

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

    fn html(m: &AutomergeModel) -> String {
        m.get_content_as_html()
    }

    // ===================================================================
    // set_content_from_html (mapping test_set_content.rs)
    // ===================================================================

    #[test]
    fn set_content_from_html_stub_clears_model() {
        // Note: HTML parsing is currently a stub (inserts nothing)
        let mut model = model_with_text("existing");
        model.set_content_from_html("content");
        // The stub clears existing text but doesn't insert new text
        let p = plain(&model);
        // With stub, model is cleared
        assert_eq!(p, "");
    }

    #[test]
    fn set_content_from_html_resets_selection() {
        let mut model = model_with_text("existing");
        model.set_content_from_html("new");
        assert_eq!(model.get_selection(), (0, 0));
    }

    // ===================================================================
    // set_content_from_markdown
    // ===================================================================

    #[test]
    fn set_content_from_markdown_inserts_as_plain_text() {
        // Currently a stub that inserts markdown as plain text
        let mut model = new_model();
        model.set_content_from_markdown("**bold**");
        let p = plain(&model);
        assert_eq!(p, "**bold**");
    }

    #[test]
    fn set_content_from_markdown_clears_existing() {
        let mut model = model_with_text("existing");
        model.set_content_from_markdown("new text");
        assert_eq!(plain(&model), "new text");
    }

    // ===================================================================
    // get_content_as_html
    // ===================================================================

    #[test]
    fn html_of_plain_text_is_just_text() {
        let model = model_with_text("plain");
        assert_eq!(html(&model), "plain");
    }

    #[test]
    fn html_of_empty_model_is_empty() {
        let model = new_model();
        assert_eq!(html(&model), "");
    }

    #[test]
    fn html_of_bold_text_contains_strong() {
        let mut model = model_with_text("hello");
        model.select(0, 5);
        model.bold();
        let h = html(&model);
        assert!(h.contains("<strong>"), "expected <strong> in: {h}");
    }

    // ===================================================================
    // get_content_as_plain_text
    // ===================================================================

    #[test]
    fn plain_text_matches_inserted_text() {
        let model = model_with_text("Hello, world!");
        assert_eq!(plain(&model), "Hello, world!");
    }

    #[test]
    fn plain_text_strips_formatting() {
        let mut model = model_with_text("hello world");
        model.select(0, 5);
        model.bold();
        // Plain text should have no formatting
        assert_eq!(plain(&model), "hello world");
    }

    #[test]
    fn plain_text_of_empty_model() {
        let model = new_model();
        assert_eq!(plain(&model), "");
    }

    // ===================================================================
    // get_content_as_markdown / message variants
    // ===================================================================

    #[test]
    fn markdown_of_plain_text_is_just_text() {
        let model = model_with_text("hello");
        // Currently stub returns plain text
        assert_eq!(model.get_content_as_markdown(), "hello");
    }

    #[test]
    fn message_html_returns_html() {
        let mut model = model_with_text("hello");
        model.select(0, 5);
        model.bold();
        let h = model.get_content_as_message_html();
        assert!(h.contains("<strong>"), "expected <strong> in: {h}");
    }

    #[test]
    fn message_markdown_returns_text() {
        let model = model_with_text("hello");
        assert_eq!(model.get_content_as_message_markdown(), "hello");
    }

    // ===================================================================
    // clear (mapping test_set_content.rs)
    // ===================================================================

    #[test]
    fn clear_empties_the_document() {
        let mut model = model_with_text("hello world");
        model.clear();
        assert_eq!(plain(&model), "");
        assert_eq!(model.get_selection(), (0, 0));
    }

    #[test]
    fn clear_resets_pending_formats() {
        let mut model = model_with_text("hello");
        model.bold(); // set pending
        model.clear();
        assert!(model.pending_formats.is_empty());
    }

    #[test]
    fn clear_allows_new_content() {
        let mut model = model_with_text("hello");
        model.clear();
        model.replace_text("world");
        assert_eq!(plain(&model), "world");
    }
}
