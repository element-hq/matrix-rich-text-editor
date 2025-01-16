// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use indoc::indoc;
use widestring::Utf16String;

use crate::{
    dom::DomCreationError,
    tests::{testutils_composer_model::tx, testutils_conversion::utf16},
    HtmlParseError,
};

use super::testutils_composer_model::cm;

#[test]
fn set_content_from_html() {
    let mut model = cm("|");
    model.set_content_from_html(&utf16("content")).unwrap();
    assert_eq!(tx(&model), "content|");
}

#[test]
fn set_content_from_html_invalid() {
    let mut model = cm("|");
    let error = model
        .set_content_from_html(&utf16("<strong>hello<strong>"))
        .unwrap_err();
    assert_eq!(
        error,
        DomCreationError::HtmlParseError(HtmlParseError::new(vec![
            "Unexpected open tag at end of body".into()
        ]))
    );
}

#[test]
fn set_content_from_html_containing_newlines() {
    let mut model = cm("|");
    model
        .set_content_from_html(&utf16(
            "<p> \n <strong> \n \n Hello world! \n \n </strong> \n \n </p> \n\n\n",
        ))
        .unwrap();
    assert_eq!(
        &model.to_tree(),
        indoc! {
        r#"

        └>p
          └>strong
            └>"Hello world!"
        "#}
    );
    assert_eq!(tx(&model), "<p><strong>Hello world!|</strong></p>");
}

#[test]
fn set_content_from_html_paragraphs() {
    let mut model = cm("|");
    model
        .set_content_from_html(&utf16(
            "<p>\n  paragraph 1\n</p>\n<p> \n  paragraph 2\n</p>",
        ))
        .unwrap();
    assert_eq!(
        &model.to_tree(),
        indoc! {
        r#"

        ├>p
        │ └>"paragraph 1"
        └>p
          └>"paragraph 2"
        "#}
    );
    assert_eq!(tx(&model), "<p>paragraph 1</p><p>paragraph 2|</p>");
}

#[test]
fn set_content_from_html_paragraphs_containing_newline() {
    let mut model = cm("|");
    model
        .set_content_from_html(&utf16(
            "<p>\n  paragraph\n  across two lines\n</p>\n",
        ))
        .unwrap();
    assert_eq!(
        &model.to_tree(),
        indoc! {
        r#"

        └>p
          └>"paragraph across two lines"
        "#}
    );
    assert_eq!(tx(&model), "<p>paragraph across two lines|</p>");
}

#[test]
fn set_content_from_html_paragraphs_and_inline() {
    let mut model = cm("|");
    model
        .set_content_from_html(&utf16(
            "<p>\n  paragraph 1\n</p>\n<b>\n  inline\n</b>\n<p>\n  paragraph 2\n</p>",
        ))
        .unwrap();
    assert_eq!(
        &model.to_tree(),
        indoc! {
        r#"

        ├>p
        │ └>"paragraph 1"
        ├>p
        │ └>b
        │   └>"inline"
        └>p
          └>"paragraph 2"
        "#}
    );
    assert_eq!(
        tx(&model),
        "<p>paragraph 1</p><p><b>inline</b></p><p>paragraph 2|</p>"
    );
}

#[test]
fn set_content_from_markdown() {
    let mut model = cm("|");
    model.set_content_from_markdown(&utf16("**abc**")).unwrap();
    assert_eq!(tx(&model), "<strong>abc|</strong>");
}

#[test]
fn set_content_from_html_moves_cursor_to_the_end() {
    let mut model = cm("abc|");
    model.set_content_from_html(&"content".into()).unwrap();
    assert_eq!(tx(&model), "content|");
}

#[test]
fn set_content_from_html_single_br() {
    let mut model = cm("|");
    model.set_content_from_html(&utf16("test<br>test")).unwrap();
    assert_eq!(tx(&model), "<p>test</p><p>test|</p>");
}

#[test]
fn set_content_from_html_multiple_br() {
    let mut model = cm("|");
    model
        .set_content_from_html(&utf16("test<br><br>test"))
        .unwrap();
    assert_eq!(tx(&model), "<p>test</p><p>&nbsp;</p><p>test|</p>");
}

#[test]
fn clear() {
    let mut model = cm("|");
    model
        .set_content_from_html(&Utf16String::from("content"))
        .unwrap();
    model.clear();
    assert_eq!(tx(&model), "|");
}

#[test]
fn set_contents_with_line_break_in_code_block() {
    // The first line break inside a block node will be removed as it can be used to just give
    // structure to the node
    let model = cm("<pre>\n<code>|Test</code></pre>");
    assert_eq!(tx(&model), "<pre><code>|Test</code></pre>");
}

#[test]
fn set_content_from_markdown_blockquote() {
    let mut model = cm("|");
    model.set_content_from_markdown(&utf16("> quote")).unwrap();
    assert_eq!(tx(&model), "<blockquote><p>quote|</p></blockquote>");
}

#[test]
fn set_content_from_markdown_blockquote_multiline() {
    let mut model = cm("|");
    model
        .set_content_from_markdown(&utf16("> quote\n\nfollowing text"))
        .unwrap();
    assert_eq!(
        tx(&model),
        "<blockquote><p>quote</p></blockquote><p>following text|</p>"
    );
}

#[test]
fn set_content_from_markdown_codeblock_with_newlines() {
    let mut model = cm("|");
    model
        .set_content_from_markdown(&utf16("```\nI am a code block\n```"))
        .unwrap();
    assert_eq!(tx(&model), "<pre><code>I am a code block|</code></pre>");
}

#[test]
fn set_content_from_markdown_codeblock_with_newlines_in_the_middle() {
    let mut model = cm("|");
    model
        .set_content_from_markdown(&utf16("```\nI am\na code block\n```"))
        .unwrap();
    assert_eq!(tx(&model), "<pre><code>I am\na code block|</code></pre>");
}

#[test]
fn set_content_from_markdown_multiple_new_lines() {
    let mut model = cm("|");
    model
        .set_content_from_markdown(&utf16("test\n\n\ntest"))
        .unwrap();
    assert_eq!(tx(&model), "<p>test</p><p>&nbsp;</p><p>test|</p>");
}

#[test]
fn set_content_from_markdown_one_new_line() {
    let mut model = cm("|");
    model
        .set_content_from_markdown(&utf16("test\ntest"))
        .unwrap();
    assert_eq!(tx(&model), "<p>test</p><p>test|</p>");
}
