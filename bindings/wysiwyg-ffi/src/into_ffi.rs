// Copyright 2025 New Vector Ltd.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use std::collections::HashMap;

use crate::{ActionState, ComposerAction};

pub trait IntoFfi {
    fn into_ffi(self) -> HashMap<ComposerAction, ActionState>;
}

impl IntoFfi for &HashMap<wysiwyg::ComposerAction, wysiwyg::ActionState> {
    fn into_ffi(self) -> HashMap<ComposerAction, ActionState> {
        self.iter().map(|(a, s)| (a.into(), s.into())).collect()
    }
}
