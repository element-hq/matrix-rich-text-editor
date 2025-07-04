// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::composer_model::example_format::SelectionWriter;
use crate::dom::dom_handle::DomHandle;
use crate::dom::to_html::{ToHtml, ToHtmlState};
use crate::dom::to_markdown::{MarkdownError, MarkdownOptions, ToMarkdown};
use crate::dom::to_plain_text::ToPlainText;
use crate::dom::to_raw_text::ToRawText;
use crate::dom::to_tree::ToTree;
use crate::dom::unicode_string::{UnicodeStrExt, UnicodeStringExt};
use crate::dom::UnicodeString;
use std::marker::PhantomData;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineBreakNode<S>
where
    S: UnicodeString,
{
    _phantom_data: PhantomData<S>,
    handle: DomHandle,
}

impl<S> Default for LineBreakNode<S>
where
    S: UnicodeString,
{
    /// Create a new default LineBreakNode
    ///
    /// NOTE: Its handle() will be unset until you call set_handle() or
    /// append() it to another node.
    fn default() -> Self {
        Self {
            _phantom_data: PhantomData {},
            handle: DomHandle::new_unset(),
        }
    }
}

impl<S> LineBreakNode<S>
where
    S: UnicodeString,
{
    pub fn name(&self) -> S {
        "br".into()
    }

    pub fn set_handle(&mut self, handle: DomHandle) {
        self.handle = handle;
    }

    pub fn handle(&self) -> DomHandle {
        self.handle.clone()
    }

    // A br tag is always treated as 1 character, so this always returns 1
    pub fn text_len(&self) -> usize {
        1
    }
}

impl<S> ToHtml<S> for LineBreakNode<S>
where
    S: UnicodeString,
{
    fn fmt_html(
        &self,
        buf: &mut S,
        selection_writer: Option<&mut SelectionWriter>,
        _: &ToHtmlState,
        _as_message: bool,
    ) {
        let cur_pos = buf.len();
        buf.push(S::from("<br />"));
        if let Some(sel_writer) = selection_writer {
            sel_writer.write_selection_line_break_node(buf, cur_pos, self);
        }
    }
}

impl<S> ToRawText<S> for LineBreakNode<S>
where
    S: UnicodeString,
{
    fn to_raw_text(&self) -> S {
        "\\n".into()
    }
}

impl<S> ToPlainText<S> for LineBreakNode<S>
where
    S: UnicodeString,
{
    fn to_plain_text(&self) -> S {
        "\n".into()
    }
}

impl<S> ToTree<S> for LineBreakNode<S>
where
    S: UnicodeString,
{
    fn to_tree_display(&self, continuous_positions: Vec<usize>) -> S {
        self.tree_line(
            self.name(),
            self.handle.raw().len(),
            continuous_positions,
        )
    }
}

impl<S> ToMarkdown<S> for LineBreakNode<S>
where
    S: UnicodeString,
{
    fn fmt_markdown(
        &self,
        buffer: &mut S,
        options: &MarkdownOptions,
        _as_message: bool,
    ) -> Result<(), MarkdownError<S>> {
        if options.contains(MarkdownOptions::IGNORE_LINE_BREAK) {
            // Replace the line break by a single space.
            buffer.push(' ');
        } else {
            // A line break is a `\n` in Markdown. Two or more line breaks
            // usually generate a new block (i.e. a new paragraph). To
            // avoid that, we can prefix `\n` by a backslash. Thus:
            //
            // ```html
            // abc<br />def
            //
            // ghi<br /><br />jkl
            // ```
            //
            // maps to:
            //
            // ```md
            // abc\
            // def
            //
            // ghi\
            // \
            // jkl
            // ```
            buffer.push("\\\n");
        }

        Ok(())
    }
}
