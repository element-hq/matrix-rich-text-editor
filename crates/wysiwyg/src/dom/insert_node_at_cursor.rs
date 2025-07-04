// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::{DomHandle, DomNode, UnicodeString};

use super::{Dom, DomLocation, Range};

impl<S> Dom<S>
where
    S: UnicodeString,
{
    // Inserts the new node at the current cursor position if possible, panics if
    // the range passed is a selection
    pub fn insert_node_at_cursor(
        &mut self,
        range: &Range,
        new_node: DomNode<S>,
    ) -> DomHandle {
        if range.is_selection() {
            panic!("Attempted to use `insert_node_at_cursor` with a selection")
        }

        #[cfg(any(test, feature = "assert-invariants"))]
        self.assert_invariants();

        let inserted_handle: DomHandle;

        // manipulate the state of the dom as required
        if let Some(leaf) = range.leaves().next() {
            // when we have a leaf, the way we treat the insertion depends on the cursor position inside that leaf
            let cursor_at_end = leaf.start_offset == leaf.length;
            let cursor_at_start = leaf.start_offset == 0;
            let leaf_is_placeholder =
                self.lookup_node(&leaf.node_handle).is_placeholder();

            if leaf_is_placeholder || cursor_at_start {
                // insert the new node before a placeholder leaf or one that contains a cursor at the start
                inserted_handle = self.insert_at(&leaf.node_handle, new_node);
            } else if cursor_at_end {
                // insert the new node after a leaf that contains a cursor at the end
                inserted_handle = self
                    .append(&self.parent(&leaf.node_handle).handle(), new_node);
            } else {
                // otherwise insert the new node in the middle of a text node
                inserted_handle = self.insert_into_text(
                    &leaf.node_handle,
                    leaf.start_offset,
                    new_node,
                )
            }
        } else {
            // if we don't have a leaf, try to find the first container that we're inside
            let first_location: Option<&DomLocation> =
                range.locations.iter().find(|l| l.start_offset < l.length);
            match first_location {
                // if we haven't found anything, we're inserting into an empty dom
                None => {
                    inserted_handle = self.append_at_end_of_document(new_node);
                }
                Some(container) => {
                    inserted_handle =
                        self.append(&container.node_handle, new_node);
                }
            };
        }

        #[cfg(any(test, feature = "assert-invariants"))]
        self.assert_invariants();

        inserted_handle
    }
}

#[cfg(test)]
mod test {
    use crate::{
        tests::{testutils_composer_model::cm, testutils_conversion::utf16},
        DomNode, ToHtml,
    };
    #[test]
    #[should_panic]
    fn panics_if_passed_selection() {
        let mut model = cm("{something}|");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_node_at_cursor(
            &range,
            DomNode::new_link(utf16("href"), vec![], vec![]),
        );
    }

    #[test]
    fn inserts_node_in_empty_model() {
        let mut model = cm("|");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_node_at_cursor(
            &range,
            DomNode::new_link(utf16("href"), vec![], vec![]),
        );

        assert_eq!(model.state.dom.to_html(), "<a href=\"href\"></a>")
    }

    #[test]
    fn inserts_node_into_empty_container() {
        let mut model = cm("<p>|</p>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_node_at_cursor(
            &range,
            DomNode::new_link(utf16("href"), vec![], vec![]),
        );

        assert_eq!(model.state.dom.to_html(), "<p><a href=\"href\"></a></p>")
    }

    #[test]
    fn inserts_node_into_leaf_start() {
        let mut model = cm("<p>|this is a leaf</p>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_node_at_cursor(
            &range,
            DomNode::new_link(utf16("href"), vec![], vec![]),
        );

        assert_eq!(
            model.state.dom.to_html(),
            "<p><a href=\"href\"></a>this is a leaf</p>"
        )
    }

    #[test]
    fn inserts_node_into_leaf_middle() {
        let mut model = cm("<p>this is| a leaf</p>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_node_at_cursor(
            &range,
            DomNode::new_link(utf16("href"), vec![], vec![]),
        );

        assert_eq!(
            model.state.dom.to_html(),
            "<p>this is<a href=\"href\"></a> a leaf</p>"
        )
    }

    #[test]
    fn inserts_node_into_leaf_end() {
        let mut model = cm("<p>this is a leaf|</p>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_node_at_cursor(
            &range,
            DomNode::new_link(utf16("href"), vec![], vec![]),
        );

        assert_eq!(
            model.state.dom.to_html(),
            "<p>this is a leaf<a href=\"href\"></a></p>"
        )
    }

    #[test]
    fn inserts_node_into_empty_paragraph() {
        let mut model = cm("<p>&nbsp;</p><p>&nbsp;|</p><p>&nbsp;</p>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_node_at_cursor(
            &range,
            DomNode::new_link(utf16("href"), vec![], vec![]),
        );

        assert_eq!(
            model.state.dom.to_html(),
            "<p>\u{a0}</p><p><a href=\"href\"></a>\u{a0}</p><p>\u{a0}</p>"
        )
    }
}
