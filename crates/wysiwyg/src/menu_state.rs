// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::action_state::ActionState;
use crate::ComposerAction;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuState {
    Keep,
    Update(MenuStateUpdate),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuStateUpdate {
    pub action_states: HashMap<ComposerAction, ActionState>,
}
