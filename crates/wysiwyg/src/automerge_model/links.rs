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

//! Link operations: set, edit, and remove hyperlinks.
//!
//! Links use an Automerge mark named `"link"` with `ExpandMark::None`,
//! meaning the link does not grow when the user types at its boundary.

use automerge::marks::{ExpandMark, Mark};
use automerge::transaction::Transactable;

use super::AutomergeModel;
use crate::composer_model_interface::Attribute;
use crate::{ComposerUpdate, LinkAction};

impl AutomergeModel {
    /// Set a link URL on the current selection.
    pub fn set_link(
        &mut self,
        url: &str,
        _attributes: &[Attribute],
    ) -> ComposerUpdate<String> {
        let start = self.sel_start();
        let end = self.sel_end();

        if start == end {
            return ComposerUpdate::keep();
        }

        self.push_undo();

        let mark = Mark::new("link".to_string(), url, start, end);
        let _ = self.doc.mark(&self.text_id, mark, ExpandMark::None);

        self.create_update_replace_all()
    }

    /// Set a link with explicit display text, replacing the current selection.
    pub fn set_link_with_text(
        &mut self,
        url: &str,
        text: &str,
        _attributes: &[Attribute],
    ) -> ComposerUpdate<String> {
        self.push_undo();

        let start = self.sel_start();
        let end = self.sel_end();

        // Delete the current selection
        if start != end {
            let del = (end - start) as isize;
            self.doc
                .splice_text(&self.text_id, start, del, "")
                .expect("splice_text failed");
        }

        // Insert the link text
        self.doc
            .splice_text(&self.text_id, start, 0, text)
            .expect("splice_text failed");

        let text_end = start + text.encode_utf16().count();

        // Apply the link mark
        let mark = Mark::new("link".to_string(), url, start, text_end);
        let _ = self.doc.mark(&self.text_id, mark, ExpandMark::None);

        self.selection_start = text_end;
        self.selection_end = text_end;

        self.create_update_replace_all()
    }

    /// Remove all links from the current selection.
    ///
    /// When the cursor is collapsed (no selection), finds the full extent
    /// of the link surrounding the cursor and removes it.
    pub fn remove_links(&mut self) -> ComposerUpdate<String> {
        let start = self.sel_start();
        let end = self.sel_end();

        let (unmark_start, unmark_end) = if start == end {
            // Collapsed cursor — find the link extent around the cursor
            match self.find_link_extent(start) {
                Some(extent) => extent,
                None => return ComposerUpdate::keep(),
            }
        } else {
            (start, end)
        };

        self.push_undo();

        let _ = self.doc.unmark(
            &self.text_id,
            "link",
            unmark_start,
            unmark_end,
            ExpandMark::None,
        );

        self.create_update_replace_all()
    }

    /// Find the contiguous range `[start, end)` of text covered by a link
    /// mark at `pos`.  Returns `None` if no link is active at `pos`.
    fn find_link_extent(&self, pos: usize) -> Option<(usize, usize)> {
        use automerge::iter::Span;
        use automerge::ReadDoc;

        let spans = self.doc.spans(&self.text_id).ok()?;

        let mut offset: usize = 0;
        for span in spans {
            match span {
                Span::Text { ref text, ref marks } => {
                    let len = text.encode_utf16().count();
                    let span_end = offset + len;
                    if pos >= offset && pos < span_end {
                        // Cursor is inside this span — check for link mark
                        if let Some(mark_set) = marks {
                            if Self::mark_value_in_set(
                                mark_set, "link",
                            )
                            .is_some()
                            {
                                return Some((offset, span_end));
                            }
                        }
                        return None;
                    }
                    offset = span_end;
                }
                Span::Block(_) => {
                    offset += 1;
                }
            }
        }

        None
    }

    /// Query the link action available at the current cursor position.
    pub fn get_link_action(&self) -> LinkAction<String> {
        self.compute_link_action()
    }
}

#[cfg(test)]
mod tests {
    use crate::{AutomergeModel, LinkAction, TextUpdate};

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
    // set_link (mapping test_links.rs)
    // ===================================================================

    #[test]
    fn set_link_to_empty_selection_is_noop() {
        let mut model = model_with_text("test");
        let update = model.set_link("https://element.io", &[]);
        assert_eq!(update.text_update, TextUpdate::Keep);
    }

    #[test]
    fn set_link_wraps_selection_in_link_tag() {
        let mut model = model_with_text("hello world");
        model.select(0, 5); // "hello"
        model.set_link("https://element.io", &[]);
        let h = html(&model);
        assert!(
            h.contains("href=\"https://element.io\""),
            "expected link href in: {h}"
        );
        assert!(h.contains("hello"), "expected link text in: {h}");
    }

    #[test]
    fn set_link_preserves_plain_text() {
        let mut model = model_with_text("hello world");
        model.select(0, 5);
        model.set_link("https://element.io", &[]);
        assert_eq!(plain(&model), "hello world");
    }

    #[test]
    fn set_link_on_entire_text() {
        let mut model = model_with_text("hello");
        model.select(0, 5);
        model.set_link("https://matrix.org", &[]);
        let h = html(&model);
        assert!(h.contains("href=\"https://matrix.org\""), "expected href in: {h}");
        assert!(h.contains("hello"), "expected text in: {h}");
    }

    #[test]
    fn set_link_partial_selection() {
        let mut model = model_with_text("hello world");
        model.select(6, 11); // "world"
        model.set_link("https://example.com", &[]);
        let h = html(&model);
        assert!(h.contains("hello"), "expected 'hello' outside link in: {h}");
        assert!(
            h.contains("href=\"https://example.com\""),
            "expected link in: {h}"
        );
    }

    #[test]
    fn set_link_on_already_linked_text_overwrites() {
        let mut model = model_with_text("link_text");
        model.select(0, 9);
        model.set_link("https://element.io", &[]);
        // Now re-link with a different URL
        model.select(0, 9);
        model.set_link("https://matrix.org", &[]);
        let h = html(&model);
        assert!(
            h.contains("href=\"https://matrix.org\""),
            "expected new href in: {h}"
        );
    }

    // ===================================================================
    // set_link_with_text
    // ===================================================================

    #[test]
    fn set_link_with_text_inserts_linked_text() {
        let mut model = model_with_text("test");
        model.set_link_with_text("https://element.io", "added_link", &[]);
        let h = html(&model);
        assert!(
            h.contains("href=\"https://element.io\""),
            "expected link href in: {h}"
        );
        assert!(
            h.contains("added_link"),
            "expected 'added_link' in: {h}"
        );
    }

    #[test]
    fn set_link_with_text_at_empty_model() {
        let mut model = new_model();
        model.set_link_with_text("https://matrix.org", "link", &[]);
        let h = html(&model);
        assert!(h.contains("href=\"https://matrix.org\""), "expected href in: {h}");
        assert!(h.contains("link"), "expected 'link' in: {h}");
    }

    #[test]
    fn set_link_with_text_replaces_selection() {
        let mut model = model_with_text("hello world");
        model.select(6, 11); // "world"
        model.set_link_with_text("https://matrix.org", "Matrix", &[]);
        let p = plain(&model);
        assert!(p.contains("Matrix"), "expected 'Matrix' in: {p}");
        assert!(!p.contains("world"), "expected 'world' gone from: {p}");
    }

    #[test]
    fn set_link_with_text_and_undo() {
        let mut model = model_with_text("test");
        model.set_link_with_text("https://element.io", "added_link", &[]);
        assert!(
            html(&model).contains("added_link"),
            "should have link text"
        );
        model.undo();
        assert_eq!(plain(&model), "test");
    }

    #[test]
    fn set_link_with_text_cursor_at_end_of_link_text() {
        let mut model = model_with_text("test");
        model.set_link_with_text("https://matrix.org", "link", &[]);
        // Cursor should be after the link text
        let (start, end) = model.get_selection();
        assert_eq!(start, end);
        assert!(start >= 4 + 4); // "test" + "link"
    }

    // ===================================================================
    // remove_links
    // ===================================================================

    #[test]
    fn remove_links_removes_anchor() {
        let mut model = model_with_text("hello world");
        model.select(0, 5);
        model.set_link("https://element.io", &[]);
        // Now remove
        model.select(0, 5);
        model.remove_links();
        let h = html(&model);
        assert!(
            !h.contains("href"),
            "expected no href after remove_links, got: {h}"
        );
    }

    #[test]
    fn remove_links_preserves_text() {
        let mut model = model_with_text("hello world");
        model.select(0, 5);
        model.set_link("https://element.io", &[]);
        model.select(0, 5);
        model.remove_links();
        assert_eq!(plain(&model), "hello world");
    }

    #[test]
    fn remove_links_with_no_selection_is_noop() {
        let mut model = model_with_text("hello");
        let update = model.remove_links();
        assert_eq!(update.text_update, TextUpdate::Keep);
    }

    // ===================================================================
    // get_link_action
    // ===================================================================

    #[test]
    fn get_link_action_returns_create_with_text_at_cursor() {
        let model = model_with_text("hello");
        let action = model.get_link_action();
        assert_eq!(action, LinkAction::CreateWithText);
    }

    #[test]
    fn get_link_action_returns_create_with_selection() {
        let mut model = model_with_text("hello");
        model.select(0, 3);
        let action = model.get_link_action();
        assert_eq!(action, LinkAction::Create);
    }

    #[test]
    fn get_link_action_returns_edit_inside_link() {
        let mut model = model_with_text("hello");
        model.select(0, 5);
        model.set_link("https://matrix.org", &[]);
        // Place cursor inside the link
        model.select(2, 2);
        let action = model.get_link_action();
        match action {
            LinkAction::Edit(url) => {
                assert_eq!(url, "https://matrix.org");
            }
            other => panic!("expected Edit, got: {:?}", other),
        }
    }

    #[test]
    fn get_link_action_returns_create_with_text_outside_link() {
        let mut model = model_with_text("hello world");
        model.select(0, 5);
        model.set_link("https://matrix.org", &[]);
        // Place cursor outside the link
        model.select(8, 8);
        let action = model.get_link_action();
        assert_eq!(action, LinkAction::CreateWithText);
    }

    // ===================================================================
    // Link + text ops interactions
    // ===================================================================

    #[test]
    fn add_text_after_link_does_not_extend_link() {
        let mut model = model_with_text("hello");
        model.select(0, 5);
        model.set_link("https://element.io", &[]);
        // Move cursor to end and type
        model.select(5, 5);
        model.replace_text(" world");
        let h = html(&model);
        // "world" should NOT be inside the link (ExpandMark::None)
        assert!(h.contains("world"), "expected 'world' in: {h}");
    }

    #[test]
    fn replacing_linked_text_removes_link() {
        let mut model = model_with_text("hello world");
        model.select(0, 5);
        model.set_link("https://element.io", &[]);
        // Replace the linked text
        model.select(0, 5);
        model.replace_text("bye");
        let p = plain(&model);
        assert_eq!(p, "bye world");
    }

    #[test]
    fn link_on_formatted_text() {
        let mut model = model_with_text("hello world");
        model.select(0, 5);
        model.bold();
        model.select(0, 5);
        model.set_link("https://matrix.org", &[]);
        let h = html(&model);
        assert!(h.contains("<strong>"), "expected bold in: {h}");
        assert!(h.contains("href"), "expected link in: {h}");
    }

    #[test]
    fn multiple_links_in_document() {
        let mut model = model_with_text("hello world test");
        model.select(0, 5);
        model.set_link("https://link1.com", &[]);
        model.select(6, 11);
        model.set_link("https://link2.com", &[]);
        let h = html(&model);
        assert!(h.contains("link1.com"), "expected link1 in: {h}");
        assert!(h.contains("link2.com"), "expected link2 in: {h}");
    }
}
