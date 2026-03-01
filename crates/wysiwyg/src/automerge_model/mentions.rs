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

//! Mention operations: insert user/room mentions and @room.
//!
//! Mentions are represented as text with a `"mention"` mark
//! (ExpandMark::None) whose value is the Matrix URI.

use std::collections::HashSet;

use automerge::marks::{ExpandMark, Mark};
use automerge::transaction::Transactable;
use automerge::ReadDoc;

use super::AutomergeModel;
use crate::composer_model_interface::Attribute;
use crate::{ComposerUpdate, MentionsState, SuggestionPattern};

impl AutomergeModel {
    /// Insert a user/room mention at the current cursor position.
    pub fn insert_mention(
        &mut self,
        url: &str,
        text: &str,
        _attributes: &[Attribute],
    ) -> ComposerUpdate<String> {
        self.push_undo();

        let start = self.sel_start();
        let end = self.sel_end();

        // Delete selection if any
        if start != end {
            let del = (end - start) as isize;
            self.doc
                .splice_text(&self.text_id, start, del, "")
                .expect("splice_text failed");
        }

        // Insert the display text
        self.doc
            .splice_text(&self.text_id, start, 0, text)
            .expect("splice_text failed");

        let text_end = start + text.encode_utf16().count();

        // Apply the mention mark with the Matrix URI as value
        let mark =
            Mark::new("mention".to_string(), url, start, text_end);
        let _ = self.doc.mark(&self.text_id, mark, ExpandMark::None);

        // Move cursor after the mention and add a trailing NBSP
        self.doc
            .splice_text(&self.text_id, text_end, 0, "\u{00A0}")
            .expect("splice_text failed");
        self.selection_start = text_end + 1;
        self.selection_end = text_end + 1;

        self.create_update_replace_all()
    }

    /// Insert a mention replacing a suggestion pattern match.
    pub fn insert_mention_at_suggestion(
        &mut self,
        url: &str,
        text: &str,
        suggestion: &SuggestionPattern,
        _attributes: &[Attribute],
    ) -> ComposerUpdate<String> {
        self.push_undo();

        let start = suggestion.start;
        let end = suggestion.end;
        let del = (end - start) as isize;

        // Replace the suggestion text
        self.doc
            .splice_text(&self.text_id, start, del, text)
            .expect("splice_text failed");

        let text_end = start + text.encode_utf16().count();

        // Apply the mention mark
        let mark =
            Mark::new("mention".to_string(), url, start, text_end);
        let _ = self.doc.mark(&self.text_id, mark, ExpandMark::None);

        // Trailing NBSP
        self.doc
            .splice_text(&self.text_id, text_end, 0, "\u{00A0}")
            .expect("splice_text failed");
        self.selection_start = text_end + 1;
        self.selection_end = text_end + 1;

        self.create_update_replace_all()
    }

    /// Insert an @room mention at the current cursor position.
    pub fn insert_at_room_mention(
        &mut self,
        _attributes: &[Attribute],
    ) -> ComposerUpdate<String> {
        self.insert_mention(
            "https://matrix.to/#/#room:matrix.org",
            "@room",
            &[],
        )
    }

    /// Insert an @room mention replacing a suggestion pattern match.
    pub fn insert_at_room_mention_at_suggestion(
        &mut self,
        suggestion: &SuggestionPattern,
        _attributes: &[Attribute],
    ) -> ComposerUpdate<String> {
        self.insert_mention_at_suggestion(
            "https://matrix.to/#/#room:matrix.org",
            "@room",
            suggestion,
            &[],
        )
    }

    /// Compute the current mentions state by iterating all spans
    /// and collecting mention mark values.
    pub fn get_mentions_state(&self) -> MentionsState {
        let mut state = MentionsState::default();

        // Iterate through spans to find mention marks
        if let Ok(spans) = self.doc.spans(&self.text_id) {
            for span in spans {
                if let automerge::iter::Span::Text { marks, .. } = &span {
                    if let Some(mark_set) = marks {
                        if let Some(value) =
                            Self::mark_value_in_set(mark_set, "mention")
                        {
                            if let Some(uri) = value.to_str() {
                                Self::add_mention_to_state(
                                    &mut state, uri,
                                );
                            }
                        }
                    }
                }
            }
        }

        state
    }

    /// Parse a Matrix URI and add it to the appropriate set in MentionsState.
    fn add_mention_to_state(state: &mut MentionsState, uri: &str) {
        if uri.contains("@room") || uri.contains("#room:") {
            state.has_at_room_mention = true;
        } else if uri.contains('@') {
            // User mention (e.g., https://matrix.to/#/@user:server.com)
            state.user_ids.insert(uri.to_string());
        } else if uri.contains('#') {
            // Room mention
            // Check for alias vs ID
            if uri.contains('!') {
                state.room_ids.insert(uri.to_string());
            } else {
                state.room_aliases.insert(uri.to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{AutomergeModel, MentionsState};

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
    // Insert mention (mapping test_mentions.rs)
    // ===================================================================

    #[test]
    fn insert_mention_at_cursor_inserts_text() {
        let mut model = new_model();
        model.insert_mention(
            "https://matrix.to/#/@alice:matrix.org",
            "Alice",
            &[],
        );
        let p = plain(&model);
        assert!(p.contains("Alice"), "expected 'Alice' in: {p}");
    }

    #[test]
    fn insert_mention_adds_trailing_nbsp() {
        let mut model = new_model();
        model.insert_mention(
            "https://matrix.to/#/@alice:matrix.org",
            "Alice",
            &[],
        );
        let p = plain(&model);
        assert!(
            p.contains('\u{00A0}'),
            "expected trailing NBSP in: {p:?}"
        );
    }

    #[test]
    fn insert_mention_creates_mention_mark_in_html() {
        let mut model = new_model();
        model.insert_mention(
            "https://matrix.to/#/@alice:matrix.org",
            "Alice",
            &[],
        );
        let h = html(&model);
        assert!(h.contains("Alice"), "expected 'Alice' in html: {h}");
        assert!(
            h.contains("href"),
            "expected mention link in html: {h}"
        );
    }

    #[test]
    fn insert_mention_at_start_of_text() {
        let mut model = model_with_text(" says hello");
        model.select(0, 0);
        model.insert_mention(
            "https://matrix.to/#/@alice:matrix.org",
            "Alice",
            &[],
        );
        let p = plain(&model);
        assert!(
            p.starts_with("Alice"),
            "expected 'Alice' at start: {p}"
        );
    }

    #[test]
    fn insert_mention_in_middle_of_text() {
        let mut model = model_with_text("Like  said");
        model.select(5, 5); // between the spaces
        model.insert_mention(
            "https://matrix.to/#/@alice:matrix.org",
            "Alice",
            &[],
        );
        let p = plain(&model);
        assert!(p.contains("Alice"), "expected 'Alice' in: {p}");
    }

    #[test]
    fn insert_mention_at_end_of_text() {
        let mut model = model_with_text("hello ");
        model.insert_mention(
            "https://matrix.to/#/@alice:matrix.org",
            "Alice",
            &[],
        );
        let p = plain(&model);
        assert!(
            p.contains("hello"),
            "expected 'hello' preserved: {p}"
        );
        assert!(p.contains("Alice"), "expected 'Alice' at end: {p}");
    }

    // ===================================================================
    // Insert at-room mention
    // ===================================================================

    #[test]
    fn insert_at_room_mention() {
        let mut model = new_model();
        model.insert_at_room_mention(&[]);
        let p = plain(&model);
        assert!(p.contains("@room"), "expected '@room' in: {p}");
    }

    #[test]
    fn insert_at_room_mention_creates_html() {
        let mut model = new_model();
        model.insert_at_room_mention(&[]);
        let h = html(&model);
        assert!(h.contains("@room"), "expected '@room' in html: {h}");
    }

    // ===================================================================
    // Insert mention replacing selection
    // ===================================================================

    #[test]
    fn insert_mention_replaces_selection() {
        let mut model = model_with_text("replace_me");
        model.select(0, 10);
        model.insert_mention(
            "https://matrix.to/#/@alice:matrix.org",
            "Alice",
            &[],
        );
        let p = plain(&model);
        assert!(!p.contains("replace_me"), "old text should be gone");
        assert!(p.contains("Alice"), "expected 'Alice' in: {p}");
    }

    #[test]
    fn insert_mention_partial_selection() {
        let mut model = model_with_text("hello replace_me world");
        model.select(6, 16); // "replace_me"
        model.insert_mention(
            "https://matrix.to/#/@alice:matrix.org",
            "Alice",
            &[],
        );
        let p = plain(&model);
        assert!(p.contains("hello"), "expected 'hello' in: {p}");
        assert!(p.contains("Alice"), "expected 'Alice' in: {p}");
        assert!(p.contains("world"), "expected 'world' in: {p}");
    }

    // ===================================================================
    // Insert mention at suggestion
    // ===================================================================

    #[test]
    fn insert_mention_at_suggestion() {
        use crate::{PatternKey, SuggestionPattern};

        let mut model = model_with_text("hello @ali world");
        let suggestion = SuggestionPattern {
            key: PatternKey::At,
            text: "@ali".to_string(),
            start: 6,
            end: 10,
        };
        model.insert_mention_at_suggestion(
            "https://matrix.to/#/@alice:matrix.org",
            "Alice",
            &suggestion,
            &[],
        );
        let p = plain(&model);
        assert!(p.contains("Alice"), "expected 'Alice' in: {p}");
        assert!(!p.contains("@ali"), "expected '@ali' replaced in: {p}");
    }

    // ===================================================================
    // Get mentions state (mapping test_mentions.rs)
    // ===================================================================

    #[test]
    fn get_mentions_state_for_no_mentions() {
        let model = model_with_text("hello!");
        assert_eq!(model.get_mentions_state(), MentionsState::default());
    }

    #[test]
    fn get_mentions_state_for_user_mention() {
        let mut model = new_model();
        model.insert_mention(
            "https://matrix.to/#/@alice:matrix.org",
            "Alice",
            &[],
        );
        let state = model.get_mentions_state();
        assert!(
            state
                .user_ids
                .contains("https://matrix.to/#/@alice:matrix.org"),
            "expected user mention: {:?}",
            state.user_ids
        );
    }

    #[test]
    fn get_mentions_state_for_multiple_user_mentions() {
        let mut model = new_model();
        model.insert_mention(
            "https://matrix.to/#/@alice:matrix.org",
            "Alice",
            &[],
        );
        model.insert_mention(
            "https://matrix.to/#/@bob:matrix.org",
            "Bob",
            &[],
        );
        let state = model.get_mentions_state();
        assert!(state.user_ids.len() >= 2, "expected 2+ users: {:?}", state);
    }

    #[test]
    fn get_mentions_state_for_at_room_mention() {
        let mut model = new_model();
        model.insert_at_room_mention(&[]);
        let state = model.get_mentions_state();
        assert!(
            state.has_at_room_mention,
            "expected at-room mention in state"
        );
    }

    #[test]
    fn get_mentions_state_empty_for_plain_text() {
        let model = model_with_text("just some plain text");
        let state = model.get_mentions_state();
        assert_eq!(state, MentionsState::default());
    }
}
