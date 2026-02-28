// Copyright (c) 2026 Element Creations Ltd
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

//! Block-aware projection layer.
//!
//! Produces a flat list of [`BlockProjection`]s from the nested DOM tree.
//! The nested-container formatting model is preserved unchanged; this module
//! only reads the tree and exposes a flattened view suitable for rendering an
//! `NSAttributedString` on iOS without going through HTML.
//!
//! All offsets are UTF-16 code units, consistent with the rest of the
//! `ComposerModel` API surface.

use crate::dom::nodes::container_node::{ContainerNode, ContainerNodeKind};
use crate::dom::nodes::mention_node::MentionNodeKind;
use crate::dom::nodes::DomNode;
use crate::dom::unicode_string::UnicodeStrExt;
use crate::dom::{DomHandle, UnicodeString};
use crate::list_type::ListType;
use crate::InlineFormatType;

// ─── Public types ────────────────────────────────────────────────────────────

/// A block identifier is the [`DomHandle`] path of the block's container node
/// (e.g. the `<p>` or `<li>` node).
///
/// **Stability note:** `DomHandle` paths shift whenever siblings are inserted
/// or removed (structural edits).  After any structural edit consumers must
/// call `get_block_projections()` again to refresh all IDs.
pub type BlockId = DomHandle;

/// The set of formatting attributes that can appear on a text run.
/// Nested `<em><strong>` trees in the DOM are flattened into this struct.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct AttributeSet {
    pub bold: bool,
    pub italic: bool,
    pub strike_through: bool,
    pub underline: bool,
    pub inline_code: bool,
    /// Present when the run is wrapped in a link container.
    pub link_url: Option<String>,
}

/// A single logical run of content within a block.
#[derive(Clone, Debug)]
pub enum InlineRunKind {
    Text {
        text: String,
        attributes: AttributeSet,
    },
    /// An atomic @-mention.  `text_len()` in the DOM is always 1 UTF-16 code
    /// unit; the display text is carried here for the iOS renderer to use as
    /// the pill label.
    Mention {
        url: String,
        display_text: String,
    },
    /// A `<br>` — one UTF-16 code unit.
    LineBreak,
}

/// A run of content with a defined UTF-16 extent within the document.
#[derive(Clone, Debug)]
pub struct InlineRun {
    /// Handle to the corresponding leaf node in the DOM.
    pub node_handle: DomHandle,
    /// Absolute UTF-16 start offset (document-level).
    pub start_utf16: usize,
    /// Absolute UTF-16 end offset (exclusive, document-level).
    pub end_utf16: usize,
    pub kind: InlineRunKind,
}

/// The semantic kind of a block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockKind {
    Paragraph,
    Quote,
    CodeBlock,
    ListItem {
        list_type: ListType,
        /// 1-based nesting depth (top-level list = 1).
        depth: usize,
    },
    /// Root-level inline content that is not wrapped in any block element.
    Generic,
}

/// A flat projection of a single block and its inline content.
#[derive(Clone, Debug)]
pub struct BlockProjection {
    /// The `DomHandle` of the block's container node.
    pub block_id: BlockId,
    pub kind: BlockKind,
    /// Whether this block lives inside a `<blockquote>` ancestor.
    pub in_quote: bool,
    /// Absolute UTF-16 start offset of the first content code unit.
    pub start_utf16: usize,
    /// Absolute UTF-16 end offset (exclusive).  Does **not** include the
    /// inter-block separator code unit.
    pub end_utf16: usize,
    pub inline_runs: Vec<InlineRun>,
}

// ─── Internal walk helpers ────────────────────────────────────────────────────

/// Context accumulated as we recurse into structural block containers.
#[derive(Clone, Default)]
pub(crate) struct WalkContext {
    in_code_block: bool,
    in_quote: bool,
    /// 1-based nesting depth; 0 means we are not inside any list.
    list_depth: usize,
    /// The `ListType` of the innermost list, if any.
    list_type: Option<ListType>,
}

impl WalkContext {
    /// Return a new context for the children of `container`.
    fn entering<S: UnicodeString>(
        &self,
        container: &ContainerNode<S>,
    ) -> Self {
        let mut ctx = self.clone();
        match container.kind() {
            ContainerNodeKind::CodeBlock => ctx.in_code_block = true,
            ContainerNodeKind::Quote => ctx.in_quote = true,
            ContainerNodeKind::List(lt) => {
                ctx.list_depth += 1;
                ctx.list_type = Some(lt.clone());
            }
            _ => {}
        }
        ctx
    }

    /// Derive the [`BlockKind`] for a leaf block given this context.
    fn derive_kind<S: UnicodeString>(
        &self,
        container: &ContainerNode<S>,
    ) -> BlockKind {
        if self.in_code_block {
            return BlockKind::CodeBlock;
        }
        // List depth takes priority over quote — a list inside a blockquote
        // should still emit ListItem blocks so the renderer can add markers.
        if self.list_depth > 0 {
            return BlockKind::ListItem {
                list_type: self.list_type.clone().unwrap_or(ListType::Unordered),
                depth: self.list_depth,
            };
        }
        if self.in_quote {
            return BlockKind::Quote;
        }
        match container.kind() {
            ContainerNodeKind::Paragraph => BlockKind::Paragraph,
            ContainerNodeKind::Generic => BlockKind::Generic,
            // ListItem with direct inline children (no wrapping Paragraph)
            ContainerNodeKind::ListItem => BlockKind::ListItem {
                list_type: ListType::Unordered, // fallback; List parent sets this
                depth: 1,
            },
            _ => BlockKind::Paragraph,
        }
    }
}

/// Walk the direct block children of `container`, adding inter-block separator
/// code units and collecting [`BlockProjection`]s.
///
/// When the container has **no** block children but does have inline children
/// (e.g. the document root after a plain `replace_text` call with no block
/// wrapper), the container itself is treated as a single leaf block so that
/// the caller always receives at least one projection.
pub(crate) fn walk_container<S: UnicodeString>(
    container: &ContainerNode<S>,
    cursor: &mut usize,
    context: &WalkContext,
    projections: &mut Vec<BlockProjection>,
) {
    let has_block_children =
        container.children().iter().any(|ch| ch.is_block_node());

    if has_block_children && context.in_code_block {
        // Code blocks: collapse all <p> children into a single
        // BlockProjection with \n between paragraphs (matching to_html
        // behaviour which strips the <p> wrappers).
        let start = *cursor;
        let mut runs: Vec<InlineRun> = Vec::new();
        let mut first_block = true;
        for child in container.children() {
            if child.is_block_node() {
                if !first_block {
                    // Insert a newline run between paragraphs.
                    runs.push(InlineRun {
                        node_handle: container.handle().clone(),
                        start_utf16: *cursor,
                        end_utf16: *cursor + 1,
                        kind: InlineRunKind::Text {
                            text: "\n".to_owned(),
                            attributes: AttributeSet::default(),
                        },
                    });
                    *cursor += 1;
                }
                first_block = false;
                if let DomNode::Container(c) = child {
                    for inner in c.children() {
                        collect_inline_runs(
                            inner,
                            AttributeSet::default(),
                            cursor,
                            &mut runs,
                        );
                    }
                }
            }
        }
        merge_adjacent_runs(&mut runs);
        projections.push(BlockProjection {
            block_id: container.handle().clone(),
            kind: BlockKind::CodeBlock,
            in_quote: context.in_quote,
            start_utf16: start,
            end_utf16: *cursor,
            inline_runs: runs,
        });
    } else if has_block_children {
        let mut first_block = true;
        for child in container.children() {
            if child.is_block_node() {
                if !first_block {
                    *cursor += 1; // inter-block separator code unit
                }
                first_block = false;
                walk_block_node(child, cursor, context, projections);
            }
            // Inline children mixed with block children should not occur per
            // the DOM invariant, but we skip them gracefully.
        }
    } else if !container.children().is_empty() {
        // Container has only inline children — treat it as a single leaf
        // content block (e.g. document root with no <p> wrapper).
        let start = *cursor;
        let kind = context.derive_kind(container);
        let mut runs: Vec<InlineRun> = Vec::new();
        for child in container.children() {
            collect_inline_runs(
                child,
                AttributeSet::default(),
                cursor,
                &mut runs,
            );
        }
        merge_adjacent_runs(&mut runs);
        projections.push(BlockProjection {
            block_id: container.handle().clone(),
            kind,
            in_quote: context.in_quote,
            start_utf16: start,
            end_utf16: *cursor,
            inline_runs: runs,
        });
    }
}

/// Recurse into a single block node.  If it has no block children it is a leaf
/// and we emit a [`BlockProjection`].  Otherwise we recurse deeper.
fn walk_block_node<S: UnicodeString>(
    node: &DomNode<S>,
    cursor: &mut usize,
    context: &WalkContext,
    projections: &mut Vec<BlockProjection>,
) {
    let DomNode::Container(c) = node else {
        // Leaf nodes (Text, Mention, LineBreak) cannot be block nodes.
        return;
    };

    let has_block_children =
        c.children().iter().any(|ch| ch.is_block_node());

    if has_block_children {
        // Structural block — descend with updated context.
        let child_ctx = context.entering(c);
        walk_container(c, cursor, &child_ctx, projections);
    } else {
        // Leaf content block — emit a projection.
        let start = *cursor;
        let kind = context.derive_kind(c);
        let mut runs: Vec<InlineRun> = Vec::new();
        for child in c.children() {
            collect_inline_runs(
                child,
                AttributeSet::default(),
                cursor,
                &mut runs,
            );
        }
        merge_adjacent_runs(&mut runs);
        projections.push(BlockProjection {
            block_id: c.handle().clone(),
            kind,
            in_quote: context.in_quote,
            start_utf16: start,
            end_utf16: *cursor,
            inline_runs: runs,
        });
    }
}

/// Recursively collect inline runs from `node`, accumulating formatting
/// attributes from ancestor containers into `inherited`.
fn collect_inline_runs<S: UnicodeString>(
    node: &DomNode<S>,
    inherited: AttributeSet,
    cursor: &mut usize,
    runs: &mut Vec<InlineRun>,
) {
    match node {
        DomNode::Text(t) => {
            let len = t.data().len();
            runs.push(InlineRun {
                node_handle: t.handle().clone(),
                start_utf16: *cursor,
                end_utf16: *cursor + len,
                kind: InlineRunKind::Text {
                    text: t.data().to_string(),
                    attributes: inherited,
                },
            });
            *cursor += len;
        }
        DomNode::Container(c) => {
            let mut attrs = inherited.clone();
            match c.kind() {
                ContainerNodeKind::Formatting(fmt) => {
                    apply_format(&mut attrs, fmt);
                }
                ContainerNodeKind::Link(url) => {
                    attrs.link_url = Some(url.to_string());
                }
                // Other container kinds shouldn't appear inside inline content.
                _ => {}
            }
            for child in c.children() {
                collect_inline_runs(child, attrs.clone(), cursor, runs);
            }
        }
        DomNode::Mention(m) => {
            let (url, display) = match m.kind() {
                MentionNodeKind::MatrixUri { mention } => (
                    mention.uri().to_string(),
                    m.display_text().to_string(),
                ),
                MentionNodeKind::AtRoom => (
                    "@room".to_string(),
                    m.display_text().to_string(),
                ),
            };
            runs.push(InlineRun {
                node_handle: m.handle().clone(),
                start_utf16: *cursor,
                end_utf16: *cursor + 1,
                kind: InlineRunKind::Mention {
                    url,
                    display_text: display,
                },
            });
            *cursor += 1; // mentions are always 1 code unit
        }
        DomNode::LineBreak(lb) => {
            runs.push(InlineRun {
                node_handle: lb.handle().clone(),
                start_utf16: *cursor,
                end_utf16: *cursor + 1,
                kind: InlineRunKind::LineBreak,
            });
            *cursor += 1;
        }
    }
}

/// Apply a single `InlineFormatType` on top of an existing `AttributeSet`.
fn apply_format(attrs: &mut AttributeSet, fmt: &InlineFormatType) {
    match fmt {
        InlineFormatType::Bold => attrs.bold = true,
        InlineFormatType::Italic => attrs.italic = true,
        InlineFormatType::StrikeThrough => attrs.strike_through = true,
        InlineFormatType::Underline => attrs.underline = true,
        InlineFormatType::InlineCode => attrs.inline_code = true,
    }
}

/// Merge adjacent [`InlineRun`]s whose `InlineRunKind` is `Text` and whose
/// `AttributeSet`s are identical.  This satisfies invariant 5.
fn merge_adjacent_runs(runs: &mut Vec<InlineRun>) {
    let mut i = 0;
    while i + 1 < runs.len() {
        let can_merge = match (&runs[i].kind, &runs[i + 1].kind) {
            (
                InlineRunKind::Text {
                    attributes: a1, ..
                },
                InlineRunKind::Text {
                    attributes: a2, ..
                },
            ) => a1 == a2,
            _ => false,
        };
        if can_merge {
            let next = runs.remove(i + 1);
            let InlineRunKind::Text { text: next_text, .. } = next.kind else {
                unreachable!()
            };
            let InlineRunKind::Text { text, .. } = &mut runs[i].kind else {
                unreachable!()
            };
            text.push_str(&next_text);
            runs[i].end_utf16 = next.end_utf16;
        } else {
            i += 1;
        }
    }
}
