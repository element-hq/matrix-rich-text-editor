// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::{ComposerModel, Location};
use widestring::Utf16String;

use crate::tests::testutils_composer_model::{cm, restore_whitespace, tx};
use crate::tests::testutils_conversion::utf16;

#[test]
fn typing_a_character_into_an_empty_box_appends_it() {
    let mut model = cm("|");
    replace_text(&mut model, "v");
    assert_eq!(tx(&model), "v|");
}

#[test]
fn typing_a_character_at_the_end_appends_it() {
    let mut model = cm("abc|");
    replace_text(&mut model, "d");
    assert_eq!(tx(&model), "abcd|");
}

#[test]
fn typing_a_character_inside_a_tag_inserts_it() {
    let mut model = cm("AAA<b>BB|B</b>CCC");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "AAA<b>BBZ|B</b>CCC");
}

#[test]
fn typing_a_character_in_the_middle_inserts_it() {
    let mut model = cm("|abc");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "Z|abc");
}

#[test]
fn replacing_a_selection_past_the_end_is_harmless() {
    let mut model = cm("|");
    model.select(Location::from(7), Location::from(7));
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "Z|");
}

#[test]
fn replacing_a_selection_with_a_character() {
    let mut model = cm("abc{def}|ghi");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "abcZ|ghi");
}

#[test]
fn replacing_a_backwards_selection_with_a_character() {
    let mut model = cm("abc|{def}ghi");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "abcZ|ghi");
}

#[test]
fn typing_a_character_after_a_multi_codepoint_character() {
    // Woman Astronaut:
    // Woman+Dark Skin Tone+Zero Width Joiner+Rocket
    let mut model = cm("\u{1F469}\u{1F3FF}\u{200D}\u{1F680}|");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "\u{1F469}\u{1F3FF}\u{200D}\u{1F680}Z|");
}

#[test]
fn replacing_an_explicit_text_range_works() {
    let mut model = cm("0123456789|");
    let new_text = utf16("654");
    model.replace_text_in(new_text, 4, 7);
    assert_eq!(tx(&model), "0123654|789");
}

#[test]
fn can_replace_text_in_an_empty_composer_model() {
    let mut cm = ComposerModel::new();
    cm.replace_text(utf16("foo"));
    assert_eq!(tx(&cm), "foo|");
}

#[test]
fn typing_a_character_when_spanning_two_tags_extends_the_first_tag() {
    let mut model = cm("before<b>bo{ld</b>aft}|er");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "before<b>boZ|</b>er");
}

#[test]
fn replacing_an_explicit_range_when_spanning_two_tags_extends_the_first_tag() {
    let mut model = cm("|before<b>bold</b>after");
    model.replace_text_in(utf16("XYZ"), 8, 13);
    assert_eq!(tx(&model), "before<b>boXYZ|</b>er");
}

#[test]
fn typing_a_character_when_spanning_two_whole_tags_extends_the_first_tag() {
    let mut model = cm("before<b>{bold</b>after}|");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "before<b>Z|</b>");
}

#[test]
fn typing_a_character_when_spanning_entire_tag_keeps_formatting() {
    let mut model = cm("before<b>{bo<i>x</i>ld}|</b>after");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "before<b>Z|</b>after");
}

#[test]
fn typing_a_character_when_spanning_over_newly_opened_tags_deletes_them() {
    let mut model = cm("before<b>bo{ld</b>a<i>f</i>t}|er");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "before<b>boZ|</b>er");
}

#[test]
fn typing_a_character_when_spanning_two_separate_identical_tags_joins_them() {
    let mut model = cm("<b>bo{ld</b> plain <b>BO}|LD</b>");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "<b>boZ|LD</b>");
}

#[test]
fn typing_a_character_can_join_the_parents_and_grandparents() {
    let mut model = cm("<b>BB<i>II{II</i>BB</b> gap <b>CC<i>JJ}|JJ</i>CC</b>");
    replace_text(&mut model, "_");
    assert_eq!(tx(&model), "<b>BB<i>II_|JJ</i>CC</b>");
}

#[test]
fn typing_when_spanning_multiple_close_tags_extends_the_first_tag() {
    let mut model = cm("00<code><i>2<b>33{33</b></i>55</code>6}|6");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "00<code><i>2<b>33Z|</b></i></code>6");
}

#[test]
fn typing_when_spanning_open_tags_moves_their_start_forwards() {
    let mut model = cm("0{0<b>1<i>2}|2</i>3</b>44");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "0Z|<b><i>2</i>3</b>44");
}

#[test]
fn typing_that_empties_an_end_tag_deletes_it() {
    let mut model = cm("00{00<b>1111}|</b>");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "00Z|");
}

#[test]
fn typing_when_spanning_whole_open_tags_moves_their_start_forwards() {
    let mut model = cm("{00<b>1<i>22}|</i>3</b>44");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "Z|<b>3</b>44");
}

#[test]
fn typing_into_a_list_item_adds_characters() {
    let mut model = cm("<ul><li>item|</li></ul>");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "<ul><li>itemZ|</li></ul>");
}

#[test]
fn replacing_within_a_list_replaces_characters() {
    let mut model = cm("<ul><li>i{te}|m</li></ul>");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "<ul><li>iZ|m</li></ul>");
}

#[test]
fn replacing_across_list_items_deletes_intervening_ones() {
    let mut model = cm("<ol>\
            <li>1{1</li>\
            <li>22</li>\
            <li>3}|3</li>\
            <li>44</li>\
        </ol>");
    replace_text(&mut model, "Z");
    assert_eq!(
        restore_whitespace(&tx(&model)),
        "<ol><li>1Z|3</li><li>44</li></ol>"
    );
}

#[test]
fn replacing_across_lists_joins_them() {
    let mut model = cm("<ol>\
            <li>1{1</li>\
            <li>22</li>\
        </ol>\
        <ol>\
            <li>33</li>\
            <li>4}|4</li>\
        </ol>");
    replace_text(&mut model, "Z");
    assert_eq!(restore_whitespace(&tx(&model)), "<ol><li>1Z|4</li></ol>");
}

#[test]
fn replacing_a_selection_containing_br_with_a_character() {
    let mut model = cm("abc{de<br />f}|ghi");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "<p>abcZ|ghi</p>");
}

#[test]
fn replacing_a_selection_starting_br_with_a_character() {
    let mut model = cm("abc{<br />def}|ghi");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "<p>abcZ|ghi</p>");
}

#[test]
fn replacing_a_selection_ending_br_with_a_character() {
    let mut model = cm("abc{def<br />}|ghi");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "<p>abcZ|ghi</p>");
}

#[test]
fn multiple_spaces_translates_into_non_breakable_whitespaces() {
    let mut model = cm("abc|");
    replace_text(&mut model, " ");
    assert_eq!(tx(&model), "abc&nbsp;|");
    replace_text(&mut model, " ");
    assert_eq!(tx(&model), "abc&nbsp;&nbsp;|");
    replace_text(&mut model, " ");
    assert_eq!(tx(&model), "abc&nbsp;&nbsp;&nbsp;|");
}

#[test]
fn multiple_spaces_between_text() {
    let model = cm("abc  def ghi   jkl|");
    assert_eq!(tx(&model), "abc&nbsp;&nbsp;def ghi&nbsp;&nbsp; jkl|");
}

#[test]
fn replacing_text_with_empty_paragraphs_removes_nbsps_from_them() {
    let mut model = cm("|");
    replace_text(&mut model, "1\n\u{A0}\n2");
    assert_eq!(
        model.to_tree().to_string(),
        r#"
├>p
│ └>"1"
├>p
└>p
  └>"2"
"#
    );
}

#[test]
fn typing_html_does_not_break_anything() {
    let mut model = cm("|");
    replace_text(&mut model, "<");
    // TODO: tx should handle &lt; and similar
    assert_eq!(tx(&model), "&|lt;");
}

#[test]
fn newline_characters_insert_br_tags() {
    let mut model = cm("|");
    replace_text(&mut model, "abc\ndef\nghi");
    assert_eq!(tx(&model), "<p>abc</p><p>def</p><p>ghi|</p>");
}

#[test]
fn leading_and_trailing_newline_characters_insert_br_tags() {
    let mut model = cm("|");
    replace_text(&mut model, "\nabc");
    assert_eq!(tx(&model), "<p>&nbsp;</p><p>abc|</p>");

    let mut model = cm("|");
    replace_text(&mut model, "abc\n");
    assert_eq!(tx(&model), "<p>abc</p><p>&nbsp;|</p>");

    let mut model = cm("|");
    replace_text(&mut model, "\nabc\n");
    assert_eq!(tx(&model), "<p>&nbsp;</p><p>abc</p><p>&nbsp;|</p>");
}

#[test]
#[allow(deprecated)]
fn inserting_a_line_break_and_text_before_a_line_break_works() {
    let mut model = cm("|{AAA}");
    model.add_line_break();
    model.select(Location::from(0), Location::from(0));
    // Inserting a line break at index 0 (no text node before it) can cause issues
    model.add_line_break();
    model.select(Location::from(0), Location::from(0));
    // Inserting text before a line break with no text node before it is a special case too
    model.replace_text("Test".into());
    assert_eq!(tx(&model), "Test|<br /><br />");
}

#[test]
fn insert_text_between_line_breaks() {
    let mut model = cm("A<br />|<br />B");
    model.replace_text(utf16("C"));
    assert_eq!(tx(&model), "<p>A</p><p>C|</p><p>B</p>");
}

#[test]
fn insert_text_between_line_breaks_in_format_node() {
    let mut model = cm("A<br /><b>|<br />B</b>");
    model.replace_text(utf16("C"));
    assert_eq!(tx(&model), "<p>A</p><p><b>C|</b></p><p><b>B</b></p>");
}

#[test]
fn leading_whitespace_is_replaced_with_nbsp() {
    let model = cm("<p> text|</p>");
    assert_eq!(tx(&model), "<p>&nbsp;text|</p>")
}

#[test]
fn multiple_leading_whitespaces_are_replaced_with_nbsp() {
    let model = cm("<p>  text|</p>");
    assert_eq!(tx(&model), "<p>&nbsp;&nbsp;text|</p>")
}

#[test]
fn trailing_whitespace_is_replaced_with_nbsp() {
    let model = cm("<p>text |</p>");
    assert_eq!(tx(&model), "<p>text&nbsp;|</p>")
}

#[test]
fn multiple_trailing_whitespaces_are_replaced_with_nbsp() {
    let model = cm("<p>text  |</p>");
    assert_eq!(tx(&model), "<p>text&nbsp;&nbsp;|</p>")
}

#[test]
fn leading_and_trailing_whitespace_are_both_replaced_with_nbsp() {
    let model = cm("<p> text |</p>");
    assert_eq!(tx(&model), "<p>&nbsp;text&nbsp;|</p>");
}

#[test]
fn multiple_leading_and_trailing_whitespace_are_all_replaced_with_nbsp() {
    let model = cm("<p>  text  |</p>");
    assert_eq!(tx(&model), "<p>&nbsp;&nbsp;text&nbsp;&nbsp;|</p>");
}

fn replace_text(model: &mut ComposerModel<Utf16String>, new_text: &str) {
    model.replace_text(utf16(new_text));
}
