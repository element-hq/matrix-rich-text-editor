// Copyright 2025 New Vector Ltd.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::{DomHandle, DomNode, UnicodeString};

use super::nodes::ContainerNode;
use super::{Dom, Range};

impl<S> Dom<S>
where
    S: UnicodeString,
{
    #[allow(dead_code, clippy::let_and_return)]
    /// Insert a node and make this node the parent of a given range.
    pub fn insert_parent(
        &mut self,
        range: &Range,
        new_node: DomNode<S>,
    ) -> DomHandle {
        #[cfg(any(test, feature = "assert-invariants"))]
        self.assert_invariants();

        if range.is_empty() {
            panic!("Cannot add a parent to an empty range");
        }

        // Nodes that the range only touches (it starts where they end, or
        // ends where they start) are not part of it.
        let range = Range::new(
            range
                .locations
                .iter()
                .filter(|l| l.start_offset != l.end_offset || l.length == 0),
        );

        // The new parent node goes inside the deepest node holding the whole
        // range, unless that node is completely covered.
        let shared_parent = range.shared_parent_outside();
        let depth = shared_parent.depth();
        let first_leaf = range.leaves().min().unwrap();
        let last_leaf = range.leaves().max().unwrap();

        // Split the shared parent at the end of the range and then at its
        // start, so that the range is made of whole children of it.
        let after = self.split_children_from(
            &last_leaf.node_handle,
            last_leaf.end_offset,
            depth,
        );
        let starts_with_parent = first_leaf.start_offset == 0
            && first_leaf.node_handle.raw()[depth..]
                .iter()
                .all(|i| *i == 0);
        let inside = if starts_with_parent {
            // Nothing to split (and splitting would remove the parent, as
            // it would be left empty)
            self.lookup_container_mut(&shared_parent).remove_children()
        } else {
            self.split_children_from(
                &first_leaf.node_handle,
                first_leaf.start_offset,
                depth,
            )
        };

        // Move the range into the new node, and insert it between what
        // comes before and after the range.
        let new_node = new_node
            .into_container()
            .expect("New node is not a container");
        let new_node =
            DomNode::Container(new_node.clone_with_new_children(inside));
        let parent = self.lookup_container_mut(&shared_parent);
        let inserted = shared_parent.child_handle(parent.children().len());
        parent.append_child(new_node);
        parent.append_children(after);

        #[cfg(any(test, feature = "assert-invariants"))]
        self.assert_invariants();

        inserted
    }

    /// Split the node at depth `depth` above the leaf at `handle`, at
    /// `offset` inside the leaf, and return its children from there on.
    fn split_children_from(
        &mut self,
        handle: &DomHandle,
        offset: usize,
        depth: usize,
    ) -> Vec<DomNode<S>> {
        let mut right = self.split_sub_tree_from(handle, offset, depth);
        // Splitting at the end of a node leaves an empty copy of it at the
        // start of the right part: remove the top-most one.
        let mut first = DomHandle::root().child_handle(0);
        while right.contains(&first) {
            let node = right.lookup_node(&first);
            if !node.is_container_node() {
                break;
            }
            if node.text_len() == 0 {
                if !node.is_block_node() {
                    right.remove(&first);
                }
                break;
            }
            first = first.child_handle(0);
        }
        right.into_container().take_children()
    }

    fn lookup_container_mut(
        &mut self,
        handle: &DomHandle,
    ) -> &mut ContainerNode<S> {
        match self.lookup_node_mut(handle) {
            DomNode::Container(container) => container,
            _ => panic!("Node is not a container"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        tests::{testutils_composer_model::cm, testutils_conversion::utf16},
        DomNode, ToHtml,
    };

    #[test]
    fn insert_parent_flat_part() {
        let mut model = cm("A{B}|C");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], vec![]),
        );

        assert_eq!(model.state.dom.to_html(), r#"A<a href="link">B</a>C"#)
    }

    #[test]
    fn insert_parent_flat_start() {
        let mut model = cm("{AB}|C");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], vec![]),
        );

        assert_eq!(model.state.dom.to_html(), r#"<a href="link">AB</a>C"#)
    }

    #[test]
    fn insert_parent_flat_end() {
        let mut model = cm("A{BC}|");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], vec![]),
        );

        assert_eq!(model.state.dom.to_html(), r#"A<a href="link">BC</a>"#)
    }

    #[test]
    fn insert_parent_flat_whole() {
        let mut model = cm("{ABC}|");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], vec![]),
        );

        assert_eq!(model.state.dom.to_html(), r#"<a href="link">ABC</a>"#)
    }

    #[test]
    fn insert_parent_simple() {
        let mut model = cm("{<b>AB</b><i><u>C</u></i>}|");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], vec![]),
        );

        assert_eq!(
            model.state.dom.to_html(),
            r#"<a href="link"><b>AB</b><i><u>C</u></i></a>"#
        )
    }

    #[test]
    fn insert_parent_ignores_adjacent_nodes() {
        let mut model = cm("X{<b>AB</b><i><u>C</u></i>}|<u>D</u>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], vec![]),
        );

        assert_eq!(
            model.state.dom.to_html(),
            r#"X<a href="link"><b>AB</b><i><u>C</u></i></a><u>D</u>"#
        )
    }

    #[test]
    fn insert_parent_splits_single_partially_covered_node() {
        let mut model = cm("<b><u>XX{ABC}|YY</u></b>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], vec![]),
        );

        assert_eq!(
            model.state.dom.to_html(),
            r#"<b><u>XX<a href="link">ABC</a>YY</u></b>"#
        )
    }

    #[test]
    fn insert_parent_splits_multiple_partially_covered_nodes() {
        let mut model = cm("XX{A<b>B</b><u>C}|YY</u>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], vec![]),
        );

        assert_eq!(
            model.state.dom.to_html(),
            r#"XX<a href="link">A<b>B</b><u>C</u></a><u>YY</u>"#
        )
    }

    #[test]
    fn insert_parent_splits_multiple_nested_partially_covered_nodes() {
        let mut model = cm("<i><u>X{X</u>ABC<b>Y}|Y</b></i>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], vec![]),
        );

        assert_eq!(
            model.state.dom.to_html(),
            r#"<i><u>X</u><a href="link"><u>X</u>ABC<b>Y</b></a><b>Y</b></i>"#
        )
    }

    #[test]
    fn insert_parent_ignores_shared_parent_nodes() {
        let mut model = cm("<i><em>XX{A<b>B</b><u>C}|YY</u></em></i>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], vec![]),
        );

        assert_eq!(
            model.state.dom.to_html(),
            r#"<i><em>XX<a href="link">A<b>B</b><u>C</u></a><u>YY</u></em></i>"#
        )
    }

    #[test]
    fn insert_parent_ignores_shared_parent_nodes_with_siblings() {
        let mut model =
            cm("<u>U</u><i><em>XX{A<b>B</b><u>C}|YY</u></em></i><u>W</u>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], vec![]),
        );

        assert_eq!(
            model.state.dom.to_html(),
            r#"<u>U</u><i><em>XX<a href="link">A<b>B</b><u>C</u></a><u>YY</u></em></i><u>W</u>"#
        )
    }

    #[test]
    fn insert_parent_includes_covered_shared_parent_nodes() {
        let mut model = cm("<i><em>{A<b>B</b><u>C</u>}|</em>D</i>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], vec![]),
        );

        assert_eq!(
            model.state.dom.to_html(),
            r#"<i><a href="link"><em>A<b>B</b><u>C</u></em></a>D</i>"#
        )
    }

    #[test]
    #[should_panic]
    fn insert_parent_panics_if_new_is_not_container() {
        let mut model = cm("{X}|");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model
            .state
            .dom
            .insert_parent(&range, DomNode::new_text(utf16("not a container")));
    }
}
