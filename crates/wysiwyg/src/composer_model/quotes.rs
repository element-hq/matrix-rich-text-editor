// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::dom::nodes::dom_node::DomNodeKind::{Generic, ListItem, Quote};
use crate::dom::DomLocation;
use crate::{
    ComposerAction, ComposerModel, ComposerUpdate, DomNode, UnicodeString,
};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn quote(&mut self) -> ComposerUpdate<S> {
        if self.action_is_reversed(ComposerAction::Quote) {
            self.remove_quote()
        } else {
            self.add_quote()
        }
    }

    fn add_quote(&mut self) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();
        let Some(wrap_result) =
            self.state.dom.find_nodes_to_wrap_in_block(s, e)
        else {
            // No nodes to be wrapped found.
            // Adding an empty Quote block with a paragraph
            let range = self.state.dom.find_range(s, e);
            let leaves: Vec<&DomLocation> = range.leaves().collect();
            let node =
                DomNode::new_quote(vec![DomNode::new_paragraph(Vec::new())]);
            if leaves.is_empty() {
                if let Some(deepest_block_location) =
                    range.deepest_block_node(None)
                {
                    let mut block_node = self
                        .state
                        .dom
                        .remove(&deepest_block_location.node_handle);
                    let node = if block_node.is_list_item() {
                        let list_item = block_node.as_container_mut().unwrap();
                        let children = list_item.remove_children();
                        list_item.append_child(DomNode::new_quote(children));
                        block_node
                    } else {
                        DomNode::new_quote(vec![block_node])
                    };
                    self.state
                        .dom
                        .insert_at(&deepest_block_location.node_handle, node);
                } else {
                    self.state.dom.append_at_end_of_document(node);
                }
            } else {
                let first_leaf_loc = leaves.first().unwrap();
                let insert_at = if first_leaf_loc.is_start() {
                    first_leaf_loc.node_handle.next_sibling()
                } else {
                    first_leaf_loc.node_handle.clone()
                };
                self.state.dom.insert_at(&insert_at, node);
            }
            return self.create_update_replace_all();
        };

        let parent_handle = wrap_result.ancestor_handle;

        let start_handle = wrap_result.start_handle;
        let end_handle = wrap_result.end_handle;

        let mut subtree = self.state.dom.split_sub_tree_between(
            &start_handle,
            0,
            &end_handle,
            usize::MAX,
            parent_handle.depth(),
        );

        let insert_at_handle =
            self.state.dom.find_insert_handle_for_extracted_block_node(
                &start_handle,
                &parent_handle,
            );
        let subtree_root_kind = subtree.document_node().kind();
        let quote_node = if subtree_root_kind.is_block_kind()
            && subtree_root_kind != Generic
            && subtree_root_kind != ListItem
        {
            DomNode::new_quote(vec![subtree.take_document()])
        } else {
            let subtree_container = subtree.document_mut();
            let needs_paragraph = subtree_container
                .children()
                .iter()
                .any(|n| !n.is_block_node());
            let children = if needs_paragraph {
                vec![DomNode::new_paragraph(
                    subtree_container.remove_children(),
                )]
            } else {
                subtree_container.remove_children()
            };
            DomNode::new_quote(children)
        };

        if subtree_root_kind == ListItem {
            self.state.dom.insert_at(
                &insert_at_handle,
                DomNode::new_list_item(vec![quote_node]),
            );
        } else {
            self.state.dom.insert_at(&insert_at_handle, quote_node);
        }

        self.state.dom.join_nodes_in_container(&parent_handle);

        self.create_update_replace_all()
    }

    fn remove_quote(&mut self) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        let Some(quote_location) =
            range.locations.iter().find(|l| l.kind == Quote)
        else {
            return ComposerUpdate::keep();
        };

        self.state
            .dom
            .remove_and_keep_children(&quote_location.node_handle);

        self.create_update_replace_all()
    }
}

#[cfg(test)]
mod test {
    use crate::tests::testutils_composer_model::{cm, tx};

    #[test]
    fn apply_quote_to_empty_dom() {
        let mut model = cm("|");
        model.quote();
        assert_eq!(tx(&model), "<blockquote><p>&nbsp;|</p></blockquote>")
    }

    #[test]
    fn apply_quote_to_simple_text() {
        let mut model = cm("Some text|");
        model.quote();
        assert_eq!(tx(&model), "<blockquote><p>Some text|</p></blockquote>")
    }

    #[test]
    fn apply_quote_to_formatted_text() {
        let mut model = cm("<i>Some text|</i>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<blockquote><p><i>Some text|</i></p></blockquote>"
        )
    }

    #[test]
    fn apply_quote_to_nested_formatted_text() {
        let mut model = cm("<i><b>Some text|</b></i>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<blockquote><p><i><b>Some text|</b></i></p></blockquote>"
        )
    }

    #[test]
    fn apply_quote_to_text_and_formatted_text() {
        let mut model = cm("Plain text and <i><b>Some formatted text|</b></i>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<blockquote><p>Plain text and <i><b>Some formatted text|</b></i></p></blockquote>"
        )
    }

    #[test]
    fn apply_quote_to_single_list_item() {
        let mut model = cm("<ul><li>List item|</li></ul>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<ul><li><blockquote><p>List item|</p></blockquote></li></ul>"
        )
    }

    #[test]
    fn apply_quote_to_list() {
        let mut model =
            cm("<ul><li>First {item</li><li>Second}| item</li></ul>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<blockquote><ul><li>First {item</li><li>Second}| item</li></ul></blockquote>"
        )
    }

    #[test]
    fn apply_quote_to_list_and_formatted_text() {
        let mut model =
            cm("<p>Plain text <b>bold {text</b></p><ul><li>First item</li><li>Second}| item</li></ul>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<blockquote><p>Plain text <b>bold {text</b></p><ul><li>First item</li><li>Second}| item</li></ul></blockquote>"
        )
    }

    #[test]
    fn apply_quote_to_simple_text_with_paragraphs() {
        let mut model = cm("<p>Some |text</p><p>Next line</p>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<blockquote><p>Some |text</p></blockquote><p>Next line</p>"
        )
    }

    #[test]
    fn apply_quote_to_several_nodes_with_line_breaks() {
        let mut model = cm("<p>Some {text</p><p><b>Next}| line</b></p>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<blockquote><p>Some {text</p><p><b>Next}| line</b></p></blockquote>"
        )
    }

    #[test]
    fn apply_quote_to_formatted_text_with_line_breaks() {
        let mut model = cm("<b>Some |text<br />Next line</b>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<blockquote><p><b>Some |text</b></p></blockquote><p><b>Next line</b></p>"
        )
    }

    #[test]
    fn apply_quote_to_single_list_item_with_formatted_text_and_line_breaks() {
        let mut model = cm(
            "<ul><li><p><b>Some |text</b></p><p><b>Next line</b></p></li></ul>",
        );
        model.quote();
        assert_eq!(
            tx(&model),
            "<ul><li><blockquote><p><b>Some |text</b></p></blockquote><p><b>Next line</b></p></li></ul>"
        )
    }

    #[test]
    fn apply_quote_to_several_list_items_with_formatted_text_and_line_breaks() {
        let mut model =
            cm("<ul><li><b>Some {text<br />Next line</b></li><li><i>Second}| item</i></li><li>Third item</li></ul>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<blockquote><ul><li><p><b>Some {text</b></p><p><b>Next line</b></p></li><li><i>Second}| item</i></li></ul></blockquote><ul><li>Third item</li></ul>"
        )
    }

    #[test]
    fn apply_quote_to_code_block() {
        let mut model = cm("<pre><code>Some| code</code></pre>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<blockquote><pre><code>Some| code</code></pre></blockquote>"
        )
    }

    #[test]
    fn remove_quote_with_simple_text() {
        let mut model = cm("<blockquote><p>Text|</p></blockquote>");
        model.quote();
        assert_eq!(tx(&model), "<p>Text|</p>");
    }

    #[test]
    fn remove_quote_with_simple_text_with_adjacent_nodes() {
        let mut model =
            cm("<blockquote><p>Text|</p></blockquote><p>with plain text</p>");
        model.quote();
        assert_eq!(tx(&model), "<p>Text|</p><p>with plain text</p>");
    }

    #[test]
    fn remove_quote_with_nested_formatted_text() {
        let mut model = cm("<blockquote><p><b><i>Text|</i></b></p><p><b><i>Some other text</i></b></p></blockquote>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<p><b><i>Text|</i></b></p><p><b><i>Some other text</i></b></p>"
        );
    }

    #[test]
    fn remove_quote_with_list() {
        let mut model = cm("<blockquote><p><b><i>Text|</i></b></p><ul><li>Fist item</li></ul></blockquote>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<p><b><i>Text|</i></b></p><ul><li>Fist item</li></ul>"
        );
    }

    #[test]
    fn create_and_remove_quote() {
        let mut model = cm("|");
        model.quote();
        assert_eq!(tx(&model), "<blockquote><p>&nbsp;|</p></blockquote>");
        model.quote();
        assert_eq!(tx(&model), "<p>&nbsp;|</p>");
    }

    #[test]
    fn add_quote_to_empty_list_item() {
        let mut model = cm("<ul><li>|</li></ul>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<ul><li><blockquote><p>&nbsp;|</p></blockquote></li></ul>"
        )
    }
}
