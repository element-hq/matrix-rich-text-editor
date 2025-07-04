// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::tests::testutils_composer_model::{cm, tx};

#[test]
fn outputs_paragraphs_as_line_breaks() {
    let mut model = cm("|");
    model.replace_text("hello".into());
    model.enter();
    model.enter();
    model.enter();
    model.enter();
    model.replace_text("Alice".into());

    assert_eq!(
        tx(&model),
        "<p>hello</p><p>&nbsp;</p><p>&nbsp;</p><p>&nbsp;</p><p>Alice|</p>"
    );
    let message_output = model.get_content_as_message_html();
    assert_eq!(message_output, "hello<br /><br /><br /><br />Alice");
}

#[test]
fn outputs_paragraphs_content_without_linebreak_when_followed_by_block() {
    let model = cm("<p>foo</p><blockquote>bar|</blockquote>");
    assert_eq!(tx(&model), "<p>foo</p><blockquote>bar|</blockquote>");
    let message_output = model.get_content_as_message_html();
    assert_eq!(message_output, "foo<blockquote>bar</blockquote>");
}

#[test]
fn only_outputs_href_attribute_on_user_mention() {
    let mut model = cm("|");
    model.insert_mention(
        "https://matrix.to/#/@alice:matrix.org".into(),
        "inner text".into(),
        vec![("style".into(), "some css".into())],
    );
    assert_eq!(tx(&model), "<a style=\"some css\" data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">inner text</a>&nbsp;|");

    let message_output = model.get_content_as_message_html();
    assert_eq!(
        message_output,
        "<a href=\"https://matrix.to/#/@alice:matrix.org\">inner text</a>\u{a0}"
    );
}

#[test]
fn only_outputs_href_attribute_on_room_mention_and_uses_mx_id() {
    let mut model = cm("|");
    model.insert_mention(
        "https://matrix.to/#/#alice:matrix.org".into(),
        "inner text".into(),
        vec![("style".into(), "some css".into())],
    );
    assert_eq!(tx(&model), "<a style=\"some css\" data-mention-type=\"room\" href=\"https://matrix.to/#/#alice:matrix.org\" contenteditable=\"false\">inner text</a>&nbsp;|");

    let message_output = model.get_content_as_message_html();
    assert_eq!(
        message_output,
        "<a href=\"https://matrix.to/#/#alice:matrix.org\">#alice:matrix.org</a>\u{a0}"
    );
}

#[test]
fn only_outputs_href_inner_text_for_at_room_mention() {
    let mut model = cm("|");
    model.insert_at_room_mention(vec![("style".into(), "some css".into())]);
    assert_eq!(tx(&model), "<a style=\"some css\" data-mention-type=\"at-room\" href=\"#\" contenteditable=\"false\">@room</a>&nbsp;|");

    let message_output = model.get_content_as_message_html();
    assert_eq!(message_output, "@room\u{a0}");
}
