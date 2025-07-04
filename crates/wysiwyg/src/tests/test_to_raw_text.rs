// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use widestring::Utf16String;

use crate::tests::testutils_composer_model::cm;
use crate::ToRawText;

#[test]
fn empty_text_converts_to_empty_raw_string() {
    assert_eq!(raw("|"), "");
}

#[test]
fn simple_text_converts_directly_to_raw_version() {
    assert_eq!(raw("abcdef|"), "abcdef");
}

#[test]
fn multi_code_unit_characters_convert_to_raw_text_unchanged() {
    assert_eq!(
        raw("\u{1F469}\u{1F3FF}\u{200D}\u{1F680}|"),
        "\u{1F469}\u{1F3FF}\u{200D}\u{1F680}"
    );
}

#[test]
fn tags_are_stripped_from_raw_text() {
    assert_eq!(raw("t<b>a</b>g|"), "tag",);

    assert_eq!(raw("nes<b>ted<i>tag</i></b>s|"), "nestedtags",);

    assert_eq!(
        raw("some <a href=\"https://matrix.org\">link</a>|"),
        "some link",
    );

    // mention
    assert_eq!(
        raw("some <a href=\"https://matrix.to/#/@test:example.org\">test</a>|"),
        "some test",
    );

    assert_eq!(
        raw("list: <ol><li>ab</li><li>cd</li><li><b>e<i>f</i></b></li></ol>|"),
        "list: abcdef",
    );

    assert_eq!(raw("|emptynodes<b><i></i></b>"), "emptynodes");
}

fn raw(s: &str) -> Utf16String {
    cm(s).state.dom.to_raw_text()
}
