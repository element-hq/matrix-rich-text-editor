// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::dom::nodes::dom_node::DomNodeKind;
use crate::dom::nodes::DomNode;
use crate::dom::parser::parse;
use crate::dom::unicode_string::UnicodeStrExt;
use crate::dom::DomCreationError;
use crate::dom::{DomLocation, Range};
use crate::{
    ComposerModel, ComposerUpdate, DomHandle, Location, SuggestionPattern,
    UnicodeString,
};
use std::cmp::min;

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    /// Replaces text in the current selection with new_text.
    /// Treats its input as plain text, so any HTML code will show up in
    /// the document (i.e. it will be escaped).
    pub fn replace_html(&mut self, new_html: S) -> ComposerUpdate<S> {
        self.push_state_to_history();
        if self.has_selection() {
            self.do_replace_text(S::default());
        }
        let new_dom = parse(&new_html.to_string())
            .unwrap()
            .document_node()
            .clone();
        let (start, end) = self.safe_selection();
        let range = self.state.dom.find_range(start, end);

        let new_cursor_index = start + new_dom.text_len();

        let handle = self.state.dom.insert_node_at_cursor(&range, new_dom);

        // manually move the cursor to the end of the mention
        self.state.start = Location::from(new_cursor_index);
        self.state.end = self.state.start;

        // add a trailing space in cases when we do not have a next sibling
        if self.state.dom.is_last_in_parent(&handle) {
            self.do_replace_text(" ".into())
        } else {
            self.create_update_replace_all()
        }
    }
}
