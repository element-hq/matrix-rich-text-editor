// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use std::ops;

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Default)]
pub struct Location(usize);

impl From<usize> for Location {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<Location> for usize {
    fn from(val: Location) -> Self {
        val.0
    }
}

impl PartialEq<usize> for Location {
    fn eq(&self, other: &usize) -> bool {
        self.0 == *other
    }
}

impl ops::Add for Location {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl ops::AddAssign<isize> for Location {
    fn add_assign(&mut self, rhs: isize) {
        let mut i = isize::try_from(self.0).expect("Location was too large!");
        i += rhs;
        if i < 0 {
            i = 0;
        }
        self.0 = usize::try_from(i).unwrap();
    }
}

impl ops::SubAssign<isize> for Location {
    fn sub_assign(&mut self, rhs: isize) {
        *self += -rhs
    }
}
