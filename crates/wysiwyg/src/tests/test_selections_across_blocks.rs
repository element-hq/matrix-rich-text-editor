// Copyright 2026 New Vector Ltd.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

//! Replacing a selection spanning several block nodes, either by typing or
//! pressing enter, should behave as deleting the selection first.

use widestring::Utf16String;

use crate::tests::testutils_composer_model::{cm, tx};

// region: enter

#[test]
fn enter_with_selection_across_paragraphs() {
    let mut model = cm("<p>a{bc</p><p>de}|f</p>");
    model.enter();
    assert_eq!(tx(&model), "<p>a</p><p>|f</p>");
}

#[test]
fn enter_with_selection_of_a_whole_paragraph() {
    let mut model = cm("<p>{abc}|</p><p>def</p>");
    model.enter();
    assert_eq!(tx(&model), "<p>&nbsp;</p><p>&nbsp;|</p><p>def</p>");
}

#[test]
fn enter_with_selection_of_a_whole_paragraph_and_the_start_of_the_next_one() {
    let mut model = cm("<p>{abc</p><p>d}|ef</p>");
    model.enter();
    assert_eq!(tx(&model), "<p>&nbsp;</p><p>|ef</p>");
}

#[test]
fn enter_with_selection_ending_at_the_end_of_the_content() {
    let mut model = cm("<p>abc</p><p>{def}|</p>");
    model.enter();
    assert_eq!(tx(&model), "<p>abc</p><p>&nbsp;</p><p>&nbsp;|</p>");
}

#[test]
fn enter_with_selection_across_list_items() {
    let mut model = cm("<ul><li>o{ne</li><li>tw}|o</li></ul>");
    model.enter();
    assert_eq!(tx(&model), "<ul><li>o</li><li>|o</li></ul>");
}

#[test]
fn enter_with_selection_of_a_line_break_between_paragraphs() {
    let mut model = cm("<p>abc{</p><p>}|def</p>");
    model.enter();
    assert_eq!(tx(&model), "<p>abc</p><p>|def</p>");
}

#[test]
fn enter_with_selection_of_an_empty_list_item() {
    let mut model = cm("<ul><li>a{</li><li>}|</li><li>b</li></ul>");
    model.enter();
    assert_eq!(tx(&model), "<ul><li>a</li><li>|</li><li>b</li></ul>");
}

#[test]
fn enter_with_selection_within_a_paragraph() {
    let mut model = cm("<p>a{b}|c</p>");
    model.enter();
    assert_eq!(tx(&model), "<p>a</p><p>|c</p>");
}

// endregion

// region: replace_text

#[test]
fn replacing_selection_across_paragraphs() {
    let mut model = cm("<p>a{bc</p><p>de}|f</p>");
    model.replace_text(Utf16String::from("x"));
    assert_eq!(tx(&model), "<p>ax|f</p>");
}

#[test]
fn replacing_selection_across_list_items() {
    let mut model = cm("<ul><li>{one</li><li>}|two</li></ul><p>after</p>");
    model.replace_text(Utf16String::from("x"));
    assert_eq!(tx(&model), "<ul><li>x|two</li></ul><p>after</p>");
}

#[test]
fn replacing_selection_across_list_items_in_the_middle() {
    let mut model = cm("<ul><li>on{e</li><li>t}|wo</li></ul>");
    model.replace_text(Utf16String::from("x"));
    assert_eq!(tx(&model), "<ul><li>onx|wo</li></ul>");
}

#[test]
fn replacing_selection_from_a_list_into_a_paragraph() {
    // This used to panic
    let mut model = cm("<ul><li>{one</li><li>two</li></ul><p>a}|fter</p>");
    model.replace_text(Utf16String::from("x"));
    assert_eq!(tx(&model), "<p>x|fter</p>");
}

#[test]
fn replacing_selection_from_a_list_item_into_a_paragraph() {
    let mut model = cm("<ul><li>one</li><li>t{wo</li></ul><p>af}|ter</p>");
    model.replace_text(Utf16String::from("x"));
    assert_eq!(tx(&model), "<ul><li>one</li><li>tx|ter</li></ul>");
}

#[test]
fn replacing_selection_from_a_nested_list() {
    // Same result as deleting the selection with backspace and then typing
    let mut model =
        cm("<ol><li><p>{a</p><ul><li>b}|c</li></ul></li><li>d</li></ol>");
    model.replace_text(Utf16String::from("x"));
    assert_eq!(
        tx(&model),
        "<ol><li><ul><li>x|c</li></ul></li><li>d</li></ol>"
    );
}

#[test]
fn replacing_selection_from_a_quote_into_a_code_block() {
    let mut model =
        cm("<blockquote><p>a{b</p></blockquote><pre><code>c}|d</code></pre>");
    model.replace_text(Utf16String::from("x"));
    assert_eq!(tx(&model), "<blockquote><p>ax|d</p></blockquote>");
}

#[test]
fn replacing_selection_across_code_block_lines() {
    let mut model = cm("<pre><code>a{b\nc}|d</code></pre>");
    model.replace_text(Utf16String::from("x"));
    assert_eq!(tx(&model), "<pre><code>ax|d</code></pre>");
}

#[test]
fn replacing_selection_within_formatting_keeps_it() {
    // Not across blocks, but make sure the behaviour doesn't change
    let mut model = cm("<p>a<strong>{bc}|</strong>d</p><p>e</p>");
    model.replace_text(Utf16String::from("x"));
    assert_eq!(tx(&model), "<p>a<strong>x|</strong>d</p><p>e</p>");
}

#[test]
fn replacing_selection_across_list_items_with_several_lines() {
    let mut model = cm("<ul><li>{one</li><li>}|two</li></ul>");
    model.replace_text(Utf16String::from("p\nq"));
    assert_eq!(tx(&model), "<ul><li>p</li><li>q|two</li></ul>");
}

#[test]
fn replace_text_in_across_list_items() {
    let mut model = cm("<ul><li>|one</li><li>two</li></ul>");
    model.replace_text_in(Utf16String::from("x"), 1, 5);
    assert_eq!(tx(&model), "<ul><li>ox|wo</li></ul>");
}

// endregion
