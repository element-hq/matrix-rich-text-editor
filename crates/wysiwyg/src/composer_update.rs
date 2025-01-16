// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::dom::UnicodeString;
use crate::link_action::LinkActionUpdate;
use crate::{
    Location, MenuAction, MenuState, ReplaceAll, Selection, TextUpdate,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposerUpdate<S>
where
    S: UnicodeString,
{
    pub text_update: TextUpdate<S>,
    pub menu_state: MenuState,
    pub menu_action: MenuAction,
    pub link_action: LinkActionUpdate<S>,
}

impl<S> ComposerUpdate<S>
where
    S: UnicodeString,
{
    pub fn keep() -> Self {
        Self {
            text_update: TextUpdate::<S>::Keep,
            menu_state: MenuState::Keep,
            menu_action: MenuAction::Keep,
            link_action: LinkActionUpdate::Keep,
        }
    }

    pub fn update_menu_state(
        menu_state: MenuState,
        menu_action: MenuAction,
    ) -> Self {
        Self {
            text_update: TextUpdate::<S>::Keep,
            menu_state,
            menu_action,
            link_action: LinkActionUpdate::Keep,
        }
    }

    pub fn update_selection(
        start: Location,
        end: Location,
        menu_state: MenuState,
        menu_action: MenuAction,
        link_action: LinkActionUpdate<S>,
    ) -> Self {
        Self {
            text_update: TextUpdate::<S>::Select(Selection { start, end }),
            menu_state,
            menu_action,
            link_action,
        }
    }

    pub fn replace_all(
        replacement_html: S,
        start: Location,
        end: Location,
        menu_state: MenuState,
        menu_action: MenuAction,
        link_action: LinkActionUpdate<S>,
    ) -> Self {
        Self {
            text_update: TextUpdate::ReplaceAll(ReplaceAll {
                replacement_html,
                start,
                end,
            }),
            menu_state,
            menu_action,
            link_action,
        }
    }
}
