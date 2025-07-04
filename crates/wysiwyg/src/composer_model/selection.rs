// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::{ComposerModel, ComposerUpdate, Location, UnicodeString};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    /// Select the text at the supplied code unit positions.
    /// The cursor is at end.
    pub fn select(
        &mut self,
        start: Location,
        end: Location,
    ) -> ComposerUpdate<S> {
        if self.state.start == start && self.state.end == end {
            return ComposerUpdate::keep();
        }
        self.state.toggled_format_types.clear();
        self.state.start = start;
        self.state.end = end;

        self.create_update_update_selection()
    }

    /// Return the start and end of the selection, ensuring the first number
    /// returned is <= the second, and they are both between 0 and the number
    /// of code units in the string representation of the Dom.
    pub(crate) fn safe_selection(&self) -> (usize, usize) {
        self.safe_locations_from(self.state.start, self.state.end)
    }

    pub(crate) fn safe_locations_from(
        &self,
        start: Location,
        end: Location,
    ) -> (usize, usize) {
        let len = self.state.dom.text_len();

        let mut s: usize = start.into();
        let mut e: usize = end.into();
        s = s.clamp(0, len);
        e = e.clamp(0, len);
        if s > e {
            (e, s)
        } else {
            (s, e)
        }
    }

    /// Return a boolean to let us know if we have a selection
    pub fn has_selection(&self) -> bool {
        let (s, e) = self.safe_selection();
        s != e
    }

    /// Return a boolean to let us know if we have a cursor, ie a zero length selection
    pub fn has_cursor(&self) -> bool {
        let (s, e) = self.safe_selection();
        s == e
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::tests::testutils_composer_model::cm;

    #[test]
    fn safe_selection_leaves_forward_selection_untouched() {
        let model = cm("out{ <b>bol}|d</b> spot");
        assert_eq!((3, 7), model.safe_selection());
    }

    #[test]
    fn safe_selection_reverses_backward_selection() {
        let model = cm("out|{ <b>bol}d</b> spot");
        assert_eq!((3, 7), model.safe_selection());
    }

    #[test]
    fn safe_selection_fixes_too_wide_selection() {
        let mut model = cm("out <b>bol</b> spot|");
        model.state.start = Location::from(0);
        model.state.end = Location::from(13);
        assert_eq!((0, 12), model.safe_selection());

        let mut model = cm("out <b>bol</b> {spot}|");
        model.state.end = Location::from(33);
        assert_eq!((8, 12), model.safe_selection());
    }
}
