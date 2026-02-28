// Copyright (c) 2026 Element Creations Ltd
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use wysiwyg::{AttributeSet, BlockKind, BlockProjection, DomHandle, InlineRun, InlineRunKind, ListType};

/// Serialise a `DomHandle` to a compact comma-separated index string, e.g. "0,2,1".
pub(crate) fn handle_to_string(handle: &DomHandle) -> String {
    handle
        .raw()
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct FfiAttributeSet {
    pub bold: bool,
    pub italic: bool,
    pub strike_through: bool,
    pub underline: bool,
    pub inline_code: bool,
    pub link_url: Option<String>,
}

#[derive(uniffi::Enum, Clone, Debug)]
pub enum FfiInlineRunKind {
    Text { text: String, attributes: FfiAttributeSet },
    Mention { url: String, display_text: String },
    LineBreak,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct FfiInlineRun {
    pub node_id: String,
    pub start_utf16: u32,
    pub end_utf16: u32,
    pub kind: FfiInlineRunKind,
}

#[derive(uniffi::Enum, Clone, Debug)]
pub enum FfiBlockKind {
    Paragraph,
    Quote,
    CodeBlock,
    ListItemOrdered { depth: u32 },
    ListItemUnordered { depth: u32 },
    Generic,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct FfiBlockProjection {
    pub block_id: String,
    pub kind: FfiBlockKind,
    /// Whether this block lives inside a `<blockquote>` ancestor.
    pub in_quote: bool,
    pub start_utf16: u32,
    pub end_utf16: u32,
    pub inline_runs: Vec<FfiInlineRun>,
}

// ─── Conversions ─────────────────────────────────────────────────────────────

impl From<&AttributeSet> for FfiAttributeSet {
    fn from(a: &AttributeSet) -> Self {
        Self {
            bold: a.bold,
            italic: a.italic,
            strike_through: a.strike_through,
            underline: a.underline,
            inline_code: a.inline_code,
            link_url: a.link_url.clone(),
        }
    }
}

impl From<&InlineRunKind> for FfiInlineRunKind {
    fn from(k: &InlineRunKind) -> Self {
        match k {
            InlineRunKind::Text { text, attributes } => Self::Text {
                text: text.clone(),
                attributes: attributes.into(),
            },
            InlineRunKind::Mention { url, display_text } => Self::Mention {
                url: url.clone(),
                display_text: display_text.clone(),
            },
            InlineRunKind::LineBreak => Self::LineBreak,
        }
    }
}

impl From<&InlineRun> for FfiInlineRun {
    fn from(r: &InlineRun) -> Self {
        Self {
            node_id: handle_to_string(&r.node_handle),
            start_utf16: r.start_utf16 as u32,
            end_utf16: r.end_utf16 as u32,
            kind: (&r.kind).into(),
        }
    }
}

impl From<&BlockKind> for FfiBlockKind {
    fn from(k: &BlockKind) -> Self {
        match k {
            BlockKind::Paragraph => Self::Paragraph,
            BlockKind::Quote => Self::Quote,
            BlockKind::CodeBlock => Self::CodeBlock,
            BlockKind::ListItem { list_type, depth } => match list_type {
                ListType::Ordered => Self::ListItemOrdered { depth: *depth as u32 },
                ListType::Unordered => Self::ListItemUnordered { depth: *depth as u32 },
            },
            BlockKind::Generic => Self::Generic,
        }
    }
}

impl From<&BlockProjection> for FfiBlockProjection {
    fn from(b: &BlockProjection) -> Self {
        Self {
            block_id: handle_to_string(&b.block_id),
            kind: (&b.kind).into(),
            in_quote: b.in_quote,
            start_utf16: b.start_utf16 as u32,
            end_utf16: b.end_utf16 as u32,
            inline_runs: b.inline_runs.iter().map(FfiInlineRun::from).collect(),
        }
    }
}
