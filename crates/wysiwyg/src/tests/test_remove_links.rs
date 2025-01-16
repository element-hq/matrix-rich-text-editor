// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::tests::testutils_composer_model::{cm, tx};

#[test]
fn remove_link_on_a_non_link_node() {
    let mut model = cm("{test}|");
    model.remove_links();
    assert_eq!(tx(&model), "{test}|");
}

#[test]
fn remove_selected_link() {
    let mut model = cm("<a href=\"https://matrix.org\">{test_link}|</a>");
    model.remove_links();
    assert_eq!(tx(&model), "{test_link}|");
}

#[test]
fn remove_link_with_cursor_at_end() {
    let mut model = cm("<a href=\"https://matrix.org\">test_link|</a>");
    model.remove_links();
    assert_eq!(tx(&model), "test_link|");
}

#[test]
fn remove_link_with_cursor_in_the_middle() {
    let mut model = cm("<a href=\"https://matrix.org\">test|_link</a>");
    model.remove_links();
    assert_eq!(tx(&model), "test|_link");
}

#[test]
fn remove_link_with_cursor_at_the_start() {
    let mut model = cm("<a href=\"https://matrix.org\">|test_link</a>");
    model.remove_links();
    assert_eq!(tx(&model), "|test_link");
}

#[test]
fn remove_selected_link_and_undo() {
    let mut model = cm("<a href=\"https://matrix.org\">{test_link}|</a>");
    model.remove_links();
    assert_eq!(tx(&model), "{test_link}|");
    model.undo();
    assert_eq!(
        tx(&model),
        "<a href=\"https://matrix.org\">{test_link}|</a>"
    );
}

#[test]
fn remove_partially_selected_link() {
    let mut model = cm("<a href=\"https://matrix.org\">{test}|_link</a>");
    model.remove_links();
    assert_eq!(tx(&model), "{test}|_link");
}

#[test]
fn remove_link_in_selected_container() {
    let mut model = cm(
        "<b>{test <a href=\"https://matrix.org\">test_link_bold}|</a></b> test",
    );
    model.remove_links();
    assert_eq!(tx(&model), "<b>{test test_link_bold}|</b> test");
}

#[test]
fn remove_link_that_contains_a_container() {
    let mut model =
        cm("<a href=\"https://matrix.org\"><b>{test_link_bold}|</b></a>");
    model.remove_links();
    assert_eq!(tx(&model), "<b>{test_link_bold}|</b>");
}

#[test]
fn remove_multiple_selected_links() {
    let mut model = cm("<a href=\"https://matrix.org\">{test_link_1</a> <a href=\"https://element.io\">test_link_2}|</a>");
    model.remove_links();
    assert_eq!(tx(&model), "{test_link_1 test_link_2}|");
}

#[test]
fn remove_multiple_partially_selected_links() {
    let mut model = cm("<a href=\"https://matrix.org\">test_{link_1</a> <a href=\"https://element.io\">test}|_link_2</a>");
    model.remove_links();
    assert_eq!(tx(&model), "test_{link_1 test}|_link_2");
}

#[test]
fn remove_multiple_partially_selected_links_in_different_containers() {
    let mut model = cm("<b><a href=\"https://matrix.org\">test_{link_bold</a></b> <a href=\"https://element.io\"><i>test}|_link_italic</i></a>");
    model.remove_links();
    assert_eq!(
        tx(&model),
        "<b>test_{link_bold</b> <i>test}|_link_italic</i>"
    );
}

#[test]
fn remove_link_between_text_nodes_joins() {
    let mut model = cm("abc{<a href=\"https://matrix.org\">def</a>}|ghi");
    model.remove_links();
    assert_eq!(tx(&model), "abc{def}|ghi");
    model.state.dom.explicitly_assert_invariants();
}
