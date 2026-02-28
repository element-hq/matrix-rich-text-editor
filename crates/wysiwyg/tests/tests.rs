// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use widestring::Utf16String;
use wysiwyg::{
    AttributeSet, BlockKind, BlockProjection, ComposerModel, InlineRunKind,
    Location, TextUpdate,
};

#[test]
fn can_instantiate_a_model_and_call_methods() {
    let mut model = ComposerModel::new();
    model.replace_text(Utf16String::from_str("foo"));
    model.select(Location::from(1), Location::from(2));

    let update = model.bold();

    if let TextUpdate::ReplaceAll(r) = update.text_update {
        assert_eq!(r.replacement_html.to_string(), "f<strong>o</strong>o");
        assert_eq!(r.start, 1);
        assert_eq!(r.end, 2);
    } else {
        panic!("Expected to receive a ReplaceAll response");
    }
}

fn model_from_html(html: &str) -> ComposerModel<Utf16String> {
    ComposerModel::from_html(html, 0, 0)
}

fn projections(model: &ComposerModel<Utf16String>) -> Vec<BlockProjection> {
    model.state.dom.get_block_projections()
}

#[test]
fn single_paragraph_projection() {
    let model = model_from_html("<p>hello</p>");
    let blocks = projections(&model);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].kind, BlockKind::Paragraph);
    assert_eq!(blocks[0].start_utf16, 0);
    assert_eq!(blocks[0].end_utf16, 5);
    assert_eq!(blocks[0].inline_runs.len(), 1);
    let InlineRunKind::Text { ref text, ref attributes } =
        blocks[0].inline_runs[0].kind
    else {
        panic!("expected Text run");
    };
    assert_eq!(text, "hello");
    assert_eq!(*attributes, AttributeSet::default());
}

#[test]
fn two_paragraphs_contiguous_offsets() {
    let model = model_from_html("<p>ab</p><p>cd</p>");
    let blocks = projections(&model);
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].start_utf16, 0);
    assert_eq!(blocks[0].end_utf16, 2);
    assert_eq!(blocks[1].start_utf16, 3);
    assert_eq!(blocks[1].end_utf16, 5);
}

#[test]
fn nested_bold_italic_both_flags_set() {
    let model = model_from_html("<p><em><strong>text</strong></em></p>");
    let blocks = projections(&model);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].inline_runs.len(), 1);
    let InlineRunKind::Text { ref attributes, .. } =
        blocks[0].inline_runs[0].kind
    else {
        panic!("expected Text run");
    };
    assert!(attributes.bold);
    assert!(attributes.italic);
}

#[test]
fn adjacent_runs_with_same_attrs_are_merged() {
    let model =
        model_from_html("<p><strong>foo</strong><strong>bar</strong></p>");
    let blocks = projections(&model);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].inline_runs.len(), 1);
    let InlineRunKind::Text { ref text, ref attributes } =
        blocks[0].inline_runs[0].kind
    else {
        panic!("expected Text run");
    };
    assert_eq!(text, "foobar");
    assert!(attributes.bold);
}

#[test]
fn adjacent_runs_with_different_attrs_not_merged() {
    let model =
        model_from_html("<p><strong>foo</strong><em>bar</em></p>");
    let blocks = projections(&model);
    assert_eq!(blocks[0].inline_runs.len(), 2);
}

#[test]
fn block_at_offset_paragraph() {
    let model = model_from_html("<p>abc</p><p>def</p>");
    let dom = &model.state.dom;
    let blocks = dom.get_block_projections();
    assert_eq!(blocks[0].start_utf16, 0);
    assert_eq!(blocks[0].end_utf16, 3);
    assert_eq!(blocks[1].start_utf16, 4);
    assert_eq!(blocks[1].end_utf16, 7);
    let id0 = blocks[0].block_id.clone();
    let id1 = blocks[1].block_id.clone();
    assert_eq!(dom.block_at_offset(0), Some(id0.clone()));
    assert_eq!(dom.block_at_offset(2), Some(id0.clone()));
    assert_eq!(dom.block_at_offset(3), Some(id0.clone()));
    assert_eq!(dom.block_at_offset(4), Some(id1.clone()));
    assert_eq!(dom.block_at_offset(6), Some(id1.clone()));
    assert_eq!(dom.block_at_offset(7), Some(id1.clone()));
}

#[test]
fn br_splits_into_two_paragraphs() {
    let model = model_from_html("<p>a<br />b</p>");
    let blocks = projections(&model);
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].end_utf16, 1);
    assert_eq!(blocks[1].start_utf16, 2);
    assert_eq!(blocks[1].end_utf16, 3);
}

#[test]
fn code_block_kind() {
    let model = model_from_html("<pre><code>fn main() {}</code></pre>");
    let blocks = projections(&model);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].kind, BlockKind::CodeBlock);
}

#[test]
fn code_block_multiline_produces_single_block() {
    // A multi-line <pre><code> collapses into a single CodeBlock with \n in the text.
    let model = model_from_html(
        "<pre><code>if snapshot {\n\treturn true\n}</code></pre>"
    );
    let blocks = projections(&model);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].kind, BlockKind::CodeBlock);
    // All text including newlines is in one block's runs.
    let all_text: String = blocks[0]
        .inline_runs
        .iter()
        .filter_map(|r| {
            if let InlineRunKind::Text { text, .. } = &r.kind {
                Some(text.as_str())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(all_text, "if snapshot {\n\treturn true\n}");
}

#[test]
fn quote_block_kind() {
    let model = model_from_html("<blockquote><p>quoted</p></blockquote>");
    let blocks = projections(&model);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].kind, BlockKind::Quote);
}

#[test]
fn unordered_list_items() {
    use wysiwyg::ListType;
    let model = model_from_html("<ul><li><p>a</p></li><li><p>b</p></li></ul>");
    let blocks = projections(&model);
    assert_eq!(blocks.len(), 2);
    for b in &blocks {
        assert!(
            matches!(&b.kind, BlockKind::ListItem { list_type, depth } if *list_type == ListType::Unordered && *depth == 1)
        );
    }
}

#[test]
fn ordered_list_items() {
    use wysiwyg::ListType;
    let model = model_from_html("<ol><li><p>x</p></li><li><p>y</p></li></ol>");
    let blocks = projections(&model);
    assert_eq!(blocks.len(), 2);
    for b in &blocks {
        assert!(
            matches!(&b.kind, BlockKind::ListItem { list_type, depth } if *list_type == ListType::Ordered && *depth == 1)
        );
    }
}

#[test]
fn list_item_offsets_contiguous() {
    let model = model_from_html("<ul><li><p>ab</p></li><li><p>cd</p></li></ul>");
    let blocks = projections(&model);
    assert_eq!(blocks[0].start_utf16, 0);
    assert_eq!(blocks[0].end_utf16, 2);
    assert_eq!(blocks[1].start_utf16, 3);
    assert_eq!(blocks[1].end_utf16, 5);
}

#[test]
fn projection_offsets_after_structural_edit_enter() {
    let mut model = model_from_html("<p>ab</p>");
    model.select(Location::from(1), Location::from(1));
    model.enter();
    let blocks = projections(&model);
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].start_utf16, 0);
    assert_eq!(blocks[0].end_utf16, 1);
    assert_eq!(blocks[1].start_utf16, 2);
    assert_eq!(blocks[1].end_utf16, 3);
}

#[test]
fn link_url_in_attribute_set() {
    let model = model_from_html(
        r#"<p><a href="https://example.com">link</a></p>"#,
    );
    let blocks = projections(&model);
    let InlineRunKind::Text { ref attributes, .. } =
        blocks[0].inline_runs[0].kind
    else {
        panic!("expected Text run");
    };
    assert_eq!(
        attributes.link_url.as_deref(),
        Some("https://example.com")
    );
}

#[test]
fn inline_only_root_produces_single_block() {
    // When text is entered via replace_text (no <p> wrapper), the document
    // root has only inline children.  walk_container must still produce a
    // projection.
    let mut model = ComposerModel::new();
    model.replace_text(Utf16String::from_str("hello"));
    let blocks = projections(&model);
    assert_eq!(blocks.len(), 1, "expected 1 block for inline-only root");
    assert_eq!(blocks[0].kind, BlockKind::Generic);
    assert_eq!(blocks[0].start_utf16, 0);
    assert_eq!(blocks[0].end_utf16, 5);
    assert_eq!(blocks[0].inline_runs.len(), 1);
    let InlineRunKind::Text { ref text, .. } = blocks[0].inline_runs[0].kind
    else {
        panic!("expected Text run");
    };
    assert_eq!(text, "hello");
}

#[test]
fn inline_only_root_with_bold() {
    // Bold applied via selection on a root with no block wrapper.
    let mut model = ComposerModel::new();
    model.replace_text(Utf16String::from_str("This is bold text"));
    model.select(Location::from(8), Location::from(12));
    model.bold();
    let blocks = projections(&model);
    assert_eq!(blocks.len(), 1);
    // Three runs: plain "This is ", bold "bold", plain " text"
    assert_eq!(blocks[0].inline_runs.len(), 3);
    let InlineRunKind::Text { ref text, ref attributes } =
        blocks[0].inline_runs[1].kind
    else {
        panic!("expected Text run");
    };
    assert_eq!(text, "bold");
    assert!(attributes.bold);
}

#[test]
fn total_projection_length_matches_text_len() {
    let model = model_from_html(
        "<p>hello</p><p><strong>world</strong></p><ul><li><p>item</p></li></ul>",
    );
    let blocks = projections(&model);
    let dom_len = model.state.dom.text_len();
    let last = blocks.last().unwrap();
    assert_eq!(last.end_utf16, dom_len);
    assert_eq!(blocks[0].start_utf16, 0);
    assert_eq!(blocks[0].end_utf16, 5);
    assert_eq!(blocks[1].start_utf16, 6);
    assert_eq!(blocks[1].end_utf16, 11);
    assert_eq!(blocks[2].start_utf16, 12);
    assert_eq!(blocks[2].end_utf16, 16);
}
