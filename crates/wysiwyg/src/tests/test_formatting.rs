// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::tests::testutils_composer_model::{cm, tx};
use crate::tests::testutils_conversion::utf16;
use widestring::Utf16String;

use crate::InlineFormatType::Bold;
use crate::Location;
use crate::{ComposerModel, InlineFormatType};

#[test]
fn selecting_and_bolding_multiple_times() {
    let mut model = cm("aabbcc|");
    model.select(Location::from(0), Location::from(2));
    model.bold();
    model.select(Location::from(4), Location::from(6));
    model.bold();
    assert_eq!(
        &model.state.dom.to_string(),
        "<strong>aa</strong>bb<strong>cc</strong>"
    );
}

#[test]
fn bolding_ascii_adds_strong_tags() {
    let mut model = cm("aa{bb}|cc");
    model.bold();
    assert_eq!(tx(&model), "aa<strong>{bb}|</strong>cc");

    let mut model = cm("aa|{bb}cc");
    model.bold();
    assert_eq!(tx(&model), "aa<strong>|{bb}</strong>cc");
}

#[test]
fn format_several_nodes_with_empty_text_nodes() {
    let mut model = cm("{some}| different nodes");
    model.bold();
    model.select(Location::from(5), Location::from(14));
    model.italic();
    model.select(Location::from(2), Location::from(17));
    model.strike_through();
    assert_eq!(tx(&model), "<strong>so<del>{me</del></strong><del>&nbsp;</del><em><del>different</del></em><del>&nbsp;no}|</del>des")
}

#[test]
fn selecting_and_unbolding_multiple_times() {
    let mut model = cm("<strong>aabbcc|</strong>");
    model.select(Location::from(0), Location::from(2));
    model.bold();
    model.select(Location::from(4), Location::from(6));
    model.bold();
    assert_eq!(tx(&model), "aa<strong>bb</strong>{cc}|");
}

#[test]
fn unformat_nested_node() {
    let mut model = cm("aa<em>b<strong>{bc}|</strong></em>c");
    model.bold();
    assert_eq!(tx(&model), "aa<em>b{bc}|</em>c");
}

#[test]
fn partial_unformat_nested_node() {
    let mut model = cm("aa<em>b<strong>b{c}|</strong></em>c");
    model.bold();
    assert_eq!(tx(&model), "aa<em>b<strong>b</strong>{c}|</em>c");
}

#[test]
fn unformat_toplevel_node_moves_nested_nodes() {
    let mut model = cm("aa<em>{b<strong>bc}|</strong></em>c");
    model.italic();
    assert_eq!(tx(&model), "aa{b<strong>bc}|</strong>c");
}

#[test]
fn partial_unformat_toplevel_node_reconstructs_expected_model() {
    let mut model = cm("aa<em>b<strong>b{c}|</strong></em>c");
    model.italic();
    assert_eq!(tx(&model), "aa<em>b</em><strong><em>b</em>{c}|</strong>c");
}

#[test]
fn unformat_several_nodes() {
    let mut model = cm("<strong>so<del>me</del></strong><del> </del><em><del>different</del></em><del> no</del>des|");
    model.select(Location::from(2), Location::from(17));
    model.strike_through();
    model.select(Location::from(5), Location::from(14));
    model.italic();
    model.select(Location::from(0), Location::from(4));
    model.bold();
    assert_eq!(tx(&model), "{some}| different nodes");
}

#[test]
fn formatting_twice_adds_no_formatting() {
    let input = "a{aabbbcc}|c";
    let mut model = cm(input);
    for _i in 0..=1 {
        model.bold();
        model.italic();
        model.strike_through();
        model.underline();
    }
    assert_eq!(tx(&model), input);
}

#[test]
fn formatting_nested_format_nodes_and_line_breaks() {
    let mut model = cm("aa<strong>a<br />{bbb<br />}|cc</strong>c");
    model.italic();
    assert_eq!(
        tx(&model),
        "<p>aa<strong>a</strong></p><p><strong><em>{bbb</em></strong></p><p><strong>cc</strong>c</p>"
    );
}

#[test]
fn formatting_deeper_nested_format_nodes_and_nested_line_breaks() {
    let mut model = cm("aa<strong>a<u><br />{b</u>bb<br />}|cc</strong>c");
    model.italic();
    assert_eq!(
        tx(&model),
        "<p>aa<strong>a<u></u></strong></p><p><strong><u><em>{b</em></u><em>bb</em></strong></p><p><strong>cc</strong>c</p>",
    );
}

#[test]
fn formatting_with_zero_length_selection_apply_on_replace_text() {
    let mut model = cm("aaa|bbb");
    model.bold();
    model.italic();
    model.underline();
    assert_eq!(tx(&model), "aaa|bbb");
    assert_eq!(
        model.state.toggled_format_types,
        Vec::from([
            InlineFormatType::Bold,
            InlineFormatType::Italic,
            InlineFormatType::Underline
        ])
    );
    model.replace_text(utf16("ccc"));
    assert_eq!(tx(&model), "aaa<strong><em><u>ccc|</u></em></strong>bbb");
}

#[test]
fn unformatting_with_zero_length_selection_removes_on_replace_text() {
    let mut model = cm("<strong>aaa|bbb</strong>");
    model.bold();
    assert_eq!(
        model.state.toggled_format_types,
        Vec::from([InlineFormatType::Bold]),
    );
    model.replace_text(utf16("ccc"));
    assert_eq!(tx(&model), "<strong>aaa</strong>ccc|<strong>bbb</strong>");
}

#[test]
fn formatting_and_unformatting_with_zero_length_selection() {
    let mut model = cm("<em>aaa|bbb</em>");
    model.bold();
    model.italic();
    model.replace_text(utf16("ccc"));
    assert_eq!(tx(&model), "<em>aaa</em><strong>ccc|</strong><em>bbb</em>");
}

#[test]
fn selecting_removes_toggled_format_types() {
    let mut model = cm("aaa|");
    model.bold();
    assert_eq!(
        model.state.toggled_format_types,
        Vec::from([InlineFormatType::Bold]),
    );
    model.select(Location::from(2), Location::from(2));
    assert_eq!(model.state.toggled_format_types, Vec::new(),);
    model.replace_text(utf16("ccc"));
    assert_eq!(tx(&model), "aaccc|a");
}

#[test]
fn formatting_again_removes_toggled_format_type() {
    let mut model = cm("aaa|");
    model.bold();
    assert_eq!(
        model.state.toggled_format_types,
        Vec::from([InlineFormatType::Bold]),
    );
    model.bold();
    assert_eq!(model.state.toggled_format_types, Vec::new(),);
}

#[test]
fn unformatting_consecutive_same_formatting_nodes() {
    let mut model = cm("{<strong>Test</strong><strong> </strong><strong>test</strong><strong> test</strong>}|");
    model.bold();
    assert_eq!(tx(&model), "{Test test test}|");
}

#[test]
fn unformatting_consecutive_same_formatting_nodes_with_nested_line_break() {
    let mut model = cm("{<strong>Test</strong><strong> </strong><strong>te<br />st</strong><strong> test</strong>}|");
    model.bold();
    assert_eq!(tx(&model), "<p>{Test te</p><p>st test}|</p>");
}

#[test]
fn unformatting_consecutive_same_formatting_nodes_with_nested_node() {
    let mut model = cm("\
        {<strong>Test</strong>\
        <strong> </strong>\
        <strong>t<em>es</em>t</strong>\
        <strong> test</strong>}|\
    ");

    model.bold();
    assert_eq!(tx(&model), "{Test t<em>es</em>t test}|");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn format_empty_model_applies_formatting() {
    let mut model = ComposerModel::<Utf16String>::new();
    model.bold();
    assert!(model.state.toggled_format_types.contains(&Bold));
}

#[test]
fn changing_selection_to_same_doesnt_removes_formatting_state() {
    let mut model = cm("AAA | BBB");
    model.bold();
    model.select(Location::from(4), Location::from(4));
    assert!(model.state.toggled_format_types.contains(&Bold));
}

#[test]
fn formatting_before_typing_anything_applies_formatting() {
    let mut model = cm("|");
    model.bold();
    model.replace_text(utf16("d"));
    assert_eq!(tx(&model), "<strong>d|</strong>");
}

#[test]
fn formatting_in_an_empty_model_applies_formatting() {
    let mut model = ComposerModel::new();
    model.bold();
    model.replace_text(utf16("d"));
    assert_eq!(tx(&model), "<strong>d|</strong>");
}

#[test]
fn formatting_some_char_in_word_with_inline_code() {
    let mut model = cm("w{or}|d");
    model.inline_code();
    assert_eq!(tx(&model), "w<code>{or}|</code>d");
}

#[test]
fn formatting_multiple_lines_with_inline_code() {
    let mut model = cm("fo{o<br />b}|ar");
    model.inline_code();
    assert_eq!(
        tx(&model),
        "<p>fo<code>{o</code></p><p><code>b}|</code>ar</p>"
    );
}

#[test]
fn splitting_a_formatting_tag_across_two_lines() {
    let mut model = cm("|");
    model.strike_through();
    model.replace_text(utf16("foo"));
    assert_eq!(tx(&model), "<del>foo|</del>");
    model.enter();
    assert_eq!(tx(&model), "<p><del>foo</del></p><p><del>|</del></p>");
    model.replace_text(utf16("bar"));
    assert_eq!(tx(&model), "<p><del>foo</del></p><p><del>bar|</del></p>");
}

#[test]
fn splitting_a_formatting_tag_across_multiple_lines() {
    let mut model = cm("|");
    model.strike_through();
    model.replace_text(utf16("foo"));
    assert_eq!(tx(&model), "<del>foo|</del>");
    model.enter();
    assert_eq!(tx(&model), "<p><del>foo</del></p><p><del>|</del></p>");
    model.enter();
    assert_eq!(
        tx(&model),
        "<p><del>foo</del></p><p>&nbsp;</p><p><del>|</del></p>"
    );
    model.replace_text(utf16("bar"));
    assert_eq!(
        tx(&model),
        "<p><del>foo</del></p><p>&nbsp;</p><p><del>bar|</del></p>"
    );
}

#[test]
fn splitting_a_formatting_tag_multiple_times_across_multiple_lines() {
    let mut model = cm("|");
    model.strike_through();
    model.replace_text(utf16("foo"));
    assert_eq!(tx(&model), "<del>foo|</del>");
    model.enter();
    assert_eq!(tx(&model), "<p><del>foo</del></p><p><del>|</del></p>");
    model.enter();
    assert_eq!(
        tx(&model),
        "<p><del>foo</del></p><p>&nbsp;</p><p><del>|</del></p>"
    );
    model.replace_text(utf16("bar"));
    assert_eq!(
        tx(&model),
        "<p><del>foo</del></p><p>&nbsp;</p><p><del>bar|</del></p>"
    );
    model.enter();
    assert_eq!(
        tx(&model),
        "<p><del>foo</del></p><p>&nbsp;</p><p><del>bar</del></p><p><del>|</del></p>"
    );
}

#[test]
fn locations_when_pressing_enter() {
    let mut model = cm("|");
    model.replace_text(utf16("foo"));
    assert_eq!(tx(&model), "foo|");
    model.enter();
    assert_eq!(tx(&model), "<p>foo</p><p>&nbsp;|</p>");
    assert_eq!(model.state.start, Location::from(4));
    model.enter();
    assert_eq!(tx(&model), "<p>foo</p><p>&nbsp;</p><p>&nbsp;|</p>");
    assert_eq!(model.state.start, Location::from(5));
    model.enter();
    assert_eq!(
        tx(&model),
        "<p>foo</p><p>&nbsp;</p><p>&nbsp;</p><p>&nbsp;|</p>"
    );
    assert_eq!(model.state.start, Location::from(6));
    model.enter();
}

#[test]
fn formatting_in_an_empty_paragraph_applies_formatting() {
    let mut model = cm("<p>A</p><p>|</p>");
    model.bold();
    model.replace_text("B".into());
    assert_eq!(tx(&model), "<p>A</p><p><strong>B|</strong></p>");
}
