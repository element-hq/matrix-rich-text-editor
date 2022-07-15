// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{composer_action::ActionResponse, ComposerUpdate};

pub struct ComposerModel {
    html: String, // TODO: not an AST yet!
    selection_start_codepoint: usize,
    selection_end_codepoint: usize,
}

impl ComposerModel {
    pub fn new() -> Self {
        Self {
            html: String::from(""),
            selection_start_codepoint: 0,
            selection_end_codepoint: 0,
        }
    }

    pub fn create_update_replace_all(&self) -> ComposerUpdate {
        ComposerUpdate::replace_all(
            self.html.clone(),
            self.selection_start_codepoint,
            self.selection_end_codepoint,
        )
    }

    /**
     * TODO: just a hack
     */
    fn do_bold(&mut self) {
        let mut range =
            [self.selection_start_codepoint, self.selection_end_codepoint];
        range.sort();

        self.html = format!(
            "{}<strong>{}</strong>{}",
            &self.html[..range[0]],
            &self.html[range[0]..range[1]],
            &self.html[range[1]..]
        );
    }

    /**
     * Cursor is at end_codepoint.
     */
    pub fn select(&mut self, start_codepoint: usize, end_codepoint: usize) {
        self.selection_start_codepoint = start_codepoint;
        self.selection_end_codepoint = end_codepoint;
    }

    pub fn replace_text(&mut self, new_text: &str) -> ComposerUpdate {
        self.html += new_text; // TODO: just a hack
        self.selection_start_codepoint += 1;
        self.selection_end_codepoint += 1;

        // TODO: for now, we replace every time, to check ourselves, but
        // at least some of the time we should not
        self.create_update_replace_all()
        //ComposerUpdate::keep()
    }

    pub fn enter(&mut self) -> ComposerUpdate {
        ComposerUpdate::keep()
    }

    pub fn backspace(&mut self) -> ComposerUpdate {
        ComposerUpdate::keep()
    }

    pub fn delete(&mut self) -> ComposerUpdate {
        ComposerUpdate::keep()
    }

    pub fn bold(&mut self) -> ComposerUpdate {
        self.do_bold();
        self.create_update_replace_all()
    }

    pub fn action_response(
        &mut self,
        action_id: String,
        response: ActionResponse,
    ) -> ComposerUpdate {
        drop(action_id);
        drop(response);
        ComposerUpdate::keep()
    }
}

#[cfg(test)]
mod test {
    use speculoos::{prelude::*, AssertionFailure, DescriptiveSpec, Spec};

    use super::ComposerModel;
    #[test]
    fn typing_a_character_into_an_empty_box_appends_it() {
        let mut model = cm("|");

        model.replace_text("v");

        assert_eq!(tx(model), "v|");
    }

    // Test utils

    trait Roundtrips<T> {
        fn roundtrips(&self);
    }

    impl<'s, T> Roundtrips<T> for Spec<'s, T>
    where
        T: AsRef<str>,
    {
        fn roundtrips(&self) {
            let subject = self.subject.as_ref();
            let output = tx(cm(subject));
            if tx(cm(subject)) != subject {
                AssertionFailure::from_spec(self)
                    .with_expected(String::from(subject))
                    .with_actual(output)
                    .fail();
            }
        }
    }

    fn codepoint_of_byte(s: &str, byte: usize) -> usize {
        let mut i = 0;
        let mut cp = 0;
        while i < byte {
            cp += 1;
            i += 1;
            while !s.is_char_boundary(i) {
                i += 1;
            }
        }
        cp
    }

    fn byte_of_codepoint(s: &str, codepoint: usize) -> usize {
        let mut i = 0;
        let mut cp = 0;
        while i < s.len() {
            if cp == codepoint {
                return i;
            }
            cp += 1;
            i += 1;
            while !s.is_char_boundary(i) {
                i += 1;
            }
        }
        s.len()
    }

    /**
     * Create a ComposerModel from a text representation.
     */
    fn cm(text: &str) -> ComposerModel {
        let i = text.find('|').expect(&format!(
            "ComposerModel text did not contain a '|' symbol: '{}'",
            text,
        ));

        // TODO: range selections

        let cp = codepoint_of_byte(text, i);

        let mut ret = ComposerModel::new();
        ret.selection_start_codepoint = cp;
        ret.selection_end_codepoint = cp;
        ret.html = String::from(&text[..i]) + &text[i + 1..];

        ret
    }

    /**
     * Convert a ComposerModel to a text representation.
     */
    fn tx(model: ComposerModel) -> String {
        if model.selection_start_codepoint == model.selection_end_codepoint {
            let b =
                byte_of_codepoint(&model.html, model.selection_start_codepoint);
            let mut ret = model.html.clone();
            ret.insert(b, '|');
            ret
        } else {
            todo!();
        }
    }

    #[test]
    fn cm_and_tx_roundtrip() {
        assert_that!("|").roundtrips();
        assert_that!("a|").roundtrips();
        assert_that!("a|b").roundtrips();
        assert_that!("|ab").roundtrips();
        assert_that!("foo|\u{1F4A9}bar").roundtrips();
        assert_that!("foo\u{1F4A9}|bar").roundtrips();
        assert_that!("foo|\u{1F4A9}").roundtrips();
        assert_that!("foo\u{1F4A9}|").roundtrips();
        assert_that!("|\u{1F4A9}bar").roundtrips();
        assert_that!("\u{1F4A9}|bar").roundtrips();
    }
}