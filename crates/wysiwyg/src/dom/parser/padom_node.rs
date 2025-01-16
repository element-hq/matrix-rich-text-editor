// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use html5ever::QualName;
use once_cell::sync::OnceCell;

use super::{paqual_name, PaNodeContainer, PaNodeText};

static TEXT: OnceCell<QualName> = OnceCell::new();

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PaDomNode {
    Container(PaNodeContainer),
    Document(PaNodeContainer),
    Text(PaNodeText),
}

impl PaDomNode {
    pub(crate) fn name(&self) -> &QualName {
        match self {
            PaDomNode::Container(n) => &n.name,
            PaDomNode::Document(n) => &n.name,
            PaDomNode::Text(_) => q(&TEXT, ""),
        }
    }
}

fn q<'a>(once_cell: &'a OnceCell<QualName>, local: &str) -> &'a QualName {
    once_cell.get_or_init(|| paqual_name(local))
}
