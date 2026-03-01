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

//! Block-level operations: lists, code blocks, quotes, indent/unindent.
//!
//! Block markers are Map objects inserted into the Automerge text sequence
//! via `split_block`. Each marker has:
//!
//! | Key         | Type          | Description                        |
//! |-------------|---------------|------------------------------------|
//! | `"type"`    | `Str`         | Block type (see [`BlockType`])     |
//! | `"parents"` | `List`        | Nesting path for indentation       |
//! | `"attrs"`   | `Map`         | Extra attributes (currently empty) |
//!
//! These markers appear as `Span::Block(Map)` when iterating `doc.spans()`.

use automerge::iter::Span;
use automerge::transaction::Transactable;
use automerge::ReadDoc;

use super::AutomergeModel;
use crate::ComposerUpdate;

// ────────────────────────────────────────────────────────────────────────────
// Block type constants
// ────────────────────────────────────────────────────────────────────────────

/// String constants stored in block marker `"type"` field.
pub(crate) mod block_type {
    pub const PARAGRAPH: &str = "paragraph";
    pub const ORDERED_LIST_ITEM: &str = "ordered-list-item";
    pub const UNORDERED_LIST_ITEM: &str = "unordered-list-item";
    pub const CODE_BLOCK: &str = "code-block";
    pub const QUOTE: &str = "quote";
}

// ────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ────────────────────────────────────────────────────────────────────────────

/// Information about the block marker governing a given cursor position.
#[derive(Debug, Clone)]
pub(crate) struct BlockInfo {
    /// The index of the block marker in the text sequence.
    pub index: usize,
    /// The block type string.
    pub block_type: String,
    /// The indent level (number of parents).
    pub indent: usize,
}

impl AutomergeModel {
    // ────────────────────────────────────────────────────────────────────
    // Block introspection
    // ────────────────────────────────────────────────────────────────────

    /// Find the block marker that governs the text at `pos`.
    ///
    /// Scans spans left-to-right, tracking the last-seen block marker
    /// before the running offset passes `pos`.
    pub(crate) fn block_at(&self, pos: usize) -> Option<BlockInfo> {
        let spans = self.doc.spans(&self.text_id).ok()?;

        let mut offset: usize = 0;
        let mut last_block: Option<BlockInfo> = None;

        for span in spans {
            match span {
                Span::Block(ref map) => {
                    let btype = map
                        .get("type")
                        .and_then(|v| {
                            if let automerge::hydrate::Value::Scalar(
                                automerge::ScalarValue::Str(s),
                            ) = v
                            {
                                Some(s.to_string())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| block_type::PARAGRAPH.to_string());

                    let indent = map
                        .get("parents")
                        .map(|v| {
                            if let automerge::hydrate::Value::List(list) = v {
                                list.len()
                            } else {
                                0
                            }
                        })
                        .unwrap_or(0);

                    last_block = Some(BlockInfo {
                        index: offset,
                        block_type: btype,
                        indent,
                    });
                    // Block marker consumes 1 index position
                    offset += 1;
                }
                Span::Text { ref text, .. } => {
                    let text_len = text.encode_utf16().count();
                    if offset + text_len > pos {
                        // Cursor is within this text span — the current
                        // last_block is the governing block.
                        return last_block;
                    }
                    offset += text_len;
                }
            }
        }

        // Cursor is at or past the end — use the last block seen.
        last_block
    }

    /// Return the block type string at the current cursor, or `None` if
    /// there is no block marker (plain paragraph text without blocks).
    pub(crate) fn current_block_type(&self) -> Option<String> {
        self.block_at(self.sel_start()).map(|b| b.block_type)
    }

    /// Insert a new block marker at `index` with the given type and
    /// empty parents/attrs.
    pub(crate) fn insert_block_marker(
        &mut self,
        index: usize,
        btype: &str,
    ) {
        let block_id = self
            .doc
            .split_block(&self.text_id, index)
            .expect("split_block failed");

        self.doc
            .update_object(
                &block_id,
                &automerge::hydrate::Value::from(
                    std::collections::HashMap::from([
                        ("type", automerge::hydrate::Value::from(btype)),
                        (
                            "parents",
                            automerge::hydrate::Value::List(
                                automerge::hydrate::List::default(),
                            ),
                        ),
                        (
                            "attrs",
                            automerge::hydrate::Value::Map(
                                automerge::hydrate::Map::default(),
                            ),
                        ),
                    ]),
                ),
            )
            .expect("update_object failed");
    }

    /// Replace an existing block marker at `index` with one of a new type.
    fn replace_block_marker(
        &mut self,
        index: usize,
        btype: &str,
    ) {
        let block_id = self
            .doc
            .replace_block(&self.text_id, index)
            .expect("replace_block failed");

        self.doc
            .update_object(
                &block_id,
                &automerge::hydrate::Value::from(
                    std::collections::HashMap::from([
                        ("type", automerge::hydrate::Value::from(btype)),
                        (
                            "parents",
                            automerge::hydrate::Value::List(
                                automerge::hydrate::List::default(),
                            ),
                        ),
                        (
                            "attrs",
                            automerge::hydrate::Value::Map(
                                automerge::hydrate::Map::default(),
                            ),
                        ),
                    ]),
                ),
            )
            .expect("update_object failed");
    }

    /// Remove a block marker at `index`, merging the block into its
    /// predecessor (or removing block structure entirely).
    fn remove_block_marker(&mut self, index: usize) {
        self.doc
            .join_block(&self.text_id, index)
            .expect("join_block failed");
    }

    /// Toggle a block type: if the current block is already `target_type`,
    /// revert to paragraph; otherwise, set it to `target_type`.
    ///
    /// If there is no block marker yet, insert one at index 0.
    fn toggle_block(&mut self, target_type: &str) -> ComposerUpdate<String> {
        self.push_undo();

        if let Some(info) = self.block_at(self.sel_start()) {
            if info.block_type == target_type {
                // Already this type — revert to paragraph
                self.replace_block_marker(
                    info.index,
                    block_type::PARAGRAPH,
                );
            } else {
                // Different type — change to target
                self.replace_block_marker(info.index, target_type);
            }
        } else {
            // No block marker exists yet — insert one at position 0
            self.insert_block_marker(0, target_type);
            // The block marker shifts all text by 1
            self.selection_start += 1;
            self.selection_end += 1;
        }

        self.create_update_replace_all()
    }

    // ────────────────────────────────────────────────────────────────────
    // Public API
    // ────────────────────────────────────────────────────────────────────

    /// Toggle an ordered list at the current block.
    ///
    /// If the cursor is already in an ordered list item, reverts to a
    /// plain paragraph.  Otherwise, converts the current paragraph (or
    /// other block) to an ordered list item.
    pub fn ordered_list(&mut self) -> ComposerUpdate<String> {
        self.toggle_block(block_type::ORDERED_LIST_ITEM)
    }

    /// Toggle an unordered list at the current block.
    pub fn unordered_list(&mut self) -> ComposerUpdate<String> {
        self.toggle_block(block_type::UNORDERED_LIST_ITEM)
    }

    /// Toggle a code block around the current block.
    ///
    /// If the cursor is already in a code block, reverts to paragraph.
    /// Otherwise, converts the current block to a code block.
    pub fn code_block(&mut self) -> ComposerUpdate<String> {
        self.toggle_block(block_type::CODE_BLOCK)
    }

    /// Toggle a block quote around the current block.
    pub fn quote(&mut self) -> ComposerUpdate<String> {
        self.toggle_block(block_type::QUOTE)
    }

    /// Increase indentation (nest the current list item one level deeper).
    ///
    /// Only applies when the cursor is inside a list item.  Adds the
    /// current block type to the `"parents"` list, simulating nesting.
    pub fn indent(&mut self) -> ComposerUpdate<String> {
        let info = match self.block_at(self.sel_start()) {
            Some(i)
                if i.block_type == block_type::ORDERED_LIST_ITEM
                    || i.block_type == block_type::UNORDERED_LIST_ITEM =>
            {
                i
            }
            _ => return ComposerUpdate::keep(),
        };

        self.push_undo();

        // Build parents list with one more nesting level
        let new_indent = info.indent + 1;
        let parents_values: Vec<automerge::hydrate::Value> = (0..new_indent)
            .map(|_| automerge::hydrate::Value::from(info.block_type.as_str()))
            .collect();

        let block_id = self
            .doc
            .replace_block(&self.text_id, info.index)
            .expect("replace_block failed");

        self.doc
            .update_object(
                &block_id,
                &automerge::hydrate::Value::from(
                    std::collections::HashMap::from([
                        (
                            "type",
                            automerge::hydrate::Value::from(
                                info.block_type.as_str(),
                            ),
                        ),
                        (
                            "parents",
                            automerge::hydrate::Value::List(
                                automerge::hydrate::List::from(parents_values),
                            ),
                        ),
                        (
                            "attrs",
                            automerge::hydrate::Value::Map(
                                automerge::hydrate::Map::default(),
                            ),
                        ),
                    ]),
                ),
            )
            .expect("update_object failed");

        self.create_update_replace_all()
    }

    /// Decrease indentation (un-nest one level).
    ///
    /// If already at the top level, this is a no-op (returns Keep).
    pub fn unindent(&mut self) -> ComposerUpdate<String> {
        let info = match self.block_at(self.sel_start()) {
            Some(i)
                if i.indent > 0
                    && (i.block_type == block_type::ORDERED_LIST_ITEM
                        || i.block_type
                            == block_type::UNORDERED_LIST_ITEM) =>
            {
                i
            }
            _ => return ComposerUpdate::keep(),
        };

        self.push_undo();

        let new_indent = info.indent - 1;
        let parents_values: Vec<automerge::hydrate::Value> = (0..new_indent)
            .map(|_| automerge::hydrate::Value::from(info.block_type.as_str()))
            .collect();

        let block_id = self
            .doc
            .replace_block(&self.text_id, info.index)
            .expect("replace_block failed");

        self.doc
            .update_object(
                &block_id,
                &automerge::hydrate::Value::from(
                    std::collections::HashMap::from([
                        (
                            "type",
                            automerge::hydrate::Value::from(
                                info.block_type.as_str(),
                            ),
                        ),
                        (
                            "parents",
                            automerge::hydrate::Value::List(
                                automerge::hydrate::List::from(parents_values),
                            ),
                        ),
                        (
                            "attrs",
                            automerge::hydrate::Value::Map(
                                automerge::hydrate::Map::default(),
                            ),
                        ),
                    ]),
                ),
            )
            .expect("update_object failed");

        self.create_update_replace_all()
    }
}

#[cfg(test)]
mod tests {
    use super::block_type;
    use crate::AutomergeModel;

    fn new_model() -> AutomergeModel {
        AutomergeModel::new()
    }

    fn model_with_text(text: &str) -> AutomergeModel {
        let mut m = AutomergeModel::new();
        m.replace_text(text);
        m
    }

    fn html(m: &AutomergeModel) -> String {
        m.get_content_as_html()
    }

    fn plain(m: &AutomergeModel) -> String {
        m.get_content_as_plain_text()
    }

    // ===================================================================
    // Ordered list
    // ===================================================================

    #[test]
    fn ordered_list_on_empty_model() {
        let mut model = new_model();
        model.ordered_list();
        let bt = model.current_block_type();
        assert_eq!(bt.as_deref(), Some(block_type::ORDERED_LIST_ITEM));
    }

    #[test]
    fn ordered_list_on_existing_text() {
        let mut model = model_with_text("hello");
        model.select(0, 5);
        model.ordered_list();
        assert_eq!(
            model.current_block_type().as_deref(),
            Some(block_type::ORDERED_LIST_ITEM),
        );
        assert!(plain(&model).contains("hello"));
    }

    #[test]
    fn toggle_ordered_list_off() {
        let mut model = model_with_text("hello");
        model.ordered_list();
        // Now toggle it off
        model.ordered_list();
        assert_eq!(
            model.current_block_type().as_deref(),
            Some(block_type::PARAGRAPH),
        );
    }

    #[test]
    fn ordered_list_html_output() {
        let mut model = model_with_text("item 1");
        model.ordered_list();
        let h = html(&model);
        assert!(h.contains("<ol>"), "expected <ol> in: {h}");
        assert!(h.contains("<li>"), "expected <li> in: {h}");
        assert!(h.contains("item 1"), "expected text in: {h}");
    }

    // ===================================================================
    // Unordered list
    // ===================================================================

    #[test]
    fn unordered_list_on_empty_model() {
        let mut model = new_model();
        model.unordered_list();
        assert_eq!(
            model.current_block_type().as_deref(),
            Some(block_type::UNORDERED_LIST_ITEM),
        );
    }

    #[test]
    fn toggle_unordered_list_off() {
        let mut model = model_with_text("hello");
        model.unordered_list();
        model.unordered_list();
        assert_eq!(
            model.current_block_type().as_deref(),
            Some(block_type::PARAGRAPH),
        );
    }

    #[test]
    fn unordered_list_html_output() {
        let mut model = model_with_text("bullet");
        model.unordered_list();
        let h = html(&model);
        assert!(h.contains("<ul>"), "expected <ul> in: {h}");
        assert!(h.contains("<li>"), "expected <li> in: {h}");
        assert!(h.contains("bullet"), "expected text in: {h}");
    }

    // ===================================================================
    // Code block
    // ===================================================================

    #[test]
    fn code_block_on_empty_model() {
        let mut model = new_model();
        model.code_block();
        assert_eq!(
            model.current_block_type().as_deref(),
            Some(block_type::CODE_BLOCK),
        );
    }

    #[test]
    fn code_block_on_text() {
        let mut model = model_with_text("fn main() {}");
        model.code_block();
        assert_eq!(
            model.current_block_type().as_deref(),
            Some(block_type::CODE_BLOCK),
        );
        assert!(plain(&model).contains("fn main()"));
    }

    #[test]
    fn toggle_code_block_off() {
        let mut model = model_with_text("code");
        model.code_block();
        model.code_block();
        assert_eq!(
            model.current_block_type().as_deref(),
            Some(block_type::PARAGRAPH),
        );
    }

    #[test]
    fn code_block_html_output() {
        let mut model = model_with_text("let x = 1;");
        model.code_block();
        let h = html(&model);
        assert!(
            h.contains("<pre><code>"),
            "expected <pre><code> in: {h}"
        );
        assert!(h.contains("let x = 1;"), "expected text in: {h}");
        assert!(
            h.contains("</code></pre>"),
            "expected </code></pre> in: {h}"
        );
    }

    // ===================================================================
    // Quote
    // ===================================================================

    #[test]
    fn quote_on_empty_model() {
        let mut model = new_model();
        model.quote();
        assert_eq!(
            model.current_block_type().as_deref(),
            Some(block_type::QUOTE),
        );
    }

    #[test]
    fn quote_on_text() {
        let mut model = model_with_text("wise words");
        model.quote();
        assert_eq!(
            model.current_block_type().as_deref(),
            Some(block_type::QUOTE),
        );
    }

    #[test]
    fn toggle_quote_off() {
        let mut model = model_with_text("wise words");
        model.quote();
        model.quote();
        assert_eq!(
            model.current_block_type().as_deref(),
            Some(block_type::PARAGRAPH),
        );
    }

    #[test]
    fn quote_html_output() {
        let mut model = model_with_text("A quote");
        model.quote();
        let h = html(&model);
        assert!(
            h.contains("<blockquote>"),
            "expected <blockquote> in: {h}"
        );
        assert!(h.contains("A quote"), "expected text in: {h}");
    }

    // ===================================================================
    // Switching between block types
    // ===================================================================

    #[test]
    fn switch_from_ordered_to_unordered() {
        let mut model = model_with_text("item");
        model.ordered_list();
        assert_eq!(
            model.current_block_type().as_deref(),
            Some(block_type::ORDERED_LIST_ITEM),
        );
        model.unordered_list();
        assert_eq!(
            model.current_block_type().as_deref(),
            Some(block_type::UNORDERED_LIST_ITEM),
        );
    }

    #[test]
    fn switch_from_quote_to_code_block() {
        let mut model = model_with_text("text");
        model.quote();
        model.code_block();
        assert_eq!(
            model.current_block_type().as_deref(),
            Some(block_type::CODE_BLOCK),
        );
    }

    #[test]
    fn switch_from_code_block_to_quote() {
        let mut model = model_with_text("text");
        model.code_block();
        model.quote();
        assert_eq!(
            model.current_block_type().as_deref(),
            Some(block_type::QUOTE),
        );
    }

    // ===================================================================
    // Indent / Unindent
    // ===================================================================

    #[test]
    fn indent_on_non_list_is_noop() {
        let mut model = model_with_text("hello");
        let update = model.indent();
        // Should return Keep (no change)
        assert!(
            format!("{:?}", update).contains("Keep"),
            "expected Keep for non-list indent"
        );
    }

    #[test]
    fn indent_list_item() {
        let mut model = model_with_text("nested");
        model.unordered_list();
        model.indent();
        let info = model.block_at(model.sel_start()).unwrap();
        assert_eq!(info.indent, 1);
        assert_eq!(info.block_type, block_type::UNORDERED_LIST_ITEM);
    }

    #[test]
    fn indent_twice() {
        let mut model = model_with_text("deep");
        model.ordered_list();
        model.indent();
        model.indent();
        let info = model.block_at(model.sel_start()).unwrap();
        assert_eq!(info.indent, 2);
    }

    #[test]
    fn unindent_on_non_list_is_noop() {
        let mut model = model_with_text("hello");
        let update = model.unindent();
        assert!(
            format!("{:?}", update).contains("Keep"),
            "expected Keep for non-list unindent"
        );
    }

    #[test]
    fn unindent_at_top_level_is_noop() {
        let mut model = model_with_text("item");
        model.unordered_list();
        let update = model.unindent();
        assert!(
            format!("{:?}", update).contains("Keep"),
            "expected Keep for unindent at top level"
        );
    }

    #[test]
    fn indent_then_unindent_returns_to_zero() {
        let mut model = model_with_text("item");
        model.ordered_list();
        model.indent();
        assert_eq!(model.block_at(model.sel_start()).unwrap().indent, 1);
        model.unindent();
        assert_eq!(model.block_at(model.sel_start()).unwrap().indent, 0);
    }

    // ===================================================================
    // Block + formatting interaction
    // ===================================================================

    #[test]
    fn bold_text_in_code_block() {
        let mut model = model_with_text("abc");
        model.code_block();
        model.select(1, 3); // select inside code block (offset+1 for marker)
        model.bold();
        let h = html(&model);
        assert!(h.contains("<pre><code>"), "expected code block in: {h}");
        assert!(h.contains("<strong>"), "expected bold in: {h}");
    }

    #[test]
    fn block_type_preserved_after_typing() {
        let mut model = new_model();
        model.ordered_list();
        model.replace_text("hello");
        assert_eq!(
            model.current_block_type().as_deref(),
            Some(block_type::ORDERED_LIST_ITEM),
        );
    }

    // ===================================================================
    // Undo integration
    // ===================================================================

    #[test]
    fn undo_reverts_block_toggle() {
        let mut model = model_with_text("text");
        model.code_block();
        assert_eq!(
            model.current_block_type().as_deref(),
            Some(block_type::CODE_BLOCK),
        );
        model.undo();
        // After undo, there should be no code-block marker
        let bt = model.current_block_type();
        assert_ne!(bt.as_deref(), Some(block_type::CODE_BLOCK));
    }
}
