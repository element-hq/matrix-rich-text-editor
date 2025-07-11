// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::dom::nodes::{ContainerNodeKind, DomNode};
use crate::dom::parser::parse;
use crate::{ComposerModel, ComposerUpdate, Location, UnicodeString};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    /// Attempts to clear out all the cruft from pasted HTML
    /// to get the basic formatted content.
    fn get_root_content(&mut self, root: DomNode<S>) -> Option<DomNode<S>> {
        match root {
            DomNode::Container(c) => match c.kind() {
                ContainerNodeKind::Generic => match c.get_child(0) {
                    Some(child) => self.get_root_content(child.clone()),
                    None => None,
                },
                _ => Some(DomNode::Container(c)),
            },
            _ => Some(root),
        }
    }

    /// Replaces text in the current selection with new_text.
    /// Treats its input as plain text, so any HTML code will show up in
    /// the document (i.e. it will be escaped).
    pub fn replace_html(&mut self, new_html: S) -> ComposerUpdate<S> {
        self.push_state_to_history();
        if self.has_selection() {
            self.do_replace_text(S::default());
        }
        let new_dom =
            parse(&new_html.to_string()).unwrap().into_document_node();

        // Strip away any empty container nodes.
        let content_dom = self.get_root_content(new_dom).unwrap();

        let (start, end) = self.safe_selection();
        let range = self.state.dom.find_range(start, end);

        let new_cursor_index = start + content_dom.text_len();
        let handle = self.state.dom.insert_node_at_cursor(&range, content_dom);

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
