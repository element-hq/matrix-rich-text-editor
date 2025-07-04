// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

//! Assertions that guarantee the Dom is in a known state.
//!
//! To see the full list of the invariants we enforce, look at the
//! assert_invariants method.
//!
//! In future, every public method on Dom should call assert_invariants at the
//! beginning and the end of the method. This will allow us to find places
//! in our code that don't follow the invariants.
//!
//! For now, add a call to explicitly_assert_invariants wherever you want to
//! make sure we comply.
//!
//! By default, outside tests, we don't assert anything. You can compile the
//! project to always make these assertions by enabling the feature
//! "assert-invariants".
//!
//! TODO: build the demo app with these assertions enabled
//! TODO: add more assertions - see the code of assert_invariants for ideas

#[cfg(any(test, feature = "assert-invariants"))]
use crate::dom::unicode_string::UnicodeStrExt;
use crate::dom::Dom;
use crate::UnicodeString;
#[cfg(any(test, feature = "assert-invariants"))]
use crate::{DomNode, ToTree};

impl<S> Dom<S>
where
    S: UnicodeString,
{
    /// In future, all public methods on Dom will check our invariants every
    /// time they are called. When that is the case, we won't need this method.
    /// For now, if we know the invariants will be satisfied, we can check
    /// them explicitly by calling this.
    pub fn explicitly_assert_invariants(&self) {
        #[cfg(any(test, feature = "assert-invariants"))]
        self.assert_invariants();
    }

    #[cfg(any(test, feature = "assert-invariants"))]
    pub(crate) fn assert_invariants(&self) {
        if self.is_transaction_in_progress() {
            // Invariants should not be asserted during a transaction
            // as the DOM is known to be in an inconsistent state
            return;
        }
        self.assert_no_empty_text_nodes();
        self.assert_no_adjacent_text_nodes();
        self.assert_exactly_one_generic_container();
        self.assert_all_nodes_in_containers_are_block_or_inline();

        // We probably want some more asserts like these:
        // self.assert_document_node_is_a_container();
        // self.assert_no_empty_containers_except_at_root();
        // self.assert_inline_code_contains_no_tags_except_line_breaks
        // self.assert_code_blocks_do_not_contain_structure_tags
        // self.assert_links_do_not_contain_structure_tags
        // self.assert_links_do_not_contain_links
        // self.assert_zero_width_spaces_are_only_in_empty_list_item_tags
    }

    #[cfg(any(test, feature = "assert-invariants"))]
    fn assert_no_empty_text_nodes(&self) {
        for text in self.iter_text() {
            if text.data().is_empty() {
                panic!(
                    "Empty text node found! handle: {:?}\n{}",
                    text.handle(),
                    self.to_tree(),
                );
            }
        }
    }

    #[cfg(any(test, feature = "assert-invariants"))]
    fn assert_no_adjacent_text_nodes(&self) {
        for node in self.iter_containers() {
            let mut prev_node: Option<&DomNode<S>> = None;
            for child in node.children() {
                if let Some(prev_node) = prev_node {
                    if let (DomNode::Text(_), DomNode::Text(_)) =
                        (prev_node, child)
                    {
                        panic!(
                            "Adjacent text nodes found! handle: {:?}\n{}",
                            prev_node.handle(),
                            self.to_tree()
                        );
                    }
                }
                prev_node = Some(child);
            }
        }
    }

    /// Check there is only one generic container and that it is the root node
    #[cfg(any(test, feature = "assert-invariants"))]
    fn assert_exactly_one_generic_container(&self) {
        use super::nodes::ContainerNodeKind;

        let generic_nodes = self
            .iter_containers()
            .filter(|n| matches!(n.kind(), ContainerNodeKind::Generic));
        let handles = generic_nodes.map(|n| n.handle()).collect::<Vec<_>>();

        if handles.len() > 1 {
            let first = handles.into_iter().find(|h| !h.is_root());
            panic!(
                "More than one generic container node found. Handle: {:?}\n{}",
                first.unwrap().raw(),
                self.to_tree()
            );
        }
    }

    #[cfg(any(test, feature = "assert-invariants"))]
    fn assert_all_nodes_in_containers_are_block_or_inline(&self) {
        for container in self.iter_containers() {
            let all_nodes_are_inline =
                container.children().iter().all(|n| !n.is_block_node());
            let all_nodes_are_block =
                container.children().iter().all(|n| n.is_block_node());
            if !all_nodes_are_inline && !all_nodes_are_block {
                panic!("All child nodes of handle {:?} must be either inline nodes or block nodes:\n{}", container.handle(), container.to_tree());
            }
        }
    }
}

#[cfg(test)]
mod test {
    use widestring::Utf16String;

    use crate::dom::nodes::{ContainerNode, TextNode};
    use crate::dom::Dom;
    use crate::{DomHandle, DomNode, InlineFormatType};

    #[test]
    fn should_not_panic_if_transaction_in_progress() {
        let mut dom = Dom::new(vec![]);
        dom.start_transaction();
        dom.insert(
            &DomHandle::root().child_handle(0),
            vec![DomNode::Text(TextNode::from(Utf16String::from("")))],
        );
        dom.assert_invariants();
    }

    #[test]
    #[should_panic(expected = "Empty text node found")]
    fn empty_text_node_fails_invariants() {
        let dom = Dom::new(vec![DomNode::Text(TextNode::from(
            Utf16String::from(""),
        ))]);

        dom.assert_invariants();
    }

    #[test]
    #[should_panic(expected = "Adjacent text nodes found")]
    fn double_text_node_fails_invariants() {
        let dom = Dom::new(vec![
            DomNode::Text(TextNode::from(Utf16String::from("a"))),
            DomNode::Text(TextNode::from(Utf16String::from("b"))),
        ]);

        dom.assert_invariants();
    }

    #[test]
    fn nonadjacent_text_nodes_are_fine() {
        let dom = Dom::new(vec![
            DomNode::new_formatting(
                InlineFormatType::Bold,
                vec![DomNode::Text(TextNode::from(Utf16String::from("a")))],
            ),
            DomNode::Text(TextNode::from(Utf16String::from("b"))),
        ]);

        dom.assert_invariants();
    }

    #[test]
    #[should_panic(
        expected = "More than one generic container node found. Handle: [1]"
    )]
    fn multiple_generic_containers_fails_invariants() {
        let dom = Dom::new(vec![
            DomNode::Text(TextNode::from(Utf16String::from("a"))),
            DomNode::Container(ContainerNode::default()),
        ]);

        dom.assert_invariants();
    }
}
