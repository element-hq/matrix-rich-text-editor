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

use std::collections::{HashMap, HashSet};

use automerge::marks::{ExpandMark, Mark, MarkSet};
use automerge::transaction::Transactable;
use automerge::{AutoCommit, ChangeHash, ObjType, ReadDoc, ScalarValue};

use crate::{
    ActionState, ComposerAction, ComposerUpdate, InlineFormatType,
    LinkAction, LinkActionUpdate, Location, MenuAction, MenuState,
    MenuStateUpdate,
};

/// An Automerge-backed rich text composer model.
///
/// The document contains a single text object at `doc["content"]` which
/// stores the full rich-text content as a CRDT text sequence with
/// Peritext marks (for inline formatting) and inline block markers
/// (for paragraphs, lists, code blocks, etc.).
///
/// ## Mark names
///
/// | Format         | Mark name          | Expand | Value       |
/// |----------------|--------------------|--------|-------------|
/// | Bold           | `"bold"`           | Both   | `true`      |
/// | Italic         | `"italic"`         | Both   | `true`      |
/// | Underline      | `"underline"`      | Both   | `true`      |
/// | Strikethrough  | `"strikethrough"`  | Both   | `true`      |
/// | Inline code    | `"inline_code"`    | Both   | `true`      |
/// | Link           | `"link"`           | None   | URL string  |
/// | Mention        | `"mention"`        | None   | URI string  |
pub struct AutomergeModel {
    /// The Automerge document (auto-commit mode).
    pub(crate) doc: AutoCommit,

    /// Object ID of the text content field (`doc["content"]`).
    pub(crate) text_id: automerge::ObjId,

    /// Current selection start (UTF-16 code unit offset).
    pub(crate) selection_start: usize,

    /// Current selection end (UTF-16 code unit offset).
    pub(crate) selection_end: usize,

    /// Formats toggled while the cursor is collapsed (applied on next insert).
    pub(crate) pending_formats: HashSet<String>,

    /// Undo stack: (heads snapshot, selection_start, selection_end) tuples.
    pub(crate) undo_stack: Vec<(Vec<ChangeHash>, usize, usize)>,

    /// Redo stack: (heads snapshot, selection_start, selection_end) tuples.
    pub(crate) redo_stack: Vec<(Vec<ChangeHash>, usize, usize)>,

    /// Custom suggestion trigger patterns (beyond @, #, /).
    pub custom_suggestion_patterns: HashSet<String>,
}

impl AutomergeModel {
    /// Create a new empty model.
    pub fn new() -> Self {
        let mut doc = AutoCommit::new();
        let text_id = doc
            .put_object(automerge::ROOT, "content", ObjType::Text)
            .expect("Failed to create text object");

        Self {
            doc,
            text_id,
            selection_start: 0,
            selection_end: 0,
            pending_formats: HashSet::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            custom_suggestion_patterns: HashSet::new(),
        }
    }

    /// Create a model pre-populated with HTML content.
    pub fn from_html(html: &str) -> Self {
        let mut model = Self::new();
        model.set_content_from_html_internal(html);
        model
    }

    /// The document length in UTF-16 code units.
    pub fn text_len(&self) -> usize {
        self.doc.length(&self.text_id)
    }

    /// Whether the selection is a range (start != end).
    pub fn has_selection(&self) -> bool {
        self.selection_start != self.selection_end
    }

    /// Selection start, ensuring start <= end.
    pub(crate) fn sel_start(&self) -> usize {
        self.selection_start.min(self.selection_end)
    }

    /// Selection end, ensuring start <= end.
    pub(crate) fn sel_end(&self) -> usize {
        self.selection_start.max(self.selection_end)
    }

    /// Push current state to the undo stack and clear the redo stack.
    pub(crate) fn push_undo(&mut self) {
        let heads = self.doc.get_heads();
        self.undo_stack
            .push((heads, self.selection_start, self.selection_end));
        self.redo_stack.clear();
    }

    /// Look up a mark by name from a [`MarkSet`].
    /// Returns the value if found and not null.
    pub(crate) fn mark_value_in_set<'a>(
        mark_set: &'a MarkSet,
        name: &str,
    ) -> Option<&'a ScalarValue> {
        for (k, v) in mark_set.iter() {
            if k == name && !matches!(v, ScalarValue::Null) {
                return Some(v);
            }
        }
        None
    }

    /// Check whether a mark is active at a given position.
    pub(crate) fn is_mark_active_at(
        &self,
        pos: usize,
        mark_name: &str,
    ) -> bool {
        if let Ok(mark_set) =
            self.doc.get_marks(&self.text_id, pos, None)
        {
            Self::mark_value_in_set(&mark_set, mark_name).is_some()
        } else {
            false
        }
    }

    /// Compute the current [`MenuState`] from active marks and block context.
    pub(crate) fn compute_menu_state(&self) -> MenuState {
        let states = self.compute_action_states();
        MenuState::Update(MenuStateUpdate {
            action_states: states,
        })
    }

    /// Compute action states for all toolbar actions.
    pub(crate) fn compute_action_states(
        &self,
    ) -> HashMap<ComposerAction, ActionState> {
        let mut states = HashMap::new();
        let pos = self.sel_start();

        // Query marks at the cursor position (empty set if out of range)
        let mark_set = self
            .doc
            .get_marks(&self.text_id, pos, None)
            .unwrap_or_default();

        // Inline formatting: Reversed if active at cursor, Enabled otherwise
        let inline_marks = [
            (ComposerAction::Bold, "bold"),
            (ComposerAction::Italic, "italic"),
            (ComposerAction::StrikeThrough, "strikethrough"),
            (ComposerAction::Underline, "underline"),
            (ComposerAction::InlineCode, "inline_code"),
        ];

        for (action, mark_name) in &inline_marks {
            let is_active =
                Self::mark_value_in_set(&mark_set, mark_name).is_some()
                    || self.pending_formats.contains(*mark_name);
            states.insert(
                action.clone(),
                if is_active {
                    ActionState::Reversed
                } else {
                    ActionState::Enabled
                },
            );
        }

        // Link: always Enabled for now
        states.insert(ComposerAction::Link, ActionState::Enabled);

        // Block operations: Reversed if inside that block type
        let current_bt = self.current_block_type();
        let block_actions = [
            (
                ComposerAction::OrderedList,
                super::block_ops::block_type::ORDERED_LIST_ITEM,
            ),
            (
                ComposerAction::UnorderedList,
                super::block_ops::block_type::UNORDERED_LIST_ITEM,
            ),
            (
                ComposerAction::CodeBlock,
                super::block_ops::block_type::CODE_BLOCK,
            ),
            (
                ComposerAction::Quote,
                super::block_ops::block_type::QUOTE,
            ),
        ];
        for (action, btype) in &block_actions {
            let is_active = current_bt.as_deref() == Some(*btype);
            states.insert(
                action.clone(),
                if is_active {
                    ActionState::Reversed
                } else {
                    ActionState::Enabled
                },
            );
        }

        // Indent: Enabled only when inside a list item
        // Unindent: Enabled only when inside a list item with indent > 0
        let block_info = self.block_at(self.sel_start());
        let in_list = block_info.as_ref().map_or(false, |b| {
            b.block_type == super::block_ops::block_type::ORDERED_LIST_ITEM
                || b.block_type
                    == super::block_ops::block_type::UNORDERED_LIST_ITEM
        });
        states.insert(
            ComposerAction::Indent,
            if in_list {
                ActionState::Enabled
            } else {
                ActionState::Disabled
            },
        );
        states.insert(
            ComposerAction::Unindent,
            if in_list && block_info.as_ref().map_or(false, |b| b.indent > 0)
            {
                ActionState::Enabled
            } else {
                ActionState::Disabled
            },
        );

        // Undo/Redo: Enabled if respective stack is non-empty
        states.insert(
            ComposerAction::Undo,
            if self.undo_stack.is_empty() {
                ActionState::Disabled
            } else {
                ActionState::Enabled
            },
        );
        states.insert(
            ComposerAction::Redo,
            if self.redo_stack.is_empty() {
                ActionState::Disabled
            } else {
                ActionState::Enabled
            },
        );

        states
    }

    /// Build a full [`ComposerUpdate`] with ReplaceAll and current states.
    pub(crate) fn create_update_replace_all(
        &self,
    ) -> ComposerUpdate<String> {
        let html = self.spans_to_html();
        let menu_state = self.compute_menu_state();
        let menu_action = self.compute_menu_action();
        let link_action = self.compute_link_action_update();

        ComposerUpdate::replace_all(
            html,
            Location::from(self.selection_start),
            Location::from(self.selection_end),
            menu_state,
            menu_action,
            link_action,
        )
    }

    /// Build a selection-only [`ComposerUpdate`].
    pub(crate) fn create_update_selection(
        &self,
    ) -> ComposerUpdate<String> {
        let menu_state = self.compute_menu_state();
        let menu_action = self.compute_menu_action();
        let link_action = self.compute_link_action_update();

        ComposerUpdate::update_selection(
            Location::from(self.selection_start),
            Location::from(self.selection_end),
            menu_state,
            menu_action,
            link_action,
        )
    }

    /// Compute the current [`MenuAction`] (suggestion detection).
    pub(crate) fn compute_menu_action(&self) -> MenuAction {
        // TODO: Implement suggestion pattern detection
        // Scan backwards from cursor for @, #, / triggers
        MenuAction::None
    }

    /// Compute the current [`LinkActionUpdate`].
    pub(crate) fn compute_link_action_update(
        &self,
    ) -> LinkActionUpdate<String> {
        let link_action = self.compute_link_action();
        LinkActionUpdate::Update(link_action)
    }

    /// Compute the [`LinkAction`] at the current cursor position.
    pub(crate) fn compute_link_action(&self) -> LinkAction<String> {
        if !self.has_selection() {
            if let Ok(marks) =
                self.doc.get_marks(&self.text_id, self.sel_start(), None)
            {
                if let Some(url_value) =
                    Self::mark_value_in_set(&marks, "link")
                {
                    if let Some(url) = url_value.to_str() {
                        return LinkAction::Edit(url.to_string());
                    }
                }
                if Self::mark_value_in_set(&marks, "mention").is_some() {
                    return LinkAction::Disabled;
                }
            }
            LinkAction::CreateWithText
        } else {
            LinkAction::Create
        }
    }

    /// Apply any pending (toggled) formats as marks on [start, end).
    pub(crate) fn apply_pending_marks(
        &mut self,
        start: usize,
        end: usize,
    ) {
        let pending: Vec<String> = self.pending_formats.drain().collect();
        for mark_name in pending {
            let expand = Self::expand_for_mark(&mark_name);
            let mark = Mark::new(mark_name, true, start, end);
            let _ = self.doc.mark(&self.text_id, mark, expand);
        }
    }

    /// Return the appropriate [`ExpandMark`] strategy for a given mark name.
    pub(crate) fn expand_for_mark(mark_name: &str) -> ExpandMark {
        match mark_name {
            "link" | "mention" => ExpandMark::None,
            _ => ExpandMark::Both,
        }
    }

    /// Map an [`InlineFormatType`] to the Automerge mark name.
    pub(crate) fn mark_name_for_format(
        format: &InlineFormatType,
    ) -> &'static str {
        match format {
            InlineFormatType::Bold => "bold",
            InlineFormatType::Italic => "italic",
            InlineFormatType::StrikeThrough => "strikethrough",
            InlineFormatType::Underline => "underline",
            InlineFormatType::InlineCode => "inline_code",
        }
    }

    /// Set HTML content on the model (internal, no ComposerUpdate returned).
    pub(crate) fn set_content_from_html_internal(&mut self, _html: &str) {
        // TODO: Parse HTML → spans → update_spans
        // For now this is a stub
    }

    /// Return a debug tree representation of the document.
    pub fn to_tree(&self) -> String {
        use automerge::iter::Span;
        use automerge::ReadDoc;

        let mut out = String::new();
        let sel_start = self.selection_start;
        let sel_end = self.selection_end;

        out.push_str(&format!("sel: ({sel_start},{sel_end})\n"));

        let spans: Vec<Span> = match self.doc.spans(&self.text_id) {
            Ok(s) => s.collect(),
            Err(_) => {
                out.push_str("(empty)\n");
                return out;
            }
        };

        // Walk spans and build a readable tree.
        // Track a character offset so we can annotate the selection.
        let mut offset: usize = 0;

        for span in &spans {
            match span {
                Span::Block(block_map) => {
                    let btype = block_map
                        .get("type")
                        .and_then(|v| {
                            if let automerge::hydrate::Value::Scalar(
                                automerge::ScalarValue::Str(s),
                            ) = v
                            {
                                Some(s.as_str())
                            } else {
                                None
                            }
                        })
                        .unwrap_or("paragraph");
                    out.push_str(&format!("├─ block({btype})\n"));
                }
                Span::Text { text, marks } => {
                    // Collect mark names
                    let mark_tags: Vec<String> = marks
                        .as_ref()
                        .map(|ms| {
                            ms.iter()
                                .filter(|(_, v)| {
                                    !matches!(
                                        v,
                                        automerge::ScalarValue::Null
                                    )
                                })
                                .map(|(k, v)| {
                                    if let Some(s) = v.to_str() {
                                        format!("{k}=\"{s}\"")
                                    } else {
                                        k.to_string()
                                    }
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    let marks_str = if mark_tags.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", mark_tags.join(", "))
                    };

                    let text_len = text.len();

                    // Build the display text with selection markers
                    let display_text =
                        annotate_selection(text, offset, sel_start, sel_end);

                    out.push_str(&format!(
                        "│  \"{display_text}\"{marks_str}\n"
                    ));

                    offset += text_len;
                }
            }
        }

        // If cursor is at the very end, show it
        if sel_start == offset && sel_start == sel_end {
            out.push_str("│  |\n");
        }

        out
    }
}

/// Insert `|` (cursor) or `{…}` (range selection) markers into a text span.
///
/// `span_offset` is the character offset at which this span starts in the
/// document.  `sel_start`/`sel_end` are the document-level selection bounds.
fn annotate_selection(
    text: &str,
    span_offset: usize,
    sel_start: usize,
    sel_end: usize,
) -> String {
    let span_end = span_offset + text.len();

    // Selection doesn't overlap this span at all.
    if sel_end <= span_offset || sel_start >= span_end {
        return text.to_string();
    }

    let mut result = String::new();
    for (i, ch) in text.char_indices() {
        let doc_pos = span_offset + i;
        // Opening marker
        if doc_pos == sel_start {
            if sel_start == sel_end {
                result.push('|');
            } else {
                result.push('{');
            }
        }
        result.push(ch);
        // Closing marker — after the character
        let next_pos = doc_pos + ch.len_utf8();
        if next_pos == sel_end && sel_start != sel_end {
            result.push('}');
        }
    }
    // Cursor at very end of this span
    let end_doc_pos = span_offset + text.len();
    if sel_start == end_doc_pos && sel_start == sel_end {
        result.push('|');
    }

    result
}

impl Default for AutomergeModel {
    fn default() -> Self {
        Self::new()
    }
}
