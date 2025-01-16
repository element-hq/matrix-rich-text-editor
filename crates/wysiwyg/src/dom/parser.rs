// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

//! Classes used for parsing HTML into a [super::Dom].
//!
//! We do this by creating a temporary structure held inside a [PaDom]
//! but we throw that away at the end of parsing, and return just a
//! [super::Dom]. All instances of classes within this module are thrown away
//! when parsing finishes.

pub mod markdown;
#[cfg(feature = "sys")]
mod padom;
#[cfg(feature = "sys")]
mod padom_creation_error;
#[cfg(feature = "sys")]
mod padom_creator;
#[cfg(feature = "sys")]
mod padom_handle;
#[cfg(feature = "sys")]
mod padom_node;
#[cfg(feature = "sys")]
mod panode_container;
#[cfg(feature = "sys")]
mod panode_text;
#[cfg(feature = "sys")]
mod paqual_name;
mod parse;

// Group all re-exports for `feature = "sys"`.
#[cfg(feature = "sys")]
mod sys {
    use super::*;

    pub(super) use padom::PaDom;
    pub(super) use padom_creation_error::PaDomCreationError;
    pub(super) use padom_creator::PaDomCreator;
    pub(super) use padom_handle::PaDomHandle;
    pub(super) use padom_node::PaDomNode;
    pub(super) use panode_container::PaNodeContainer;
    pub(super) use panode_text::PaNodeText;
    pub(super) use paqual_name::paqual_name;
}

#[cfg(feature = "sys")]
use sys::*;

pub use parse::parse;
