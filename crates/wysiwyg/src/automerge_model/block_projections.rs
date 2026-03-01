// Copyright 2026 The Matrix.org Foundation C.I.C.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Builds a flat `Vec<BlockProjection>` from Automerge spans.
//!
//! This is the Automerge counterpart of the DOM-based
//! `get_block_projections()` in `block_projection.rs`.  The iOS
//! `ProjectionRenderer` consumes this to build an `NSAttributedString`
//! without going through HTML.

use automerge::iter::Span;
use automerge::ReadDoc;

use super::block_ops::block_type;
use super::AutomergeModel;
use crate::block_projection::{
    AttributeSet, BlockKind, BlockProjection, InlineRun, InlineRunKind,
};
use crate::dom::DomHandle;
use crate::ListType;

/// Extract the block type string from a `Span::Block` map.
fn block_type_str(map: &automerge::hydrate::Map) -> &str {
    map.get("type")
        .and_then(|v| {
            if let automerge::hydrate::Value::Scalar(
                automerge::ScalarValue::Str(s),
            ) = v
            {
                Some(s.as_str())
            } else {
                None
            }
        })
        .unwrap_or(block_type::PARAGRAPH)
}

/// Convert an Automerge block-type string into a `BlockKind`.
fn block_kind(btype: &str) -> BlockKind {
    match btype {
        block_type::ORDERED_LIST_ITEM => BlockKind::ListItem {
            list_type: ListType::Ordered,
            depth: 1,
        },
        block_type::UNORDERED_LIST_ITEM => BlockKind::ListItem {
            list_type: ListType::Unordered,
            depth: 1,
        },
        block_type::CODE_BLOCK => BlockKind::CodeBlock,
        block_type::QUOTE => BlockKind::Quote,
        _ => BlockKind::Paragraph,
    }
}

/// Build an `AttributeSet` from Automerge marks.
fn marks_to_attr_set(
    marks: Option<&std::sync::Arc<automerge::marks::MarkSet>>,
) -> (AttributeSet, Option<String>) {
    let mut attrs = AttributeSet::default();
    let mut mention_url: Option<String> = None;

    if let Some(mark_set) = marks {
        for (name, value) in mark_set.iter() {
            if matches!(value, automerge::ScalarValue::Null) {
                continue;
            }
            match name {
                "bold" => attrs.bold = true,
                "italic" => attrs.italic = true,
                "strikethrough" => attrs.strike_through = true,
                "underline" => attrs.underline = true,
                "inline_code" => attrs.inline_code = true,
                "link" => {
                    if let Some(url) = value.to_str() {
                        attrs.link_url = Some(url.to_string());
                    }
                }
                "mention" => {
                    if let Some(url) = value.to_str() {
                        mention_url = Some(url.to_string());
                    }
                }
                _ => {}
            }
        }
    }
    (attrs, mention_url)
}

impl AutomergeModel {
    /// Produce a flat list of block projections from the Automerge document.
    ///
    /// Each block corresponds to a block marker in the text sequence.
    /// Content before the first block marker (if any) is wrapped in a
    /// synthetic `Paragraph` block.
    ///
    /// UTF-16 code-unit offsets match the platform text API convention.
    pub fn get_block_projections(&self) -> Vec<BlockProjection> {
        let spans: Vec<Span> = match self.doc.spans(&self.text_id) {
            Ok(s) => s.collect(),
            Err(_) => return vec![],
        };

        let mut projections: Vec<BlockProjection> = Vec::new();
        // Current UTF-16 offset into the document.
        let mut utf16_offset: usize = 0;
        // Index of the current block (for synthetic DomHandle IDs).
        let mut block_idx: usize = 0;
        // Index of the current inline run within the block.
        let mut run_idx: usize = 0;
        // The runs collected for the current block so far.
        let mut current_runs: Vec<InlineRun> = Vec::new();
        // The kind of the current block.
        let mut current_kind: BlockKind = BlockKind::Paragraph;
        // Whether the current block is inside a quote.
        let mut current_in_quote = false;
        // UTF-16 start of the current block's content.
        let mut block_start_utf16: usize = 0;

        for span in &spans {
            match span {
                Span::Block(block_map) => {
                    // Finish any previous block.
                    let proj = BlockProjection {
                        block_id: DomHandle::from_raw(vec![block_idx]),
                        kind: current_kind.clone(),
                        in_quote: current_in_quote,
                        start_utf16: block_start_utf16,
                        end_utf16: utf16_offset,
                        inline_runs: std::mem::take(&mut current_runs),
                    };
                    // Only emit a block if it's not a completely empty
                    // leading implicit paragraph (no content before any
                    // explicit block marker).
                    if !projections.is_empty()
                        || !proj.inline_runs.is_empty()
                    {
                        projections.push(proj);
                    }

                    block_idx += 1;
                    run_idx = 0;

                    let btype = block_type_str(block_map);
                    current_kind = block_kind(btype);
                    current_in_quote = matches!(current_kind, BlockKind::Quote);

                    // Block markers occupy 1 character (\u{fffc}) which we
                    // do NOT count toward visible UTF-16 offsets — the
                    // platform text system never sees the marker character.
                    // However we track it for Automerge-internal purposes.
                    // The block content starts at the current utf16_offset
                    // (the marker itself is invisible to the platform).
                    block_start_utf16 = utf16_offset;
                }
                Span::Text { text, marks } => {
                    let text_utf16_len =
                        text.encode_utf16().count();

                    let (attrs, mention_url) = marks_to_attr_set(
                        marks.as_ref().map(|m| m as &std::sync::Arc<_>),
                    );

                    let kind = if let Some(url) = mention_url {
                        InlineRunKind::Mention {
                            url,
                            display_text: text.clone(),
                        }
                    } else {
                        InlineRunKind::Text {
                            text: text.clone(),
                            attributes: attrs,
                        }
                    };

                    current_runs.push(InlineRun {
                        node_handle: DomHandle::from_raw(vec![
                            block_idx, run_idx,
                        ]),
                        start_utf16: utf16_offset,
                        end_utf16: utf16_offset + text_utf16_len,
                        kind,
                    });

                    utf16_offset += text_utf16_len;
                    run_idx += 1;
                }
            }
        }

        // Close the final block.
        let proj = BlockProjection {
            block_id: DomHandle::from_raw(vec![block_idx]),
            kind: current_kind,
            in_quote: current_in_quote,
            start_utf16: block_start_utf16,
            end_utf16: utf16_offset,
            inline_runs: std::mem::take(&mut current_runs),
        };
        if !projections.is_empty() || !proj.inline_runs.is_empty() {
            projections.push(proj);
        }

        // If the document is completely empty, return a single empty
        // paragraph so the renderer still has one block to display.
        if projections.is_empty() {
            projections.push(BlockProjection {
                block_id: DomHandle::from_raw(vec![0]),
                kind: BlockKind::Paragraph,
                in_quote: false,
                start_utf16: 0,
                end_utf16: 0,
                inline_runs: vec![],
            });
        }

        projections
    }
}

#[cfg(test)]
mod tests {
    use crate::block_projection::{BlockKind, InlineRunKind};
    use crate::AutomergeModel;
    use crate::ListType;

    fn new_model() -> AutomergeModel {
        AutomergeModel::new()
    }

    fn model_with_text(text: &str) -> AutomergeModel {
        let mut m = AutomergeModel::new();
        m.replace_text(text);
        m
    }

    // ----- empty document -----

    #[test]
    fn empty_doc_has_one_paragraph_block() {
        let m = new_model();
        let projs = m.get_block_projections();
        assert_eq!(projs.len(), 1);
        assert_eq!(projs[0].kind, BlockKind::Paragraph);
        assert!(projs[0].inline_runs.is_empty());
    }

    // ----- plain text (no block markers) -----

    #[test]
    fn plain_text_single_block() {
        let m = model_with_text("hello world");
        let projs = m.get_block_projections();
        assert_eq!(projs.len(), 1);
        assert_eq!(projs[0].kind, BlockKind::Paragraph);
        assert_eq!(projs[0].start_utf16, 0);
        assert_eq!(projs[0].end_utf16, 11);
        assert_eq!(projs[0].inline_runs.len(), 1);
        match &projs[0].inline_runs[0].kind {
            InlineRunKind::Text { text, .. } => assert_eq!(text, "hello world"),
            _ => panic!("expected Text run"),
        }
    }

    // ----- paragraphs -----

    #[test]
    fn enter_creates_two_blocks() {
        let mut m = model_with_text("ab");
        m.select(2, 2);
        m.enter();
        let projs = m.get_block_projections();
        // We expect 2 blocks: first paragraph "ab", second paragraph empty
        assert!(projs.len() >= 2, "expected at least 2 blocks, got {}", projs.len());
    }

    // ----- ordered list -----

    #[test]
    fn ordered_list_block_kind() {
        let mut m = new_model();
        m.ordered_list();
        m.replace_text("item");
        let projs = m.get_block_projections();
        // Should have at least one block with ListItem kind
        let list_block = projs.iter().find(|p| {
            matches!(p.kind, BlockKind::ListItem { list_type: ListType::Ordered, .. })
        });
        assert!(list_block.is_some(), "expected an ordered list block");
    }

    // ----- unordered list -----

    #[test]
    fn unordered_list_block_kind() {
        let mut m = new_model();
        m.unordered_list();
        m.replace_text("item");
        let projs = m.get_block_projections();
        let list_block = projs.iter().find(|p| {
            matches!(p.kind, BlockKind::ListItem { list_type: ListType::Unordered, .. })
        });
        assert!(list_block.is_some(), "expected an unordered list block");
    }

    // ----- code block -----

    #[test]
    fn code_block_kind() {
        let mut m = new_model();
        m.code_block();
        m.replace_text("fn main()");
        let projs = m.get_block_projections();
        let code = projs.iter().find(|p| p.kind == BlockKind::CodeBlock);
        assert!(code.is_some(), "expected a code block");
    }

    // ----- quote -----

    #[test]
    fn quote_block_kind() {
        let mut m = new_model();
        m.quote();
        m.replace_text("quoted text");
        let projs = m.get_block_projections();
        let q = projs.iter().find(|p| p.kind == BlockKind::Quote);
        assert!(q.is_some(), "expected a quote block");
        assert!(q.unwrap().in_quote, "quote block should have in_quote=true");
    }

    // ----- inline formatting on runs -----

    #[test]
    fn bold_text_shows_bold_attribute() {
        let mut m = new_model();
        m.replace_text("hello");
        m.select(0, 5);
        m.bold();
        let projs = m.get_block_projections();
        assert!(!projs.is_empty());
        let run = &projs[0].inline_runs[0];
        match &run.kind {
            InlineRunKind::Text { attributes, .. } => {
                assert!(attributes.bold, "expected bold=true");
            }
            _ => panic!("expected Text run"),
        }
    }

    #[test]
    fn italic_text_shows_italic_attribute() {
        let mut m = new_model();
        m.replace_text("hello");
        m.select(0, 5);
        m.italic();
        let projs = m.get_block_projections();
        let run = &projs[0].inline_runs[0];
        match &run.kind {
            InlineRunKind::Text { attributes, .. } => {
                assert!(attributes.italic, "expected italic=true");
            }
            _ => panic!("expected Text run"),
        }
    }

    // ----- links -----

    #[test]
    fn link_appears_in_attribute_set() {
        let mut m = new_model();
        m.replace_text("click");
        m.select(0, 5);
        m.set_link("https://example.com", &[]);
        let projs = m.get_block_projections();
        let run = &projs[0].inline_runs[0];
        match &run.kind {
            InlineRunKind::Text { attributes, .. } => {
                assert_eq!(
                    attributes.link_url.as_deref(),
                    Some("https://example.com")
                );
            }
            _ => panic!("expected Text run"),
        }
    }

    // ----- mentions -----

    #[test]
    fn mention_inline_run() {
        let mut m = new_model();
        m.insert_mention(
            "https://matrix.to/#/@alice:matrix.org",
            "Alice",
            &[],
        );
        let projs = m.get_block_projections();
        let has_mention = projs.iter().any(|p| {
            p.inline_runs.iter().any(|r| {
                matches!(&r.kind, InlineRunKind::Mention { url, .. } if url.contains("@alice"))
            })
        });
        assert!(has_mention, "expected a Mention inline run");
    }

    // ----- UTF-16 offsets -----

    #[test]
    fn utf16_offsets_for_emoji() {
        // 🦀 is U+1F980 → 2 UTF-16 code units
        let m = model_with_text("🦀ab");
        let projs = m.get_block_projections();
        assert_eq!(projs[0].start_utf16, 0);
        // 2 (emoji) + 1 (a) + 1 (b) = 4
        assert_eq!(projs[0].end_utf16, 4);
    }

    // ----- multiple formatted runs -----

    #[test]
    fn mixed_formatting_creates_multiple_runs() {
        let mut m = new_model();
        m.replace_text("plain");
        m.select(5, 5);
        m.bold();
        m.replace_text("bold");
        m.select(0, 0); // reset cursor
        let projs = m.get_block_projections();
        // There should be at least 2 inline runs: unformatted + bold
        assert!(
            projs[0].inline_runs.len() >= 2,
            "expected at least 2 runs, got {}",
            projs[0].inline_runs.len()
        );
    }
}
