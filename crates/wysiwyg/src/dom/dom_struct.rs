// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use std::fmt::Display;

use crate::composer_model::example_format::SelectionWriter;
use crate::dom::nodes::{ContainerNode, DomNode};
use crate::dom::to_html::ToHtmlState;
use crate::dom::to_markdown::{MarkdownError, MarkdownOptions, ToMarkdown};
use crate::dom::unicode_string::UnicodeStrExt;
use crate::dom::{
    find_range, to_raw_text::ToRawText, DomHandle, Range, ToTree, UnicodeString,
};
use crate::ToHtml;

use super::to_plain_text::ToPlainText;
use super::FindResult;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Dom<S>
where
    S: UnicodeString,
{
    document: DomNode<S>,
    #[cfg(any(test, feature = "assert-invariants"))]
    is_transaction_in_progress: bool,
}

impl<S> Dom<S>
where
    S: UnicodeString,
{
    pub fn new(top_level_items: Vec<DomNode<S>>) -> Self {
        let mut document = ContainerNode::default();
        document.append_children(top_level_items);
        document.set_handle(DomHandle::root());

        Self {
            document: DomNode::Container(document),
            #[cfg(any(test, feature = "assert-invariants"))]
            is_transaction_in_progress: false,
        }
    }

    /// Creates a new Dom using the passed `root_node` as its root document. This `root_node` must
    /// be a container node.
    pub fn new_with_root(root_node: DomNode<S>) -> Self {
        assert!(
            root_node.is_container_node(),
            "Root node cannot be of type {:?}",
            root_node.kind()
        );
        let mut root_node = root_node;
        root_node.set_handle(DomHandle::root());

        Self {
            document: root_node,
            #[cfg(any(test, feature = "assert-invariants"))]
            is_transaction_in_progress: false,
        }
    }

    pub fn document(&self) -> &ContainerNode<S> {
        // Would be nice if we could avoid this, but it is really convenient
        // in several places to be able to treat document as a DomNode.
        if let DomNode::Container(ret) = &self.document {
            ret
        } else {
            panic!("Document should always be a Container!")
        }
    }

    pub fn into_container(self) -> ContainerNode<S> {
        if let DomNode::Container(ret) = self.document {
            ret
        } else {
            panic!("Document should always be a Container!")
        }
    }

    pub fn document_mut(&mut self) -> &mut ContainerNode<S> {
        // Would be nice if we could avoid this, but it is really convenient
        // in several places to be able to treat document as a DomNode.
        if let DomNode::Container(ret) = &mut self.document {
            ret
        } else {
            panic!("Document should always be a Container!")
        }
    }

    pub fn document_node(&self) -> &DomNode<S> {
        &self.document
    }

    pub fn into_document_node(self) -> DomNode<S> {
        self.document
    }

    pub fn into_node(mut self, handle: &DomHandle) -> DomNode<S> {
        if handle.is_root() {
            self.into_document_node()
        } else {
            self.remove(handle)
        }
    }

    pub fn children(&self) -> &Vec<DomNode<S>> {
        self.document().children()
    }

    /// Check if a transaction is in progress.
    /// Inside a transaction, multiple DOM operations can be performed without
    /// the need to keep the DOM in a consistent state between them.
    #[cfg(any(test, feature = "assert-invariants"))]
    pub fn is_transaction_in_progress(&self) -> bool {
        self.is_transaction_in_progress
    }

    /// Asserts there is no transaction in progress (which implies the
    /// Dom not known to be in bad state).
    /// See [is_transaction_in_progress()].
    #[cfg(any(test, feature = "assert-invariants"))]
    pub fn assert_transaction_not_in_progress(&self) {
        if self.is_transaction_in_progress() {
            panic!("Transaction in progress");
        }
    }

    /// Asserts the DOM is in a good state and starts a transaction.
    /// See [is_transaction_in_progress()].
    #[cfg(any(test, feature = "assert-invariants"))]
    pub fn start_transaction(&mut self) {
        if self.is_transaction_in_progress() {
            panic!("Cannot start transaction as one is already in progress");
        }
        self.assert_invariants();
        self.is_transaction_in_progress = true;
    }

    /// Ends the current transaction and asserts the DOM is still in a good
    /// state.
    /// See [is_transaction_in_progress()].
    #[cfg(any(test, feature = "assert-invariants"))]
    pub fn end_transaction(&mut self) {
        if !self.is_transaction_in_progress() {
            panic!("Cannot end transaction as no transaction is in progress");
        }
        self.is_transaction_in_progress = false;
        self.assert_invariants();
    }

    /// Returns the last node handle of the Dom. It's useful for reverse iterators that should start
    /// at the end of the Dom.
    pub fn last_node_handle(&self) -> DomHandle {
        self.last_node_handle_in_sub_tree(&DomHandle::root())
    }

    /// Returns the node handle of the sub-tree, searching recursively. It's specially useful when
    /// we want to iterate through the sub-tree in reverse order.
    ///
    /// Example, in this sub-tree:
    /// ```text
    /// ├>b
    /// │ └>"Bold"
    /// └>i
    ///   └>"Italic"
    /// ```
    /// This method will return `[1, 0]`, `"Italic"`.
    pub fn last_node_handle_in_sub_tree(
        &self,
        handle: &DomHandle,
    ) -> DomHandle {
        if let DomNode::Container(container) = self.lookup_node(handle) {
            if !container.children().is_empty() {
                let cur_handle =
                    handle.child_handle(container.children().len() - 1);
                self.last_node_handle_in_sub_tree(&cur_handle)
            } else {
                // Empty container node.
                // We might reach this line if we use the function while we are editing the Dom.
                handle.clone()
            }
        } else {
            // Leaf node
            handle.clone()
        }
    }

    #[cfg(all(feature = "js", target_arch = "wasm32"))]
    pub(crate) fn take_children(self) -> Vec<DomNode<S>> {
        if let DomNode::Container(container) = self.document {
            container.take_children()
        } else {
            panic!("Document should always be a Container!")
        }
    }

    /// Appends the [child] node at the end of the root node (document node).
    pub fn append_at_end_of_document(
        &mut self,
        child: DomNode<S>,
    ) -> DomHandle {
        self.document_mut().append_child(child)
    }

    /// Appends [child] node at the end of the node at [parent_handle], if it's a container node.
    /// Returns the [DomHandle] of the added node.
    pub fn append(
        &mut self,
        parent_handle: &DomHandle,
        child: DomNode<S>,
    ) -> DomHandle {
        let parent = if let DomNode::Container(container) =
            self.lookup_node_mut(parent_handle)
        {
            container
        } else {
            panic!("Parent is not a container node");
        };
        parent.append_child(child)
    }

    /// Inserts the given [node] at the [node_handle] position, moving the node at that position
    /// forward, if any.
    pub fn insert_at(
        &mut self,
        node_handle: &DomHandle,
        node: DomNode<S>,
    ) -> DomHandle {
        let parent = self.parent_mut(node_handle);
        let index = node_handle.index_in_parent();
        parent.insert_child(index, node).handle()
    }

    /// Insert given [nodes] in order at the [node_handle] position,
    /// moving the node at that position forward, if any.
    pub fn insert(
        &mut self,
        node_handle: &DomHandle,
        nodes: Vec<DomNode<S>>,
    ) -> Vec<DomHandle> {
        let parent = self.parent_mut(node_handle);
        let index = node_handle.index_in_parent();
        parent.insert_children(index, nodes)
    }

    /// Replaces the node at [node_handle] with the list of [nodes]. Returns the list of new handles
    /// added to the Dom for moved nodes.
    pub fn replace(
        &mut self,
        node_handle: &DomHandle,
        nodes: Vec<DomNode<S>>,
    ) -> Vec<DomHandle> {
        let parent = self.parent_mut(node_handle);
        let index = node_handle.index_in_parent();
        parent.replace_child(index, nodes)
    }

    /// Replaces the node at [node_handle] with its children. Returns the list of new handles
    /// for the replaced children, but only if the [node_handle] belonged to a container
    /// otherwise will return an empy list.
    pub fn replace_node_with_its_children(
        &mut self,
        node_handle: &DomHandle,
    ) -> Vec<DomHandle> {
        let node = self.lookup_node(node_handle);
        let Some(parent) = node.as_container() else {
            return vec![];
        };
        let ret = self.replace(node_handle, parent.children().clone());
        self.join_nodes_in_container(&node_handle.parent_handle());
        ret
    }

    /// Removes the node at [node_handle] and returns it.
    pub fn remove(&mut self, node_handle: &DomHandle) -> DomNode<S> {
        let parent = self.parent_mut(node_handle);
        let index = node_handle.index_in_parent();
        parent.remove_child(index)
    }

    /// Returns the root document of this Dom.
    ///
    /// **Note**: this call moves the Dom, so it becomes unusable.
    pub fn take_document(self) -> DomNode<S> {
        self.document
    }

    /// Given the start and end code units, find which nodes of this Dom are
    /// selected. The returned range lists all the Dom nodes involved.
    pub fn find_range(&self, start: usize, end: usize) -> Range {
        find_range::find_range(self, start, end)
    }

    pub fn find_range_by_node(&self, node_handle: &DomHandle) -> Range {
        let result = find_range::find_pos(self, node_handle, 0, usize::MAX);

        let locations = match result {
            FindResult::Found(locations) => locations,
            _ => panic!("Node does not exist"),
        };

        let leaves = locations.iter().filter(|l| l.is_leaf());

        let s = leaves.clone().map(|l| l.position).min().unwrap();
        let e = leaves.map(|l| l.position + l.length).max().unwrap();

        self.find_range(s, e)
    }

    pub(crate) fn document_handle(&self) -> DomHandle {
        self.document.handle()
    }

    pub fn find_ancestor_list_item_or_self(
        &self,
        child_handle: &DomHandle,
    ) -> Option<DomHandle> {
        child_handle
            .with_ancestors()
            .iter()
            .rev()
            .find(|handle| {
                if let DomNode::Container(n) = self.lookup_node(handle) {
                    n.is_list_item()
                } else {
                    false
                }
            })
            .cloned()
    }

    /// Determine if a node handle has any container ancestors with the attribute contenteditable=false
    pub fn has_immutable_ancestor(&self, child_handle: &DomHandle) -> bool {
        child_handle.with_ancestors().iter().rev().any(|handle| {
            if let DomNode::Container(n) = self.lookup_node(handle) {
                n.is_immutable()
            } else {
                false
            }
        })
    }

    /// Find the node based on its handle.
    /// Panics if the handle is unset or invalid
    pub fn lookup_node(&self, node_handle: &DomHandle) -> &DomNode<S> {
        self.document_node().lookup_node(node_handle)
    }

    /// Shortcut for looking up a container at given handle.
    /// Should only be used from contexts where node is
    /// guaranteed to be a container. (e.g. if `DomLocation`
    /// provides a DomNodeKind matching container type).
    pub fn lookup_container(
        &self,
        node_handle: &DomHandle,
    ) -> &ContainerNode<S> {
        self.lookup_node(node_handle).as_container().unwrap()
    }

    /// Find the node based on its handle and returns a mutable reference.
    /// Panics if the handle is invalid or unset
    pub fn lookup_node_mut(
        &mut self,
        node_handle: &DomHandle,
    ) -> &mut DomNode<S> {
        fn nth_child<S>(
            element: &mut ContainerNode<S>,
            idx: usize,
        ) -> &mut DomNode<S>
        where
            S: UnicodeString,
        {
            element.get_child_mut(idx).expect(
                "Handle is invalid: it refers to a child index which is too \
                large for the number of children in this node.",
            )
        }

        let mut node = &mut self.document;
        if !node_handle.is_set() {
            panic!(
                "Attempting to lookup a node using an unset DomHandle ({:?})",
                node_handle.raw()
            );
        }
        for idx in node_handle.raw() {
            node = match node {
                DomNode::Container(n) => nth_child(n, *idx),
                DomNode::LineBreak(_) => panic!(
                    "Handle is invalid: refers to the child of a line break, \
                    but line breaks cannot have children."
                ),
                DomNode::Text(_) => panic!(
                    "Handle is invalid: refers to the child of a text node, \
                    but text nodes cannot have children."
                ),
                DomNode::Mention(_) => panic!(
                    "Handle is invalid: refers to the child of a mention node, \
                    but mention nodes cannot have children."
                ),
            }
        }

        node
    }

    /// Return the number of code points in the string representation of this
    /// Dom.
    pub fn text_len(&self) -> usize {
        self.document.text_len()
    }

    /// Add the supplied new_node into the text of the supplied handle, at
    /// the offset supplied.
    ///
    /// If handle points to a text node, this text node may be split if needed.
    /// If handle points to a line break node, offset should definitely be 1,
    /// and the new node will be inserted after it.
    ///
    /// Returns the handle of the inserted node.
    pub fn insert_into_text(
        &mut self,
        handle: &DomHandle,
        offset: usize,
        new_node: DomNode<S>,
    ) -> DomHandle {
        enum Where {
            Before,
            During,
            After,
        }

        let wh = match self.lookup_node(handle) {
            DomNode::Container(_) => {
                panic!("Can't insert into a non-text node!")
            }
            DomNode::LineBreak(_) | DomNode::Mention(_) => {
                if offset == 0 {
                    Where::Before
                } else if offset == 1 {
                    Where::After
                } else {
                    panic!(
                        "Attempting to insert into a node of length 1, but offset wasn't \
                        either 0 or 1: {}",
                        offset
                    );
                }
            }
            DomNode::Text(n) => {
                if offset == 0 {
                    Where::Before
                } else if offset == n.data().len() {
                    Where::After
                } else {
                    Where::During
                }
            }
        };

        match wh {
            Where::Before => self
                .parent_mut(handle)
                .insert_child(handle.index_in_parent(), new_node)
                .handle(),
            Where::During => {
                // Splice new_node in between this text node and a new one
                let old_node = self.lookup_node_mut(handle);
                if let DomNode::Text(old_text_node) = old_node {
                    let data = old_text_node.data();
                    let before_text = data[..offset].to_owned();
                    let after_text = data[offset..].to_owned();
                    old_text_node.set_data(before_text);
                    let new_text_node = DomNode::new_text(after_text);
                    let parent = self.parent_mut(handle);
                    let inserted_handle = parent
                        .insert_child(handle.index_in_parent() + 1, new_node)
                        .handle();
                    parent.insert_child(
                        handle.index_in_parent() + 2,
                        new_text_node,
                    );
                    inserted_handle
                } else {
                    panic!("Can't insert in the middle of non-text node!");
                }
            }
            Where::After => self
                .parent_mut(handle)
                .insert_child(handle.index_in_parent() + 1, new_node)
                .handle(),
        }
    }

    /// Look up the parent node of the node pointed to by this handle and
    /// provide a mutable reference.
    /// Panics if:
    /// * this handle has no parent (it is the root)
    /// * the parent is not a container node
    /// * the handle is invalid
    pub fn parent_mut(&mut self, handle: &DomHandle) -> &mut ContainerNode<S> {
        let parent = self.lookup_node_mut(&handle.parent_handle());
        if let DomNode::Container(parent) = parent {
            parent
        } else {
            panic!(
                "Parent node was not a container! handle={:?} parent={:?}",
                handle, parent
            );
        }
    }

    /// Look up the parent node of the node pointed to by this handle.
    /// Panics if:
    /// * this handle has no parent (it is the root)
    /// * the parent is not a container node
    /// * the handle is invalid
    pub fn parent(&self, handle: &DomHandle) -> &ContainerNode<S> {
        let parent = self.lookup_node(&handle.parent_handle());
        if let DomNode::Container(parent) = parent {
            parent
        } else {
            panic!("Parent node was not a container!");
        }
    }

    /// Checks if the passed [handle] exists in the DOM.
    pub fn contains(&self, handle: &DomHandle) -> bool {
        let mut current = self.document();
        let path_len = handle.raw().len();
        for i in 0..=path_len {
            let sub_handle = handle.sub_handle_up_to(i);
            if sub_handle.is_root() {
                continue;
            } else if let Some::<&DomNode<S>>(node) =
                current.children().get(sub_handle.index_in_parent())
            {
                if let DomNode::Container(node) = node {
                    current = node;
                } else if i != path_len {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    /// Checks if the passed handle is the last one in its parent.
    pub fn is_last_in_parent(&self, handle: &DomHandle) -> bool {
        self.parent(handle).children().len() == handle.index_in_parent() + 1
    }

    /// Gets the previous sibling of the node if exists.
    pub fn prev_sibling(&self, handle: &DomHandle) -> Option<&DomNode<S>> {
        if handle.index_in_parent() == 0 {
            return None;
        }
        let prev_handle = handle.prev_sibling();
        if self.contains(&prev_handle) {
            Some(self.lookup_node(&prev_handle))
        } else {
            None
        }
    }

    /// Gets the previous sibling of the node if exists as a mut ref.
    pub fn prev_sibling_mut(
        &mut self,
        handle: &DomHandle,
    ) -> Option<&mut DomNode<S>> {
        if handle.index_in_parent() == 0 {
            return None;
        }
        let prev_handle = handle.prev_sibling();
        if self.contains(&prev_handle) {
            Some(self.lookup_node_mut(&prev_handle))
        } else {
            None
        }
    }

    /// Gets the next sibling of the node if exists.
    pub fn next_sibling(&self, handle: &DomHandle) -> Option<&DomNode<S>> {
        let next_handle = handle.next_sibling();
        if self.contains(&next_handle) {
            Some(self.lookup_node(&next_handle))
        } else {
            None
        }
    }

    /// Gets the next sibling of the node if exists as a mut ref.
    pub fn next_sibling_mut(
        &mut self,
        handle: &DomHandle,
    ) -> Option<&mut DomNode<S>> {
        let next_handle = handle.next_sibling();
        if self.contains(&next_handle) {
            Some(self.lookup_node_mut(&next_handle))
        } else {
            None
        }
    }
}

impl<S> ToHtml<S> for Dom<S>
where
    S: UnicodeString,
{
    fn fmt_html(
        &self,
        buf: &mut S,
        selection_writer: Option<&mut SelectionWriter>,
        state: &ToHtmlState,
        as_message: bool,
    ) {
        self.document
            .fmt_html(buf, selection_writer, state, as_message)
    }
}

impl<S> ToRawText<S> for Dom<S>
where
    S: UnicodeString,
{
    fn to_raw_text(&self) -> S {
        self.document.to_raw_text()
    }
}

impl<S> ToPlainText<S> for Dom<S>
where
    S: UnicodeString,
{
    fn to_plain_text(&self) -> S {
        self.document.to_plain_text()
    }
}

impl<S> ToTree<S> for Dom<S>
where
    S: UnicodeString,
{
    fn to_tree_display(&self, continuous_positions: Vec<usize>) -> S {
        self.document.to_tree_display(continuous_positions)
    }
}

impl<S> ToMarkdown<S> for Dom<S>
where
    S: UnicodeString,
{
    fn fmt_markdown(
        &self,
        buffer: &mut S,
        options: &MarkdownOptions,
        as_message: bool,
    ) -> Result<(), MarkdownError<S>> {
        self.document.fmt_markdown(buffer, options, as_message)
    }
}

impl<S> Display for Dom<S>
where
    S: UnicodeString,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_html().to_string())
    }
}

/* TODO
#[derive(Clone, Debug, PartialEq)]
struct ItemNode {}

impl Display for ItemNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
*/

#[cfg(test)]
mod test {
    use widestring::Utf16String;

    use crate::dom::nodes::dom_node::DomNode;
    use crate::dom::nodes::TextNode;
    use crate::tests::testutils_composer_model::cm;
    use crate::tests::testutils_conversion::utf16;
    use crate::tests::testutils_dom::{a, b, dom, handle, i, i_c, tn};

    use super::*;

    // Creation and handles

    #[test]
    fn can_create_a_dom_and_add_nodes() {
        // Create a simple DOM
        let dom = Dom::new(vec![
            DomNode::Text(TextNode::from(utf16("a"))),
            DomNode::new_formatting_from_tag(
                utf16("b"),
                vec![DomNode::new_text(utf16("b"))],
            ),
        ]);

        // The DOM was created successfully
        assert_eq!(dom.to_string(), "a<b>b</b>");
    }

    #[test]
    fn can_find_toplevel_nodes_via_handles() {
        // Create a simple DOM
        let dom = Dom::new(vec![
            DomNode::new_text(utf16("a")),
            DomNode::new_formatting_from_tag(
                utf16("b"),
                vec![DomNode::new_text(utf16("b"))],
            ),
        ]);

        let child0 = &dom.children()[0];
        let child1 = &dom.children()[1];

        // The handles point to the right nodes
        assert_eq!(dom.lookup_node(&child0.handle()), child0);
        assert_eq!(dom.lookup_node(&child1.handle()), child1);
    }

    #[test]
    fn can_find_deep_nodes_via_handles() {
        let dom = dom(&[
            tn("foo"),
            b(&[tn("BOLD"), b(&[tn("uberbold")])]),
            tn("bar"),
        ]);

        // Given a DOM with a nested node
        let nested_node = &kids(&kids(&dom.children()[1])[1])[0];

        // When we ask for its handle
        let handle = nested_node.handle();

        // Then we can look it up and find the same node
        assert_eq!(dom.lookup_node(&handle), nested_node);
    }

    #[test]
    fn can_replace_toplevel_node_with_multiple_nodes() {
        let mut dom = dom(&[tn("foo"), tn("bar")]);

        let node_handle = dom.children()[0].handle();
        let inserted_nodes = vec![tn("ab"), b(&[tn("cd")]), tn("ef")];

        dom.replace(&node_handle, inserted_nodes);

        // Node is replaced by new insertion
        assert_eq!(dom.to_string(), "ab<b>cd</b>efbar");
        // Subsequent node handle is properly updated
        let bar_node = &dom.children()[3];
        assert_eq!(bar_node.handle().index_in_parent(), 3);
    }

    #[test]
    fn can_replace_deep_node_with_multiple_nodes() {
        let mut dom = dom(&[b(&[tn("foo")])]);

        let node_handle = kids(&dom.children()[0])[0].handle();
        let inserted_nodes = vec![tn("f"), i(&[tn("o")]), tn("o")];

        dom.replace(&node_handle, inserted_nodes);

        // Node is replaced by new insertion
        assert_eq!(dom.to_string(), "<b>f<i>o</i>o</b>");
    }

    #[test]
    fn can_remove_node_and_keep_children() {
        let mut dom = dom(&[b(&[tn("foo"), i(&[tn("bar")])])]);
        let node_handle = &dom.children()[0].handle();

        dom.remove_and_keep_children(node_handle);
        // Node is removed and replaced by its children
        assert_eq!(dom.to_string(), "foo<i>bar</i>");
    }

    // Serialisation

    #[test]
    fn empty_dom_serialises_to_empty_string() {
        assert_eq!(dom(&[]).to_string(), "");
    }

    #[test]
    fn plain_text_serialises_to_just_the_text() {
        assert_eq!(dom(&[tn("foo")]).to_string(), "foo");
    }

    #[test]
    fn mixed_text_and_tags_serialises() {
        assert_eq!(
            dom(&[tn("foo"), b(&[tn("BOLD")]), tn("bar")]).to_string(),
            "foo<b>BOLD</b>bar"
        );
    }

    #[test]
    fn nested_tags_serialise() {
        assert_eq!(
            dom(&[
                tn("foo"),
                b(&[tn("BO"), i(&[tn("LD")])]),
                i(&[tn("it")]),
                tn("bar")
            ])
            .to_string(),
            "foo<b>BO<i>LD</i></b><i>it</i>bar"
        );
    }

    #[test]
    fn empty_tag_serialises() {
        assert_eq!(dom(&[b(&[]),]).to_string(), "<b></b>");
    }

    #[test]
    fn into_node() {
        assert!(dom(&[tn("foo")]).into_node(&handle(vec![0])).is_text_node())
    }

    #[test]
    fn into_node_from_root() {
        assert!(dom(&[tn("foo")])
            .into_node(&handle(vec![]))
            .is_container_node())
    }

    #[test]
    fn hyperlink_formatting_simple() {
        let d = dom(&[a(&[tn("foo")])]);
        assert_eq!(
            "<a href=\"https://element.io\">foo</a>",
            d.to_html().to_string()
        );
    }

    #[test]
    fn hyperlink_formatting_complex() {
        let d = dom(&[a(&[b(&[tn("foo")]), tn(" "), i(&[tn("bar")])])]);
        assert_eq!(
            "<a href=\"https://element.io\"><b>foo</b> <i>bar</i></a>",
            d.to_html().to_string()
        );
    }

    #[test]
    fn inline_code_gets_formatted() {
        let d = dom(&[i_c(&[tn("some_code")])]);
        assert_eq!("<code>some_code</code>", d.to_html().to_string());
    }

    #[test]
    fn html_symbols_inside_text_tags_get_escaped() {
        let d = dom(&[tn("<p>Foo & bar</p>")]);
        assert_eq!("&lt;p&gt;Foo &amp; bar&lt;/p&gt;", d.to_html().to_string());
    }

    #[test]
    fn inline_code_text_contents_get_escaped() {
        let d = dom(&[i_c(&[tn("<b>some</b> code")])]);
        assert_eq!(
            "<code>&lt;b&gt;some&lt;/b&gt; code</code>",
            d.to_html().to_string()
        );
    }

    #[test]
    fn inline_code_node_contents_do_not_get_escaped() {
        let d = dom(&[i_c(&[b(&[tn("some")]), tn(" code")])]);
        assert_eq!("<code><b>some</b> code</code>", d.to_html().to_string());
    }

    #[test]
    fn text_len_ignores_html_tags() {
        assert_eq!(0, cm("|").state.dom.text_len());
        assert_eq!(2, cm("aa|").state.dom.text_len());
        assert_eq!(2, cm("a<b>a|</b>").state.dom.text_len());
        assert_eq!(2, cm("<del><i>a</i><b>a|</b></del>").state.dom.text_len());
    }

    #[test]
    fn text_len_counts_brs_as_1() {
        assert_eq!(1, cm("<br />|").state.dom.text_len());
        assert_eq!(3, cm("a|<br />b").state.dom.text_len());
    }

    #[test]
    fn find_parent_list_item_or_self_finds_our_parent() {
        let d = cm("|a<ul><li>b</li></ul>").state.dom;
        let res =
            d.find_ancestor_list_item_or_self(&DomHandle::from_raw(vec![
                1, 0, 0,
            ]));
        let res = res.expect("Should have found a list parent!");
        assert_eq!(res.into_raw(), vec![1, 0]);
    }

    #[test]
    fn find_parent_list_item_or_self_finds_ourself() {
        let d = cm("|a<ul><li>b</li></ul>").state.dom;
        let res =
            d.find_ancestor_list_item_or_self(&DomHandle::from_raw(vec![1, 0]));
        let res = res.expect("Should have found a list parent!");
        assert_eq!(res.into_raw(), vec![1, 0]);
    }

    #[test]
    fn find_parent_list_item_or_self_finds_our_grandparent() {
        let d = cm("|<ul><li>b<strong>c</strong></li></ul>d").state.dom;
        let res =
            d.find_ancestor_list_item_or_self(&DomHandle::from_raw(vec![
                0, 0, 1, 0,
            ]));
        let res = res.expect("Should have found a list parent!");
        assert_eq!(res.into_raw(), vec![0, 0]);
    }

    #[test]
    fn find_parent_list_item_or_self_returns_none_when_not_in_a_list() {
        let d = cm("|<ul><li>b<strong>c</strong></li></ul>d").state.dom;
        let res =
            d.find_ancestor_list_item_or_self(&DomHandle::from_raw(vec![1]));
        assert!(res.is_none(), "Should not have found a list parent!")
    }

    #[test]
    fn node_exists_returns_true_if_exists() {
        let d = cm("<ul><li>b<strong>c</strong></li></ul>d|").state.dom;
        let handle = DomHandle::from_raw(vec![0, 0, 1, 0]);
        assert!(d.contains(&handle));
    }

    #[test]
    fn node_exists_returns_false_if_child_does_not_exist() {
        let d = cm("<ul><li>b<strong>c</strong></li></ul>d|").state.dom;
        // Last level doesn't exist, [0, 0, 1, 0] is a leaf node
        let handle = DomHandle::from_raw(vec![0, 0, 1, 0, 2]);
        assert!(!d.contains(&handle));
    }

    #[test]
    fn node_exists_returns_false_if_sibling_does_not_exist() {
        let d = cm("<ul><li>b<strong>c</strong></li></ul>d|").state.dom;
        // Last level doesn't exist, [0, 0, 1] does not have 6 children
        let handle = DomHandle::from_raw(vec![0, 0, 1, 5]);
        assert!(!d.contains(&handle));
    }

    #[test]
    fn find_range_by_node() {
        let d = cm("<b><u>Hello, <i>world|</i></u></b>").state.dom;
        let range_by_node =
            d.find_range_by_node(&DomHandle::from_raw(vec![0, 0, 0]));
        let actual_range = d.find_range(0, 7);

        assert_eq!(range_by_node, actual_range);
    }

    #[test]
    fn find_range_by_node_root() {
        let d = cm("<b><u>Hello, <i>world|</i></u></b>").state.dom;
        let range_by_node = d.find_range_by_node(&DomHandle::root());
        let actual_range = d.find_range(0, 12);

        assert_eq!(range_by_node, actual_range);
    }

    #[test]
    fn text_node_with_immutable_ancestor() {
        let d = cm("<a contenteditable=\"false\" href=\"https://matrix.org\">|first</a>").state.dom;
        let handle = DomHandle::from_raw(vec![0, 0]);
        let output = d.has_immutable_ancestor(&handle);
        assert!(output);
    }

    #[test]
    fn text_node_without_immutable_ancestor() {
        let d = cm("<a href=\"https://matrix.org\">|first</a>").state.dom;
        let handle = DomHandle::from_raw(vec![0, 0]);
        let output = d.has_immutable_ancestor(&handle);
        assert!(!output);
    }

    #[test]
    fn transaction_succeeds() {
        let mut d = cm("|").state.dom;
        d.start_transaction();
        d.end_transaction();
    }

    #[test]
    #[should_panic(expected = "More than one generic container node found")]
    fn transaction_start_asserts() {
        let mut bad_dom =
            dom(vec![&DomNode::Container(ContainerNode::default())]);
        bad_dom.start_transaction();
    }

    #[test]
    #[should_panic(expected = "More than one generic container node found")]
    fn transaction_end_asserts() {
        let mut dom = dom(vec![]);
        dom.start_transaction();
        let container =
            DomNode::<Utf16String>::Container(ContainerNode::default());
        dom.insert(&DomHandle::root().child_handle(0), vec![container]);
        dom.end_transaction();
    }

    #[test]
    #[should_panic(
        expected = "Cannot start transaction as one is already in progress"
    )]
    fn transaction_start_twice() {
        let mut d = cm("|").state.dom;
        d.start_transaction();
        d.start_transaction();
    }

    #[test]
    #[should_panic(
        expected = "Cannot end transaction as no transaction is in progress"
    )]
    fn transaction_end_without_start() {
        let mut d = cm("|").state.dom;
        d.end_transaction();
    }

    const NO_CHILDREN: &Vec<DomNode<Utf16String>> = &Vec::new();

    /// If this node is an element, return its children - otherwise panic
    fn kids(node: &DomNode<Utf16String>) -> &Vec<DomNode<Utf16String>> {
        match node {
            DomNode::Container(n) => n.children(),
            DomNode::LineBreak(_) | DomNode::Mention(_) => NO_CHILDREN,
            DomNode::Text(_) => {
                panic!("We expected an Element, but found Text")
            }
        }
    }
}
