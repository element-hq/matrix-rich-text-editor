// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use widestring::Utf16String;

use crate::dom::nodes::DomNode;
use crate::dom::Dom;
use crate::DomHandle;

use crate::tests::testutils_conversion::utf16;

pub fn dom<'a>(
    children: impl IntoIterator<Item = &'a DomNode<Utf16String>>,
) -> Dom<Utf16String> {
    Dom::new(clone_children(children))
}

pub fn a<'a>(
    children: impl IntoIterator<Item = &'a DomNode<Utf16String>>,
) -> DomNode<Utf16String> {
    DomNode::new_link(
        utf16("https://element.io"),
        clone_children(children),
        vec![],
    )
}

pub fn b<'a>(
    children: impl IntoIterator<Item = &'a DomNode<Utf16String>>,
) -> DomNode<Utf16String> {
    DomNode::new_formatting_from_tag(utf16("b"), clone_children(children))
}

pub fn i<'a>(
    children: impl IntoIterator<Item = &'a DomNode<Utf16String>>,
) -> DomNode<Utf16String> {
    DomNode::new_formatting_from_tag(utf16("i"), clone_children(children))
}

pub fn i_c<'a>(
    children: impl IntoIterator<Item = &'a DomNode<Utf16String>>,
) -> DomNode<Utf16String> {
    DomNode::new_formatting_from_tag(utf16("code"), clone_children(children))
}

fn clone_children<'a>(
    children: impl IntoIterator<Item = &'a DomNode<Utf16String>>,
) -> Vec<DomNode<Utf16String>> {
    children.into_iter().cloned().collect()
}

pub fn tn(data: &str) -> DomNode<Utf16String> {
    DomNode::new_text(utf16(data))
}

pub fn handle(raw_path: Vec<usize>) -> DomHandle {
    DomHandle::from_raw(raw_path)
}
