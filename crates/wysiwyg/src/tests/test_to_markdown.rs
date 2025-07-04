// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::{
    dom::parser::markdown::MarkdownHTMLParser,
    tests::testutils_composer_model::tx, ComposerModel, ToMarkdown,
};
use widestring::Utf16String;

use super::testutils_composer_model::cm;

#[test]
fn text() {
    assert_to_message_md("abc", "abc");
    assert_to_message_md("abc def", "abc def");
    // Internal spaces are preserved.
    assert_to_message_md("abc   def", "abc   def");
}

// Markdown output contains unescaped special characters but this is ok.
// Athough the plain text content of a Matrix message _may_ contain markdown,
// the format is not specified. It is more important for the message to be
// human readable than valid or machine readable markdown.
#[test]
fn text_with_ascii_punctuation() {
    assert_to_md_no_roundtrip(r"<em>**b**</em>", r"***b***");
    assert_to_md_no_roundtrip(
        r##"!&quot;#$%&amp;'()*+,-./:;&lt;=&gt;?@[\]^_`{|}~"##,
        r##"!"#$%&'()*+,-./:;<=>?@[\]^_`{|}~"##,
    );
}

#[test]
fn text_with_linebreaks() {
    // One new line.
    assert_to_message_md_no_roundtrip(
        "abc<br />def",
        r#"abc
def"#,
    );

    // Two new lines (isn't transformed into a new block).
    assert_to_message_md_no_roundtrip(
        "abc<br /><br />def",
        r#"abc

def"#,
    );
}

#[test]
fn text_with_italic() {
    assert_to_message_md("<em>abc</em>", "*abc*");
    assert_to_message_md("abc <em>def</em> ghi", "abc *def* ghi");
    assert_to_message_md_no_roundtrip(
        "abc <em>line1<br />line2<br /><br />line3</em> def",
        r#"abc *line1*
*line2*
**
*line3* def"#,
    );

    // Intraword emphasis is restricted to `*` so it works here!
    assert_to_message_md("abc<em>def</em>ghi", "abc*def*ghi");

    // Immediate intra-spaces for a strong emphasis isn't supported.
    assert_to_md_no_roundtrip("abc<em> def </em>ghi", "abc* def *ghi");
}

#[test]
fn text_with_bold() {
    assert_to_message_md("<strong>abc</strong>", "__abc__");
    assert_to_message_md("abc <strong>def</strong> ghi", "abc __def__ ghi");
    assert_to_message_md_no_roundtrip(
        "abc <strong>line1<br />line2<br /><br />line3</strong> def",
        r#"abc __line1__
__line2__
____
__line3__ def"#,
    );

    // Intraword emphasis is restricted to `*` (simple emphasis, i.e. italic),
    // it's not possible with `__` (strong emphasis, i.e. bold).
    assert_to_md_no_roundtrip("abc<strong>def</strong>ghi", "abc__def__ghi");

    // Immediate intra-spaces for a strong emphasis isn't supported.
    assert_to_md_no_roundtrip(
        "abc<strong> def </strong>ghi",
        "abc__ def __ghi",
    );
}

#[test]
fn text_with_italic_and_bold() {
    assert_to_message_md("<em><strong>abc</strong></em>", "*__abc__*");
    assert_to_message_md(
        "<em>abc <strong>def</strong></em> ghi",
        "*abc __def__* ghi",
    );
    assert_to_message_md_no_roundtrip(
        "abc <em><strong>line1<br />line2</strong> def</em>",
        r#"abc *__line1__*
*__line2__ def*"#,
    );
}

#[test]
fn text_with_strikethrough() {
    assert_to_message_md("<del>abc</del>", "~~abc~~");
    assert_to_message_md("abc <del>def</del> ghi", "abc ~~def~~ ghi");
    assert_to_message_md_no_roundtrip(
        "abc <del>line1<br />line2<br /><br />line3</del> def",
        r#"abc ~~line1~~
~~line2~~
~~~~
~~line3~~ def"#,
    );

    // Intraword strikethrough isn't supported in the specification.
    assert_to_md_no_roundtrip("abc<del>def</del>ghi", "abc~~def~~ghi");

    // Immediate intra-spaces for a strikethrough isn't supported.
    assert_to_md_no_roundtrip("abc<del> def </del>ghi", "abc~~ def ~~ghi");
}

#[test]
fn text_with_underline() {
    assert_to_message_md("<u>abc</u>", "<u>abc</u>");
}

#[test]
fn text_with_inline_code() {
    assert_to_message_md("<code>abc</code>", "`` abc ``");
    // Inline code with a backtick inside.
    assert_to_message_md("<code>abc ` def</code>", "`` abc ` def ``");
    // Inline code with a backtick at the start.
    assert_to_message_md("<code>`abc</code>", "`` `abc ``");
    assert_to_message_md("abc <code>def</code> ghi", "abc `` def `` ghi");
    assert_to_message_md("abc<code> def </code>ghi", "abc``  def  ``ghi");

    // It's impossible to get a line break inside an inline code with Markdown.
    assert_to_md_no_roundtrip(
        "abc <code>line1<br />line2<br /><br />line3</code> def",
        "abc `` line1 ``\n`` line2 ``\n``  ``\n`` line3 `` def",
    );
    // Inline formatting inside an inline code is ignored.
    assert_to_md_no_roundtrip(
        "abc <code>def <strong>ghi</strong> jkl</code> mno",
        "abc `` def __ghi__ jkl `` mno",
    );
}

#[test]
fn text_with_code_block() {
    assert_to_message_md(
        "<pre><code>**abc**</code></pre>",
        "```\n**abc**\n```\n",
    );
}

#[test]
fn link() {
    assert_to_message_md(r#"<a href="url">abc</a>"#, "[abc](<url>)");
    // Empty link.
    assert_to_message_md(r#"<a href="">abc</a>"#, r#"[abc](<>)"#);
    // Formatting inside link.
    assert_to_message_md(
        r#"<a href="url">abc <strong>def</strong> ghi</a>"#,
        r#"[abc __def__ ghi](<url>)"#,
    );
    assert_to_message_md(r#"<a href="(url)">abc</a>"#, r#"[abc](<\(url\)>)"#);

    // Escaping cannot be roundtrip'ed.
    assert_to_md_no_roundtrip(r#"<a href="u<rl">abc</a>"#, r#"[abc](<u\<rl>)"#);
}

#[test]
fn list_unordered() {
    assert_to_message_md(
        r#"<ul><li>item1</li><li>item2</li></ul>"#,
        r#"* item1
* item2"#,
    );

    assert_to_md_no_roundtrip(
        r#"<ul><li>item1<ul><li>subitem1</li><li>subitem2</li></ul></li><li>item2</li></ul>"#,
        r#"* item1
  * subitem1
  * subitem2
* item2"#,
    );
}

#[test]
fn list_ordered() {
    assert_to_message_md(
        r#"<ol><li>item1</li><li>item2</li></ol>"#,
        r#"1. item1
2. item2"#,
    );

    assert_to_md_no_roundtrip(
        r#"<ol><li>item1<ol><li>subitem1</li><li>subitem2</li></ol></li><li>item2</li></ol>"#,
        r#"1. item1
  1. subitem1
  2. subitem2
2. item2"#,
    );
}

#[test]
fn list_ordered_and_unordered() {
    assert_to_md_no_roundtrip(
        r#"<ol><li>item1<ul><li>subitem1</li><li>subitem2</li></ul></li><li>item2</li></ol>"#,
        r#"1. item1
  * subitem1
  * subitem2
2. item2"#,
    );
}

#[test]
fn user_mention_for_message() {
    assert_to_md_no_roundtrip(
        r#"<a href="https://matrix.to/#/@alice:matrix.org">test</a>"#,
        r#"test"#,
    );
}

#[test]
fn user_mention_for_composer() {
    assert_to_composer_md(
        "<a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">test</a>",
        "<a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">test</a>",
    );
}

#[test]
fn room_mention_for_message() {
    assert_to_md_no_roundtrip(
        r#"<a href="https://matrix.to/#/#alice:matrix.org">test</a>"#,
        r#"#alice:matrix.org"#,
    );
}

#[test]
fn room_mention_for_composer() {
    assert_to_composer_md(
        "<a data-mention-type=\"room\" href=\"https://matrix.to/#/#alice:matrix.org\" contenteditable=\"false\">test</a>",
        "<a data-mention-type=\"room\" href=\"https://matrix.to/#/#alice:matrix.org\" contenteditable=\"false\">test</a>",
    );
}

#[test]
fn at_room_mention_for_message() {
    assert_to_message_md("@room hello!", "@room hello!");
}

#[test]
fn at_room_mention_for_composer() {
    let model = cm("@room hello!|");

    assert_eq!(tx(&model), "<a data-mention-type=\"at-room\" href=\"#\" contenteditable=\"false\">@room</a> hello!|");

    assert_eq!(model.get_content_as_markdown(), "<a data-mention-type=\"at-room\" href=\"#\" contenteditable=\"false\">@room</a> hello!");
    assert_eq!(model.get_content_as_message_markdown(), "@room hello!");
}

fn assert_to_md_no_roundtrip(html: &str, expected_markdown: &str) {
    let markdown = to_message_markdown(html);
    assert_eq!(markdown, expected_markdown);
}

fn assert_to_message_md(html: &str, expected_markdown: &str) {
    let markdown = to_message_markdown(html);
    assert_eq!(markdown, expected_markdown);

    let expected_html = html;
    let html = MarkdownHTMLParser::to_html(&markdown).unwrap();

    assert_eq!(html, expected_html);
}

fn assert_to_message_md_no_roundtrip(html: &str, expected_markdown: &str) {
    let markdown = to_message_markdown(html);
    assert_eq!(markdown, expected_markdown);
}

fn assert_to_composer_md(html: &str, expected_markdown: &str) {
    let markdown = to_composer_markdown(html);
    assert_eq!(markdown, expected_markdown);

    let expected_html = html;
    let html = MarkdownHTMLParser::to_html(&markdown).unwrap();

    assert_eq!(html, expected_html);
}

fn to_message_markdown(html: &str) -> Utf16String {
    let markdown = ComposerModel::from_html(html, 0, 0)
        .state
        .dom
        .to_message_markdown();
    assert!(markdown.is_ok());

    markdown.unwrap()
}

fn to_composer_markdown(html: &str) -> Utf16String {
    let markdown = ComposerModel::from_html(html, 0, 0).state.dom.to_markdown();
    assert!(markdown.is_ok());

    markdown.unwrap()
}
