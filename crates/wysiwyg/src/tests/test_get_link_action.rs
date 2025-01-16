// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use indoc::indoc;

use crate::tests::testutils_composer_model::cm;
use crate::tests::testutils_conversion::utf16;

use crate::{LinkAction, Location};

#[test]
fn get_link_action_from_cursor_at_end_of_normal_text() {
    let model = cm("test|");
    assert_eq!(model.get_link_action(), LinkAction::CreateWithText)
}

#[test]
fn get_link_action_from_highlighted_normal_text() {
    let model = cm("{test}|");
    assert_eq!(model.get_link_action(), LinkAction::Create)
}

#[test]
fn get_link_action_from_cursor_inside_a_container() {
    let model = cm("<b><i> test_bold_italic |</i> test_bold </b>");
    assert_eq!(model.get_link_action(), LinkAction::CreateWithText)
}

#[test]
fn get_link_action_from_cursor_inside_text() {
    let model = cm("<b><i> test_bold|_italic </i> test_bold </b>");
    assert_eq!(model.get_link_action(), LinkAction::CreateWithText)
}

#[test]
fn get_link_action_from_selection_inside_a_container() {
    let model = cm("<b><i> {test_bold_italic </i> test}|_bold </b>");
    assert_eq!(model.get_link_action(), LinkAction::Create)
}

#[test]
fn get_link_action_from_highlighted_link() {
    let model = cm("{<a href=\"https://element.io\">test</a>}|");
    assert_eq!(
        model.get_link_action(),
        LinkAction::Edit(utf16("https://element.io"))
    )
}

#[test]
fn get_link_action_from_cursor_at_the_end_of_a_link() {
    let model = cm("<a href=\"https://element.io\">test</a>|");
    assert_eq!(
        model.get_link_action(),
        LinkAction::Edit(utf16("https://element.io"))
    )
}

#[test]
fn get_link_action_from_cursor_inside_a_link() {
    let model = cm("<a href=\"https://element.io\">te|st</a>");
    assert_eq!(
        model.get_link_action(),
        LinkAction::Edit(utf16("https://element.io"))
    )
}

#[test]
fn get_link_action_from_cursor_at_the_start_of_a_link() {
    let model = cm("|<a href=\"https://element.io\">test</a>");
    assert_eq!(
        model.get_link_action(),
        LinkAction::Edit(utf16("https://element.io"))
    )
}

#[test]
fn get_link_action_from_selection_that_contains_a_link_and_non_links() {
    let model = cm("<b>{test_bold <a href=\"https://element.io\">test}|_link</a> test_bold</b>");
    assert_eq!(
        model.get_link_action(),
        LinkAction::Edit(utf16("https://element.io"))
    )
}

#[test]
fn get_link_action_from_selection_that_contains_multiple_links() {
    let model = cm("{<a href=\"https://element.io\">test_element</a> <a href=\"https://matrix.org\">test_matrix</a>}|");
    assert_eq!(
        model.get_link_action(),
        LinkAction::Edit(utf16("https://element.io"))
    )
}

#[test]
fn get_link_action_from_selection_that_contains_multiple_links_partially() {
    let model = cm("<a href=\"https://element.io\">test_{element</a> <a href=\"https://matrix.org\">test}|_matrix</a>");
    assert_eq!(
        model.get_link_action(),
        LinkAction::Edit(utf16("https://element.io"))
    )
}

#[test]
fn get_link_action_from_selection_that_contains_multiple_links_partially_in_different_containers(
) {
    let model = cm("<a href=\"https://element.io\"> <b>test_{element</b></a> <i><a href=\"https://matrix.org\">test}|_matrix</a></i>");
    assert_eq!(
        model.get_link_action(),
        LinkAction::Edit(utf16("https://element.io"))
    )
}

#[test]
fn get_link_action_on_blank_selection() {
    let model = cm("{   }|");
    assert_eq!(model.get_link_action(), LinkAction::CreateWithText)
}

#[test]
fn set_link_with_text_on_blank_selection_after_text() {
    let model = cm("test{   }|");
    assert_eq!(model.get_link_action(), LinkAction::CreateWithText)
}

#[test]
fn set_link_with_text_on_blank_selection_before_text() {
    let model = cm("{   }|test");
    assert_eq!(model.get_link_action(), LinkAction::CreateWithText)
}

#[test]
fn get_link_action_on_blank_selection_between_texts() {
    let model = cm("test{   }|test");
    assert_eq!(model.get_link_action(), LinkAction::CreateWithText)
}

#[test]
fn get_link_action_on_blank_selection_in_container() {
    let model = cm("<b>test{   }| test</b>");
    assert_eq!(model.get_link_action(), LinkAction::CreateWithText)
}

#[test]
fn get_link_action_on_blank_selection_with_line_break() {
    let model = cm("test{  <br> }|test");
    assert_eq!(model.get_link_action(), LinkAction::CreateWithText)
}

#[test]
fn get_link_action_on_blank_selection_with_different_containers() {
    let model = cm("<b>test_bold{ </b>    <i> }|test_italic</i>");
    assert_eq!(model.get_link_action(), LinkAction::CreateWithText)
}

#[test]
fn get_link_action_on_blank_selection_with_different_types_of_whitespaces() {
    let model = cm("test { \t \n \r }| test");
    assert_eq!(model.get_link_action(), LinkAction::CreateWithText)
}

#[test]
fn get_link_action_on_blank_selection_after_a_link() {
    let model = cm("<a href=\"https://element.io\">test</a>{  }|");
    // This is the correct behaviour because the end of a link should be considered part of the link itself
    assert_eq!(
        model.get_link_action(),
        LinkAction::Edit(utf16("https://element.io"))
    )
}

#[test]
fn get_link_action_on_selected_immutable_link() {
    let model = cm(
        "<a contenteditable=\"false\" href=\"https://matrix.org\">{test}|</a>",
    );
    assert_eq!(model.get_link_action(), LinkAction::Disabled);
}

#[test]
fn get_link_action_on_immutable_link_leading() {
    let model = cm(
        "<a contenteditable=\"false\" href=\"https://matrix.org\">|test</a>",
    );
    assert_eq!(model.get_link_action(), LinkAction::Disabled);
}

#[test]
fn get_link_action_on_immutable_link_trailing() {
    let model = cm(
        "<a contenteditable=\"false\" href=\"https://matrix.org\">test|</a>",
    );
    assert_eq!(model.get_link_action(), LinkAction::Disabled);
}

#[test]
fn get_link_action_on_cross_selected_immutable_link() {
    let model = cm(
        "<a contenteditable=\"false\" href=\"https://matrix.org\">te{st</a>text}|",
    );
    assert_eq!(model.get_link_action(), LinkAction::Disabled);
}

#[test]
fn get_link_action_on_multiple_link_with_first_immutable() {
    let mut model = cm(indoc! {r#"
        <a contenteditable="false" href="https://matrix.org">{Matrix_immut</a>
        text
        <a href="https://rust-lang.org">Rust_mut}|</a>
    "#});
    assert_eq!(model.get_link_action(), LinkAction::Disabled);
    // Selecting the mutable link afterwards works
    model.select(Location::from(20), Location::from(20));
    assert_eq!(
        model.get_link_action(),
        LinkAction::Edit("https://rust-lang.org".into()),
    );
}

#[test]
fn get_link_action_on_multiple_link_with_last_immutable() {
    let mut model = cm(indoc! {r#"
        <a href="https://rust-lang.org">{Rust_mut</a>
        text
        <a contenteditable="false" href="https://matrix.org">Matrix_immut}|</a>
    "#});
    assert_eq!(model.get_link_action(), LinkAction::Disabled);
    // Selecting the mutable link afterwards works
    model.select(Location::from(0), Location::from(0));
    assert_eq!(
        model.get_link_action(),
        LinkAction::Edit("https://rust-lang.org".into()),
    );
}

#[test]
fn get_link_action_on_selected_mention() {
    let model =
        cm("{<a href=\"https://matrix.to/#/@test:example.org\">test</a>}|");
    assert_eq!(model.get_link_action(), LinkAction::Create);
}

#[test]
fn get_link_action_on_mention_leading() {
    let model =
        cm("|<a href=\"https://matrix.to/#/@test:example.org\">test</a>");
    assert_eq!(model.get_link_action(), LinkAction::CreateWithText);
}

#[test]
fn get_link_action_on_mention_trailing() {
    let model =
        cm("<a href=\"https://matrix.to/#/@test:example.org\">test</a>|");
    assert_eq!(model.get_link_action(), LinkAction::CreateWithText);
}

#[test]
fn get_link_action_on_cross_selected_mention() {
    let model =
        cm("{<a href=\"https://matrix.to/#/@test:example.org\">test</a>text}|");
    assert_eq!(model.get_link_action(), LinkAction::Create);
}

#[test]
fn get_link_action_on_multiple_link_with_first_is_mention() {
    let mut model = cm(indoc! {r#"
        {<a href="https://matrix.to/#/@test:example.org">test</a>
        text
        <a href="https://rust-lang.org">Rust_mut}|</a>
    "#});
    assert_eq!(
        model.get_link_action(),
        LinkAction::Edit("https://rust-lang.org".into()),
    );
    // Selecting the link afterwards works
    model.select(Location::from(10), Location::from(10));
    assert_eq!(
        model.get_link_action(),
        LinkAction::Edit("https://rust-lang.org".into()),
    );
}

#[test]
fn get_link_action_on_multiple_link_with_last_is_mention() {
    let mut model = cm(indoc! {r#"
        <a href="https://rust-lang.org">{Rust_mut</a>
        text
        <a href="https://matrix.to/#/@test:example.org">test</a>}|
    "#});
    assert_eq!(
        model.get_link_action(),
        LinkAction::Edit("https://rust-lang.org".into()),
    );
    // Selecting the mutable link afterwards works
    model.select(Location::from(0), Location::from(0));
    assert_eq!(
        model.get_link_action(),
        LinkAction::Edit("https://rust-lang.org".into()),
    );
}
