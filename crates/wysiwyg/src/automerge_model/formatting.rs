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

//! Inline formatting: bold, italic, strikethrough, underline, inline code.
//!
//! Each format maps to an Automerge mark with `ExpandMark::Both` so that
//! typing at the boundary continues the format.

use automerge::marks::{ExpandMark, Mark};
use automerge::transaction::Transactable;

use super::AutomergeModel;
use crate::{ComposerUpdate, InlineFormatType};

impl AutomergeModel {
    /// Toggle an inline format on the current selection.
    ///
    /// - **Collapsed cursor**: toggles the format as "pending" so that the
    ///   next inserted text will (or won't) carry the mark.
    /// - **Range selection**: applies or removes the mark on the selected
    ///   range, depending on whether the format is already fully active.
    fn toggle_inline_format(
        &mut self,
        format: &InlineFormatType,
    ) -> ComposerUpdate<String> {
        let mark_name = Self::mark_name_for_format(format);
        let start = self.sel_start();
        let end = self.sel_end();

        if start == end {
            // Collapsed cursor — toggle pending format
            if self.pending_formats.contains(mark_name) {
                self.pending_formats.remove(mark_name);
            } else if self.is_mark_active_at(start, mark_name) {
                // Mark is active at cursor → "un-pending" it
                // We store the name so apply_pending_marks will unmark
                // For now we track as a simple toggle
                self.pending_formats.insert(mark_name.to_string());
            } else {
                self.pending_formats.insert(mark_name.to_string());
            }
            // No text change, just update menu state
            self.create_update_selection()
        } else {
            self.push_undo();

            // Determine if the format is already active across the
            // entire selection by checking at both start and end - 1.
            let is_active = self.is_mark_active_at(start, mark_name)
                && (end <= start + 1
                    || self.is_mark_active_at(end - 1, mark_name));

            if is_active {
                // Remove the mark over the selection
                let _ = self.doc.unmark(
                    &self.text_id,
                    mark_name,
                    start,
                    end,
                    ExpandMark::Both,
                );
            } else {
                // Apply the mark over the selection
                let mark = Mark::new(
                    mark_name.to_string(),
                    true,
                    start,
                    end,
                );
                let _ = self.doc.mark(
                    &self.text_id,
                    mark,
                    ExpandMark::Both,
                );
            }

            self.create_update_replace_all()
        }
    }

    /// Toggle bold.
    pub fn bold(&mut self) -> ComposerUpdate<String> {
        self.toggle_inline_format(&InlineFormatType::Bold)
    }

    /// Toggle italic.
    pub fn italic(&mut self) -> ComposerUpdate<String> {
        self.toggle_inline_format(&InlineFormatType::Italic)
    }

    /// Toggle strikethrough.
    pub fn strike_through(&mut self) -> ComposerUpdate<String> {
        self.toggle_inline_format(&InlineFormatType::StrikeThrough)
    }

    /// Toggle underline.
    pub fn underline(&mut self) -> ComposerUpdate<String> {
        self.toggle_inline_format(&InlineFormatType::Underline)
    }

    /// Toggle inline code.
    pub fn inline_code(&mut self) -> ComposerUpdate<String> {
        self.toggle_inline_format(&InlineFormatType::InlineCode)
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
    // Bold (mapping test_formatting.rs)
    // ===================================================================

    #[test]
    fn selecting_and_bolding_adds_strong_tags() {
        let mut model = model_with_text("aabbcc");
        model.select(2, 4); // select "bb"
        model.bold();
        let h = html(&model);
        assert!(h.contains("<strong>"), "expected <strong> in: {h}");
        assert!(h.contains("bb"), "expected 'bb' in: {h}");
    }

    #[test]
    fn selecting_and_bolding_multiple_times() {
        let mut model = model_with_text("aabbcc");
        model.select(0, 2);
        model.bold();
        model.select(4, 6);
        model.bold();
        let h = html(&model);
        assert!(h.contains("<strong>aa</strong>"), "expected bold 'aa' in: {h}");
        assert!(h.contains("<strong>cc</strong>"), "expected bold 'cc' in: {h}");
    }

    #[test]
    fn selecting_and_unbolding() {
        let mut model = model_with_text("aabbcc");
        // Bold everything
        model.select(0, 6);
        model.bold();
        // Unbold first two chars
        model.select(0, 2);
        model.bold();
        let h = html(&model);
        // "aa" should no longer be bold
        assert!(!h.starts_with("<strong>aa"), "expected 'aa' unbolded in: {h}");
    }

    #[test]
    fn bold_toggle_off_removes_strong() {
        let mut model = model_with_text("aabbcc");
        model.select(2, 4);
        model.bold();
        model.bold();
        let h = html(&model);
        assert!(
            !h.contains("<strong>"),
            "expected no <strong> after double toggle, got: {h}"
        );
    }

    #[test]
    fn formatting_twice_adds_no_formatting() {
        let mut model = model_with_text("aabbbccc");
        model.select(2, 7);
        // Toggle each format on then off
        model.bold();
        model.italic();
        model.strike_through();
        model.underline();
        model.bold();
        model.italic();
        model.strike_through();
        model.underline();
        let h = html(&model);
        assert!(!h.contains("<strong>"), "no strong in: {h}");
        assert!(!h.contains("<em>"), "no em in: {h}");
        assert!(!h.contains("<del>"), "no del in: {h}");
        assert!(!h.contains("<u>"), "no u in: {h}");
    }

    // ===================================================================
    // Italic
    // ===================================================================

    #[test]
    fn italic_on_selection_produces_em_html() {
        let mut model = model_with_text("aabbcc");
        model.select(2, 4);
        model.italic();
        let h = html(&model);
        assert!(h.contains("<em>"), "expected <em> in: {h}");
    }

    #[test]
    fn italic_toggle_off() {
        let mut model = model_with_text("aabbcc");
        model.select(2, 4);
        model.italic();
        model.italic();
        let h = html(&model);
        assert!(!h.contains("<em>"), "expected no <em> after toggle off: {h}");
    }

    // ===================================================================
    // Strikethrough
    // ===================================================================

    #[test]
    fn strikethrough_on_selection_produces_del_html() {
        let mut model = model_with_text("aabbcc");
        model.select(2, 4);
        model.strike_through();
        let h = html(&model);
        assert!(h.contains("<del>"), "expected <del> in: {h}");
    }

    #[test]
    fn strikethrough_toggle_off() {
        let mut model = model_with_text("aabbcc");
        model.select(2, 4);
        model.strike_through();
        model.strike_through();
        let h = html(&model);
        assert!(
            !h.contains("<del>"),
            "expected no <del> after toggle off: {h}"
        );
    }

    // ===================================================================
    // Underline
    // ===================================================================

    #[test]
    fn underline_on_selection_produces_u_html() {
        let mut model = model_with_text("aabbcc");
        model.select(2, 4);
        model.underline();
        let h = html(&model);
        assert!(h.contains("<u>"), "expected <u> in: {h}");
    }

    #[test]
    fn underline_toggle_off() {
        let mut model = model_with_text("aabbcc");
        model.select(2, 4);
        model.underline();
        model.underline();
        let h = html(&model);
        assert!(!h.contains("<u>"), "expected no <u> after toggle off: {h}");
    }

    // ===================================================================
    // Inline code
    // ===================================================================

    #[test]
    fn inline_code_on_selection_produces_code_html() {
        let mut model = model_with_text("aabbcc");
        model.select(2, 4);
        model.inline_code();
        let h = html(&model);
        assert!(h.contains("<code>"), "expected <code> in: {h}");
    }

    #[test]
    fn inline_code_toggle_off() {
        let mut model = model_with_text("aabbcc");
        model.select(2, 4);
        model.inline_code();
        model.inline_code();
        let h = html(&model);
        assert!(
            !h.contains("<code>"),
            "expected no <code> after toggle off: {h}"
        );
    }

    // ===================================================================
    // Multiple formats on same range
    // ===================================================================

    #[test]
    fn multiple_formats_on_same_range() {
        let mut model = model_with_text("abcdef");
        model.select(1, 4);
        model.bold();
        model.italic();
        let h = html(&model);
        assert!(h.contains("<strong>"), "expected <strong> in: {h}");
        assert!(h.contains("<em>"), "expected <em> in: {h}");
    }

    #[test]
    fn bold_italic_and_strikethrough() {
        let mut model = model_with_text("hello");
        model.select(0, 5);
        model.bold();
        model.italic();
        model.strike_through();
        let h = html(&model);
        assert!(h.contains("<strong>"), "expected <strong> in: {h}");
        assert!(h.contains("<em>"), "expected <em> in: {h}");
        assert!(h.contains("<del>"), "expected <del> in: {h}");
    }

    // ===================================================================
    // Pending format for collapsed cursor
    // ===================================================================

    #[test]
    fn formatting_with_zero_length_selection_is_pending() {
        let mut model = model_with_text("aaabbb");
        model.select(3, 3); // collapsed cursor
        model.bold();
        model.italic();
        // No visible change in html yet
        let h = html(&model);
        assert!(!h.contains("<strong>"), "no strong yet in: {h}");
    }

    #[test]
    fn pending_format_applies_on_replace_text() {
        let mut model = model_with_text("aaabbb");
        model.select(3, 3);
        model.bold();
        model.italic();
        model.replace_text("ccc");
        let h = html(&model);
        assert!(h.contains("<strong>"), "expected <strong> in: {h}");
        assert!(h.contains("<em>"), "expected <em> in: {h}");
        assert!(h.contains("ccc"), "expected 'ccc' in: {h}");
    }

    #[test]
    fn formatting_again_removes_pending_format() {
        let mut model = model_with_text("aaa");
        model.bold();
        assert!(!model.pending_formats.is_empty());
        model.bold();
        assert!(model.pending_formats.is_empty());
    }

    #[test]
    fn selecting_clears_pending_formats() {
        let mut model = model_with_text("aaa");
        model.bold();
        assert!(!model.pending_formats.is_empty());
        model.select(1, 1);
        assert!(model.pending_formats.is_empty());
    }

    #[test]
    fn bold_then_type_extends_bold() {
        let mut model = new_model();
        model.bold();
        model.replace_text("hello");
        let h = html(&model);
        assert!(
            h.contains("<strong>hello</strong>"),
            "expected bold text in: {h}"
        );
    }

    #[test]
    fn formatting_before_typing_anything_applies_formatting() {
        let mut model = new_model();
        model.bold();
        model.replace_text("d");
        let h = html(&model);
        assert!(h.contains("<strong>d</strong>"), "expected bold 'd' in: {h}");
    }

    #[test]
    fn formatting_in_an_empty_model_applies_formatting() {
        let mut model = new_model();
        model.bold();
        model.replace_text("d");
        let h = html(&model);
        assert!(h.contains("<strong>"), "expected <strong> in: {h}");
    }

    #[test]
    fn format_empty_model_sets_pending() {
        let mut model = new_model();
        model.bold();
        assert!(model.pending_formats.contains("bold"));
    }

    // ===================================================================
    // Formatting across enter (paragraph break)
    // ===================================================================

    #[test]
    fn splitting_a_formatting_tag_across_two_lines() {
        let mut model = new_model();
        model.strike_through();
        model.replace_text("foo");
        let h = html(&model);
        assert!(h.contains("<del>foo</del>"), "expected del in: {h}");
        // The enter inserts a newline - format continues due to ExpandMark::Both
        model.enter();
        model.replace_text("bar");
        let h = html(&model);
        assert!(h.contains("foo"), "expected 'foo' in: {h}");
        assert!(h.contains("bar"), "expected 'bar' in: {h}");
    }

    // ===================================================================
    // Partial format overlap
    // ===================================================================

    #[test]
    fn formatting_partially_overlapping_ranges() {
        let mut model = model_with_text("abcdef");
        model.select(0, 3); // "abc"
        model.bold();
        model.select(2, 5); // "cde"
        model.italic();
        let h = html(&model);
        // "c" should have both bold and italic
        assert!(h.contains("<strong>"), "expected <strong> in: {h}");
        assert!(h.contains("<em>"), "expected <em> in: {h}");
    }

    #[test]
    fn formatting_adjacent_ranges() {
        let mut model = model_with_text("abcdef");
        model.select(0, 3);
        model.bold();
        model.select(3, 6);
        model.italic();
        let h = html(&model);
        assert!(h.contains("<strong>"), "expected <strong> in: {h}");
        assert!(h.contains("<em>"), "expected <em> in: {h}");
        // Plain text is unchanged
        assert_eq!(plain(&model), "abcdef");
    }

    // ===================================================================
    // Format + text ops interactions
    // ===================================================================

    #[test]
    fn typing_into_bold_range_stays_bold() {
        let mut model = model_with_text("aabbcc");
        model.select(2, 4);
        model.bold();
        // Place cursor inside the bold range
        model.select(3, 3);
        model.replace_text("X");
        let p = plain(&model);
        assert_eq!(p, "aabXbcc");
        // The X should be bold (ExpandMark::Both means it grows)
        let h = html(&model);
        assert!(h.contains("bXb"), "expected 'bXb' in: {h}");
        assert!(h.contains("<strong>"), "expected <strong> in: {h}");
    }

    #[test]
    fn deleting_bold_text_preserves_remaining_bold() {
        let mut model = model_with_text("aabbcc");
        model.select(0, 6);
        model.bold();
        model.select(2, 4); // select "bb"
        model.delete();
        let h = html(&model);
        assert!(h.contains("<strong>"), "expected <strong> in: {h}");
        assert_eq!(plain(&model), "aacc");
    }
}
