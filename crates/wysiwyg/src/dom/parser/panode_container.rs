// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use html5ever::QualName;
use regex::Regex;

use super::PaDomHandle;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PaNodeContainer {
    pub(crate) name: QualName,
    pub(crate) attrs: Vec<(String, String)>,
    pub(crate) children: Vec<PaDomHandle>,
}
impl PaNodeContainer {
    pub(crate) fn get_attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(n, _v)| n == name)
            .map(|(_n, v)| v.as_str())
    }

    pub(crate) fn contains_style(&self, name: &str, value: &str) -> bool {
        self.get_attr("style")
            .map(|v| {
                Regex::new(&format!(
                    r"(?i){}:\s*{};",
                    regex::escape(name),
                    regex::escape(value)
                ))
                .map(|re| re.is_match(v))
                .unwrap_or(false)
            })
            .unwrap_or(false)
    }
}

#[test]
fn test_contains_style() {
    let node = PaNodeContainer {
        name: QualName::new(None, "div".into(), "div".into()),
        attrs: vec![("style".into(), "font-weight:bold;".into())],
        children: Vec::new(),
    };
    assert!(node.contains_style("font-weight", "bold"));
    assert!(!node.contains_style("font-weight", "normal"));
}
