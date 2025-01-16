// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use std::collections::HashSet;

use crate::{
    dom::{
        unicode_string::{UnicodeStr, UnicodeStringExt},
        Range,
    },
    ComposerModel, MenuAction, PatternKey, SuggestionPattern, UnicodeString,
};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    /// Compute the menu action for current composer model state.
    pub(crate) fn compute_menu_action(&self) -> MenuAction {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);

        if range
            .locations
            .iter()
            .any(|l| l.kind.is_code_kind() || l.kind.is_link_kind())
        {
            return MenuAction::None;
        }
        let (raw_text, start, end) = self.extended_text(range);

        if let Some((key, text)) = Self::pattern_for_text(
            raw_text,
            start,
            &self.custom_suggestion_patterns,
        ) {
            MenuAction::Suggestion(SuggestionPattern {
                key,
                text,
                start,
                end,
            })
        } else {
            MenuAction::None
        }
    }

    /// Compute extended text from a range. Text is extended up
    /// to the leading/trailing of the text nodes, or up to the
    /// first whitespace found.
    /// Returns the extended text, and its start/end locations.
    fn extended_text(&self, range: Range) -> (S, usize, usize) {
        range
            .leaves()
            .filter_map(|loc| {
                self.state
                    .dom
                    .lookup_node(&loc.node_handle)
                    .as_text()
                    .map(|t| (t, loc.start_offset..loc.end_offset))
            })
            .fold(
                (S::default(), range.start(), range.end()),
                |(mut text, s, e), (t, range)| {
                    let (node_text, start_offset, end_offset) =
                        t.extended_text_for_range(range);
                    text.push(node_text);
                    (text, s - start_offset, e + end_offset)
                },
            )
    }

    /// Compute at/hash/slash pattern for a given text.
    /// Return pattern key and associated text, if it exists.
    fn pattern_for_text(
        mut text: S,
        start_location: usize,
        custom_suggestion_patterns: &HashSet<String>,
    ) -> Option<(PatternKey, String)> {
        let key = PatternKey::from_string_and_suggestions(
            text.to_string(),
            custom_suggestion_patterns,
        )?;

        if key.is_static_pattern() {
            text.pop_first();
        }

        // Exclude slash patterns that are not at the beginning of the document
        // and any selection that contains inner whitespaces.
        if (key == PatternKey::Slash && start_location > 0)
            || text.chars().any(|c| c.is_whitespace())
        {
            None
        } else {
            Some((key, text.to_string()))
        }
    }
}
