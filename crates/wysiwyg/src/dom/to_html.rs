// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::composer_model::example_format::SelectionWriter;

use super::{
    nodes::dom_node::DomNodeKind, unicode_string::UnicodeStringExt,
    UnicodeString,
};

pub trait ToHtml<S>
where
    S: UnicodeString,
{
    /// Convert to HTML
    ///
    /// When `is_message` is true, it outputs a clean representation of the
    /// source object, suitable for sending as a message.
    fn fmt_html(
        &self,
        buf: &mut S,
        selection_writer: Option<&mut SelectionWriter>,
        state: &ToHtmlState,
        as_message: bool,
    );

    /// Convert to a clean HTML represention of the source object, suitable
    /// for sending as a message
    fn to_message_html(&self) -> S {
        let mut buf = S::default();
        self.fmt_html(&mut buf, None, &ToHtmlState::default(), true);
        buf
    }

    /// Convert to a literal HTML represention of the source object
    fn to_html(&self) -> S {
        let mut buf = S::default();
        self.fmt_html(&mut buf, None, &ToHtmlState::default(), false);
        buf
    }
}

pub trait ToHtmlExt<S>: ToHtml<S>
where
    S: UnicodeString,
{
    fn fmt_tag_open(
        &self,
        name: &S::Str,
        formatter: &mut S,
        attrs: &Option<Vec<(S, S)>>,
    );
    fn fmt_tag_close(&self, name: &S::Str, formatter: &mut S);
}

impl<S, H: ToHtml<S>> ToHtmlExt<S> for H
where
    S: UnicodeString,
{
    fn fmt_tag_close(&self, name: &S::Str, formatter: &mut S) {
        formatter.push("</");
        formatter.push(name);
        formatter.push('>');
    }

    fn fmt_tag_open(
        &self,
        name: &S::Str,
        formatter: &mut S,
        attrs: &Option<Vec<(S, S)>>,
    ) {
        formatter.push('<');
        formatter.push(name);
        if let Some(attrs) = attrs {
            for attr in attrs {
                let (attr_name, value) = attr;
                formatter.push(' ');
                formatter.push(&**attr_name);
                formatter.push("=\"");
                formatter.push(&**value);
                formatter.push('"');
            }
        }
        formatter.push('>');
    }
}

/// State of the HTML generation at every `fmt_html` call, usually used to pass info from ancestor
/// nodes to their descendants.
#[derive(Clone, Default)]
pub struct ToHtmlState {
    pub is_inside_code_block: bool,
    pub prev_sibling: Option<DomNodeKind>,
    pub next_sibling: Option<DomNodeKind>,
}
