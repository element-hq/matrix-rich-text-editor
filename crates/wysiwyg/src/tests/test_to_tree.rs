// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::tests::testutils_composer_model::cm;
use crate::ToTree;

#[test]
fn single_nested_tag_produces_tree() {
    let model = cm("<b>abc<i>def</i></b>|");
    assert_eq!(
        model.state.dom.to_tree(),
        r#"
└>b
  ├>"abc"
  └>i
    └>"def"
"#,
    );
}

#[test]
fn multiple_tags_nested_inside_one_produce_tree() {
    let model =
        cm("<ul><li>ab</li><li><b>cd</b></li><li><i><b>ef|</b></i></li></ul>");
    assert_eq!(
        model.state.dom.to_tree(),
        r#"
└>ul
  ├>li
  │ └>"ab"
  ├>li
  │ └>b
  │   └>"cd"
  └>li
    └>i
      └>b
        └>"ef"
"#,
    );
}

#[test]
fn br_within_text_shows_up_in_tree() {
    let model = cm("a<br />|b");
    assert_eq!(
        model.state.dom.to_tree(),
        r#"
├>p
│ └>"a"
└>p
  └>"b"
"#,
    );
}

#[test]
fn link_href_shows_up_in_tree() {
    let model = cm("Some <a href=\"https://matrix.org\">url|</a>");
    assert_eq!(
        model.state.dom.to_tree(),
        r#"
├>"Some "
└>a "https://matrix.org"
  └>"url"
"#,
    );
}

#[test]
fn mention_shows_up_in_tree() {
    let model =
        cm("Some <a href=\"https://matrix.to/#/@test:example.org\">test</a>|");
    assert_eq!(
        model.state.dom.to_tree(),
        r#"
├>"Some "
└>mention "test", https://matrix.to/#/@test:example.org
"#,
    );
}
