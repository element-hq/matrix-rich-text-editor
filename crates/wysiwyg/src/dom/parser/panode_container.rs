// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
        return self
            .get_attr("style")
            .map(|v| {
                return Regex::new(&format!(
                    r"(?i){}:\s*{};",
                    regex::escape(name),
                    regex::escape(value)
                ))
                .map(|re| re.is_match(v))
                .unwrap_or(false);
            })
            .unwrap_or(false);
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