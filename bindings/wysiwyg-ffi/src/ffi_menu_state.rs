// Copyright 2025 New Vector Ltd.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use std::collections::HashMap;

use crate::into_ffi::IntoFfi;
use crate::{ActionState, ComposerAction};

#[derive(Debug, PartialEq, Eq, uniffi::Enum)]
pub enum MenuState {
    Keep,
    Update {
        action_states: HashMap<ComposerAction, ActionState>,
    },
}

impl MenuState {
    pub fn from(inner: wysiwyg::MenuState) -> Self {
        match inner {
            wysiwyg::MenuState::Keep => Self::Keep,
            wysiwyg::MenuState::Update(menu_update) => Self::Update {
                action_states: menu_update.action_states.into_ffi(),
            },
        }
    }
}
