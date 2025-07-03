// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::dom::nodes::{ContainerNodeKind, DomNode};
use crate::dom::parser::parse_from_external_html_source;
use crate::{parse, ComposerModel, ComposerUpdate, Location, UnicodeString};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    /// Replaces text in the current selection with new_html.
    /// Treats its input as html that is parsed into a DomNode and inserted into
    /// the document at the cursor.
    pub fn replace_html(
        &mut self,
        new_html: S,
        from_external_source: bool,
    ) -> ComposerUpdate<S> {
        self.push_state_to_history();
        if self.has_selection() {
            self.do_replace_text(S::default());
        }
        let result = if from_external_source {
            parse_from_external_html_source(&new_html.to_string())
        } else {
            parse(&new_html.to_string())
        };

        let dom = result.unwrap().into_document_node();

        let (start, end) = self.safe_selection();
        let range = self.state.dom.find_range(start, end);

        let new_cursor_index = start + dom.text_len();
        let handle = self.state.dom.insert_node_at_cursor(&range, dom);

        // manually move the cursor to the end of the html
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
