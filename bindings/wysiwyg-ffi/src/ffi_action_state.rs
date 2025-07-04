// Copyright 2025 New Vector Ltd.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

#[derive(Debug, PartialEq, Eq, uniffi::Enum)]
pub enum ActionState {
    Enabled,
    Reversed,
    Disabled,
}

impl From<&wysiwyg::ActionState> for ActionState {
    fn from(inner: &wysiwyg::ActionState) -> Self {
        match inner {
            wysiwyg::ActionState::Enabled => Self::Enabled,
            wysiwyg::ActionState::Reversed => Self::Reversed,
            wysiwyg::ActionState::Disabled => Self::Disabled,
        }
    }
}
