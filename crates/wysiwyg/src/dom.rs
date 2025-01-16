// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

pub mod action_list;
pub mod dom_block_nodes;
pub mod dom_creation_error;
pub mod dom_handle;
pub mod dom_invariants;
pub mod dom_list_methods;
pub mod dom_methods;
pub mod dom_struct;
pub mod find_extended_range;
pub mod find_range;
pub mod find_result;
pub mod insert_node_at_cursor;
pub mod insert_parent;
pub mod iter;
pub mod join_nodes;
pub mod nodes;
pub mod parser;
pub mod range;
pub mod to_html;
pub mod to_markdown;
pub mod to_plain_text;
pub mod to_raw_text;
pub mod to_tree;
pub mod unicode_string;

pub use dom_creation_error::DomCreationError;
pub use dom_creation_error::HtmlParseError;
pub use dom_creation_error::MarkdownParseError;
pub use dom_handle::DomHandle;
pub use dom_struct::Dom;
pub use find_result::FindResult;
pub use range::DomLocation;
pub use range::Range;
pub use to_html::ToHtml;
pub use to_markdown::{MarkdownError, ToMarkdown};
pub use to_raw_text::ToRawText;
pub use to_tree::ToTree;
pub use unicode_string::UnicodeString;
