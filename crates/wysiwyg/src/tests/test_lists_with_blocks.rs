// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::tests::testutils_composer_model::{cm, tx};

// This file defines specific tests on how lists behave in combination to
// some other block nodes such as quote and code blocks. At some point, some
// of these behaviours might change with updates such as e.g. quotes
// available inside list items.

#[test]
fn create_list_inside_quote() {
    let mut model = cm("<blockquote><p>a|</p></blockquote>");
    model.ordered_list();
    assert_eq!(tx(&model), "<blockquote><ol><li>a|</li></ol></blockquote>")
}

#[test]
fn create_list_inside_quote_with_multiple_paragraphs() {
    let mut model =
        cm("<blockquote><p>{a</p><p>b</p><p>c}|</p><p>d</p></blockquote>");
    model.ordered_list();
    assert_eq!(
        tx(&model),
        "<blockquote><ol><li>{a</li><li>b</li><li>c}|</li></ol><p>d</p></blockquote>"
    )
}

#[test]
fn create_list_with_selected_paragraph_and_quote() {
    let mut model = cm("<p>{text</p><blockquote><p>quote}|</p></blockquote>");
    model.unordered_list();
    assert_eq!(tx(&model), "<ul><li>{text</li><li>quote}|</li></ul>")
}

#[test]
fn create_list_with_selected_paragraph_and_quote_with_multiple_nested_paragraphs(
) {
    let mut model = cm(
        "<p>{text</p><blockquote><p>quote</p><p>more quote}|</p></blockquote>",
    );
    model.unordered_list();
    assert_eq!(
        tx(&model),
        "<ul><li>{text</li><li>quote</li><li>more quote}|</li></ul>"
    )
}

#[test]
fn create_list_with_selected_paragraph_and_codeblock() {
    let mut model = cm("<p>{text</p><pre><code>some code}|</code></pre>");
    model.unordered_list();
    assert_eq!(tx(&model), "<ul><li>{text</li><li>some code}|</li></ul>")
}

#[test]
fn create_list_with_selected_paragraph_and_codeblock_with_multiple_lines() {
    let mut model = cm("<p>{text</p><pre><code>code\nmore code}|</code></pre>");
    model.unordered_list();
    assert_eq!(
        tx(&model),
        "<ul><li>{text</li><li>code</li><li>more code}|</li></ul>"
    )
}

#[test]
fn create_list_with_selected_paragraph_quote_and_code_block() {
    let mut model = cm("<blockquote><p>{quote</p><p>more quote</p></blockquote><p>text</p><pre><code>code\nmore code}|</code></pre>");
    model.ordered_list();
    assert_eq!(
        tx(&model),
        "<ol><li>{quote</li><li>more quote</li><li>text</li><li>code</li><li>more code}|</li></ol>"
    )
}
