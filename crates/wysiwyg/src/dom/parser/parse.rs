// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use regex::Regex;

use crate::dom::dom_creation_error::HtmlParseError;
use crate::dom::html_source::HtmlSource;
use crate::dom::nodes::dom_node::DomNodeKind::{self};
use crate::dom::nodes::{ContainerNode, ContainerNodeKind};
use crate::dom::Dom;
use crate::{DomHandle, DomNode, UnicodeString};

pub fn parse<S>(html: &str) -> Result<Dom<S>, HtmlParseError>
where
    S: UnicodeString,
{
    cfg_if::cfg_if! {
        if #[cfg(feature = "sys")] {
            sys::HtmlParser::default().parse(html)
        } else if #[cfg(all(feature = "js", target_arch = "wasm32"))] {
            js::HtmlParser::default().parse(html)
        } else {
            unreachable!("The `sys` or `js` are mutually exclusive, and one of them must be enabled.")
        }
    }
}

pub fn parse_from_source<S>(
    html: &str,
    source: HtmlSource,
) -> Result<Dom<S>, HtmlParseError>
where
    S: UnicodeString,
{
    cfg_if::cfg_if! {
        if #[cfg(feature = "sys")] {
            sys::HtmlParser::default().parse_from_source(html, source)
        } else if #[cfg(all(feature = "js", target_arch = "wasm32"))] {
            js::HtmlParser::default().parse_from_source(html, source)
        } else {
            unreachable!("The `sys` or `js` are mutually exclusive, and one of them must be enabled.")
        }
    }
}

/* These html fragments were copied directly from google docs/ms docs(minus the cleanup/stripping we do in "replace_html" function) and represents the following content:
└>ol
  ├>li
  │ └>p
  │   └>i
  │     └>"Italic"
  ├>li
  │ └>p
  │   └>b
  │     └>"Bold"
  ├>li
  │ └>p
  │   └>"Unformatted"
  ├>li
  │ └>p
  │   └>del
  │     └>"Strikethrough"
  ├>li
  │ └>p
  │   └>u
  │     └>"Underlined"
  └>li
    ├>p
    │ └>a "http://matrix.org"
    │   └>u
    │     └>"Linked"
    └>ul
      └>li
        └>p
          └>"Nested"
*/
#[cfg(test)]
pub const GOOGLE_DOC_HTML_PASTEBOARD: &str = r#"<ol style="margin-top:0;margin-bottom:0;padding-inline-start:48px;"><li dir="ltr" style="list-style-type:decimal;font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:400;font-style:italic;font-variant:normal;text-decoration:none;vertical-align:baseline;white-space:pre;" aria-level="1"><p dir="ltr" style="line-height:1.38;margin-top:0pt;margin-bottom:0pt;" role="presentation"><span style="font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:400;font-style:italic;font-variant:normal;text-decoration:none;vertical-align:baseline;white-space:pre;white-space:pre-wrap;">Italic</span></p></li><li dir="ltr" style="list-style-type:decimal;font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:700;font-style:normal;font-variant:normal;text-decoration:none;vertical-align:baseline;white-space:pre;" aria-level="1"><p dir="ltr" style="line-height:1.38;margin-top:0pt;margin-bottom:0pt;" role="presentation"><span style="font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:700;font-style:normal;font-variant:normal;text-decoration:none;vertical-align:baseline;white-space:pre;white-space:pre-wrap;">Bold</span></p></li><li dir="ltr" style="list-style-type:decimal;font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:400;font-style:normal;font-variant:normal;text-decoration:none;vertical-align:baseline;white-space:pre;" aria-level="1"><p dir="ltr" style="line-height:1.38;margin-top:0pt;margin-bottom:0pt;" role="presentation"><span style="font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:400;font-style:normal;font-variant:normal;text-decoration:none;vertical-align:baseline;white-space:pre;white-space:pre-wrap;">Unformatted</span></p></li><li dir="ltr" style="list-style-type:decimal;font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:400;font-style:normal;font-variant:normal;text-decoration:line-through;-webkit-text-decoration-skip:none;text-decoration-skip-ink:none;vertical-align:baseline;white-space:pre;" aria-level="1"><p dir="ltr" style="line-height:1.38;margin-top:0pt;margin-bottom:0pt;" role="presentation"><span style="font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:400;font-style:normal;font-variant:normal;text-decoration:line-through;-webkit-text-decoration-skip:none;text-decoration-skip-ink:none;vertical-align:baseline;white-space:pre;white-space:pre-wrap;">Strikethrough</span></p></li><li dir="ltr" style="list-style-type:decimal;font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:400;font-style:normal;font-variant:normal;text-decoration:none;vertical-align:baseline;white-space:pre;" aria-level="1"><p dir="ltr" style="line-height:1.38;margin-top:0pt;margin-bottom:0pt;" role="presentation"><span style="font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:400;font-style:normal;font-variant:normal;text-decoration:underline;-webkit-text-decoration-skip:none;text-decoration-skip-ink:none;vertical-align:baseline;white-space:pre;white-space:pre-wrap;">Underlined</span></p></li><li dir="ltr" style="list-style-type:decimal;font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:400;font-style:normal;font-variant:normal;text-decoration:none;vertical-align:baseline;white-space:pre;" aria-level="1"><p dir="ltr" style="line-height:1.38;margin-top:0pt;margin-bottom:0pt;" role="presentation"><a href="http://matrix.org" style="text-decoration:none;"><span style="font-size:11pt;font-family:Arial,sans-serif;color:#1155cc;background-color:transparent;font-weight:400;font-style:normal;font-variant:normal;text-decoration:underline;-webkit-text-decoration-skip:none;text-decoration-skip-ink:none;vertical-align:baseline;white-space:pre;white-space:pre-wrap;">Linked</span></a></p></li><ul style="margin-top:0;margin-bottom:0;padding-inline-start:48px;"><li dir="ltr" style="list-style-type:circle;font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:400;font-style:normal;font-variant:normal;text-decoration:none;vertical-align:baseline;white-space:pre;" aria-level="2"><p dir="ltr" style="line-height:1.38;margin-top:0pt;margin-bottom:0pt;" role="presentation"><span style="font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:400;font-style:normal;font-variant:normal;text-decoration:none;vertical-align:baseline;white-space:pre;white-space:pre-wrap;">Nested</span></p></li></ul></ol>"#;
#[cfg(test)]
pub const MS_DOC_HTML_PASTEBOARD: &str = r#"<div class="ListContainerWrapper SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; position: relative; color: rgb(0, 0, 0); font-family: Aptos, Aptos_MSFontService, sans-serif; font-size: 16px; font-style: normal; font-variant-ligatures: normal; font-variant-caps: normal; font-weight: 400; letter-spacing: normal; orphans: 2; text-align: left; text-indent: 0px; text-transform: none; widows: 2; word-spacing: 0px; -webkit-text-stroke-width: 0px; white-space: normal; background-color: rgb(255, 255, 255); text-decoration-thickness: initial; text-decoration-style: initial; text-decoration-color: initial;"><ol class="NumberListStyle1 SCXW204127278 BCX0" role="list" start="1" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; cursor: text; list-style-type: decimal; overflow: visible;"><li aria-setsize="-1" data-leveltext="%1." data-font="" data-listid="3" data-list-defn-props="{&quot;335552541&quot;:0,&quot;335559685&quot;:720,&quot;335559991&quot;:360,&quot;469769242&quot;:[65533,0,46],&quot;469777803&quot;:&quot;left&quot;,&quot;469777804&quot;:&quot;%1.&quot;,&quot;469777815&quot;:&quot;hybridMultilevel&quot;}" data-aria-posinset="1" data-aria-level="1" role="listitem" class="OutlineElement Ltr SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px 0px 0px 24px; padding: 0px; user-select: text; clear: both; cursor: text; overflow: visible; position: relative; direction: ltr; display: block; font-size: 12pt; font-family: Aptos, Aptos_MSFontService, sans-serif; vertical-align: baseline;"><p class="Paragraph SCXW204127278 BCX0" paraid="81782558" paraeid="{dc225749-75de-40ef-a9ba-ce0ccc24981d}{48}" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; overflow-wrap: break-word; white-space: pre-wrap; font-weight: normal; font-style: normal; vertical-align: baseline; font-kerning: none; background-color: transparent; color: windowtext; text-align: left; text-indent: 0px;"><span data-contrast="auto" xml:lang="EN-GB" lang="EN-GB" class="TextRun SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; font-variant-ligatures: none !important; font-size: 12pt; font-style: italic; text-decoration: none; line-height: 22.0875px; font-family: Aptos, Aptos_EmbeddedFont, Aptos_MSFontService, sans-serif; font-weight: normal;"><span class="NormalTextRun SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text;">Italic</span></span><span class="EOP SCXW204127278 BCX0" data-ccp-props="{}" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; font-size: 12pt; line-height: 22.0875px; font-family: Aptos, Aptos_EmbeddedFont, Aptos_MSFontService, sans-serif;"> </span></p></li></ol></div><div class="ListContainerWrapper SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; position: relative; color: rgb(0, 0, 0); font-family: Aptos, Aptos_MSFontService, sans-serif; font-size: 16px; font-style: normal; font-variant-ligatures: normal; font-variant-caps: normal; font-weight: 400; letter-spacing: normal; orphans: 2; text-align: left; text-indent: 0px; text-transform: none; widows: 2; word-spacing: 0px; -webkit-text-stroke-width: 0px; white-space: normal; background-color: rgb(255, 255, 255); text-decoration-thickness: initial; text-decoration-style: initial; text-decoration-color: initial;"><ol class="NumberListStyle1 SCXW204127278 BCX0" role="list" start="2" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; cursor: text; list-style-type: decimal; overflow: visible;"><li aria-setsize="-1" data-leveltext="%1." data-font="" data-listid="3" data-list-defn-props="{&quot;335552541&quot;:0,&quot;335559685&quot;:720,&quot;335559991&quot;:360,&quot;469769242&quot;:[65533,0,46],&quot;469777803&quot;:&quot;left&quot;,&quot;469777804&quot;:&quot;%1.&quot;,&quot;469777815&quot;:&quot;hybridMultilevel&quot;}" data-aria-posinset="2" data-aria-level="1" role="listitem" class="OutlineElement Ltr SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px 0px 0px 24px; padding: 0px; user-select: text; clear: both; cursor: text; overflow: visible; position: relative; direction: ltr; display: block; font-size: 12pt; font-family: Aptos, Aptos_MSFontService, sans-serif; vertical-align: baseline;"><p class="Paragraph SCXW204127278 BCX0" paraid="1266616274" paraeid="{dc225749-75de-40ef-a9ba-ce0ccc24981d}{54}" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; overflow-wrap: break-word; white-space: pre-wrap; font-weight: normal; font-style: normal; vertical-align: baseline; font-kerning: none; background-color: transparent; color: windowtext; text-align: left; text-indent: 0px;"><span data-contrast="auto" xml:lang="EN-GB" lang="EN-GB" class="TextRun MacChromeBold SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; -webkit-font-smoothing: antialiased; font-variant-ligatures: none !important; font-size: 12pt; font-style: normal; text-decoration: none; line-height: 22.0875px; font-family: Aptos, Aptos_EmbeddedFont, Aptos_MSFontService, sans-serif; font-weight: bold;"><span class="NormalTextRun SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text;">Bold</span></span><span class="EOP SCXW204127278 BCX0" data-ccp-props="{}" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; font-size: 12pt; line-height: 22.0875px; font-family: Aptos, Aptos_EmbeddedFont, Aptos_MSFontService, sans-serif;"> </span></p></li></ol></div><div class="ListContainerWrapper SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; position: relative; color: rgb(0, 0, 0); font-family: Aptos, Aptos_MSFontService, sans-serif; font-size: 16px; font-style: normal; font-variant-ligatures: normal; font-variant-caps: normal; font-weight: 400; letter-spacing: normal; orphans: 2; text-align: left; text-indent: 0px; text-transform: none; widows: 2; word-spacing: 0px; -webkit-text-stroke-width: 0px; white-space: normal; background-color: rgb(255, 255, 255); text-decoration-thickness: initial; text-decoration-style: initial; text-decoration-color: initial;"><ol class="NumberListStyle1 SCXW204127278 BCX0" role="list" start="3" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; cursor: text; list-style-type: decimal; overflow: visible;"><li aria-setsize="-1" data-leveltext="%1." data-font="" data-listid="3" data-list-defn-props="{&quot;335552541&quot;:0,&quot;335559685&quot;:720,&quot;335559991&quot;:360,&quot;469769242&quot;:[65533,0,46],&quot;469777803&quot;:&quot;left&quot;,&quot;469777804&quot;:&quot;%1.&quot;,&quot;469777815&quot;:&quot;hybridMultilevel&quot;}" data-aria-posinset="3" data-aria-level="1" role="listitem" class="OutlineElement Ltr SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px 0px 0px 24px; padding: 0px; user-select: text; clear: both; cursor: text; overflow: visible; position: relative; direction: ltr; display: block; font-size: 12pt; font-family: Aptos, Aptos_MSFontService, sans-serif; vertical-align: baseline;"><p class="Paragraph SCXW204127278 BCX0" paraid="2141762432" paraeid="{dc225749-75de-40ef-a9ba-ce0ccc24981d}{60}" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; overflow-wrap: break-word; white-space: pre-wrap; font-weight: normal; font-style: normal; vertical-align: baseline; font-kerning: none; background-color: transparent; color: windowtext; text-align: left; text-indent: 0px;"><span data-contrast="auto" xml:lang="EN-GB" lang="EN-GB" class="TextRun SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; font-variant-ligatures: none !important; font-size: 12pt; font-style: normal; text-decoration: none; line-height: 22.0875px; font-family: Aptos, Aptos_EmbeddedFont, Aptos_MSFontService, sans-serif; font-weight: normal;"><span class="NormalTextRun SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text;">Unformatted</span></span><span class="EOP SCXW204127278 BCX0" data-ccp-props="{}" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; font-size: 12pt; line-height: 22.0875px; font-family: Aptos, Aptos_EmbeddedFont, Aptos_MSFontService, sans-serif;"> </span></p></li></ol></div><div class="ListContainerWrapper SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; position: relative; color: rgb(0, 0, 0); font-family: Aptos, Aptos_MSFontService, sans-serif; font-size: 16px; font-style: normal; font-variant-ligatures: normal; font-variant-caps: normal; font-weight: 400; letter-spacing: normal; orphans: 2; text-align: left; text-indent: 0px; text-transform: none; widows: 2; word-spacing: 0px; -webkit-text-stroke-width: 0px; white-space: normal; background-color: rgb(255, 255, 255); text-decoration-thickness: initial; text-decoration-style: initial; text-decoration-color: initial;"><ol class="NumberListStyle1 SCXW204127278 BCX0" role="list" start="4" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; cursor: text; list-style-type: decimal; overflow: visible;"><li aria-setsize="-1" data-leveltext="%1." data-font="" data-listid="3" data-list-defn-props="{&quot;335552541&quot;:0,&quot;335559685&quot;:720,&quot;335559991&quot;:360,&quot;469769242&quot;:[65533,0,46],&quot;469777803&quot;:&quot;left&quot;,&quot;469777804&quot;:&quot;%1.&quot;,&quot;469777815&quot;:&quot;hybridMultilevel&quot;}" data-aria-posinset="4" data-aria-level="1" role="listitem" class="OutlineElement Ltr SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px 0px 0px 24px; padding: 0px; user-select: text; clear: both; cursor: text; overflow: visible; position: relative; direction: ltr; display: block; font-size: 12pt; font-family: Aptos, Aptos_MSFontService, sans-serif; vertical-align: baseline;"><p class="Paragraph SCXW204127278 BCX0" paraid="400977494" paraeid="{dc225749-75de-40ef-a9ba-ce0ccc24981d}{66}" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; overflow-wrap: break-word; white-space: pre-wrap; font-weight: normal; font-style: normal; vertical-align: baseline; font-kerning: none; background-color: transparent; color: windowtext; text-align: left; text-indent: 0px;"><span data-contrast="auto" xml:lang="EN-GB" lang="EN-GB" class="TextRun Strikethrough SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; font-variant-ligatures: none !important; font-size: 12pt; font-style: normal; text-decoration: line-through; line-height: 22.0875px; font-family: Aptos, Aptos_EmbeddedFont, Aptos_MSFontService, sans-serif; font-weight: normal;"><span class="NormalTextRun SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text;">Strikethrough</span></span><span class="EOP SCXW204127278 BCX0" data-ccp-props="{}" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; font-size: 12pt; line-height: 22.0875px; font-family: Aptos, Aptos_EmbeddedFont, Aptos_MSFontService, sans-serif;"> </span></p></li></ol></div><div class="ListContainerWrapper SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; position: relative; color: rgb(0, 0, 0); font-family: Aptos, Aptos_MSFontService, sans-serif; font-size: 16px; font-style: normal; font-variant-ligatures: normal; font-variant-caps: normal; font-weight: 400; letter-spacing: normal; orphans: 2; text-align: left; text-indent: 0px; text-transform: none; widows: 2; word-spacing: 0px; -webkit-text-stroke-width: 0px; white-space: normal; background-color: rgb(255, 255, 255); text-decoration-thickness: initial; text-decoration-style: initial; text-decoration-color: initial;"><ol class="NumberListStyle1 SCXW204127278 BCX0" role="list" start="5" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; cursor: text; list-style-type: decimal; overflow: visible;"><li aria-setsize="-1" data-leveltext="%1." data-font="" data-listid="3" data-list-defn-props="{&quot;335552541&quot;:0,&quot;335559685&quot;:720,&quot;335559991&quot;:360,&quot;469769242&quot;:[65533,0,46],&quot;469777803&quot;:&quot;left&quot;,&quot;469777804&quot;:&quot;%1.&quot;,&quot;469777815&quot;:&quot;hybridMultilevel&quot;}" data-aria-posinset="5" data-aria-level="1" role="listitem" class="OutlineElement Ltr SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px 0px 0px 24px; padding: 0px; user-select: text; clear: both; cursor: text; overflow: visible; position: relative; direction: ltr; display: block; font-size: 12pt; font-family: Aptos, Aptos_MSFontService, sans-serif; vertical-align: baseline;"><p class="Paragraph SCXW204127278 BCX0" paraid="1929898719" paraeid="{dc225749-75de-40ef-a9ba-ce0ccc24981d}{72}" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; overflow-wrap: break-word; white-space: pre-wrap; font-weight: normal; font-style: normal; vertical-align: baseline; font-kerning: none; background-color: transparent; color: windowtext; text-align: left; text-indent: 0px;"><span data-contrast="auto" xml:lang="EN-GB" lang="EN-GB" class="TextRun Underlined SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; font-variant-ligatures: none !important; font-size: 12pt; font-style: normal; text-decoration: underline; line-height: 22.0875px; font-family: Aptos, Aptos_EmbeddedFont, Aptos_MSFontService, sans-serif; font-weight: normal;"><span class="NormalTextRun SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text;">Underlined</span></span><span class="EOP SCXW204127278 BCX0" data-ccp-props="{}" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; font-size: 12pt; line-height: 22.0875px; font-family: Aptos, Aptos_EmbeddedFont, Aptos_MSFontService, sans-serif;"> </span></p></li></ol></div><div class="ListContainerWrapper SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; position: relative; color: rgb(0, 0, 0); font-family: Aptos, Aptos_MSFontService, sans-serif; font-size: 16px; font-style: normal; font-variant-ligatures: normal; font-variant-caps: normal; font-weight: 400; letter-spacing: normal; orphans: 2; text-align: left; text-indent: 0px; text-transform: none; widows: 2; word-spacing: 0px; -webkit-text-stroke-width: 0px; white-space: normal; background-color: rgb(255, 255, 255); text-decoration-thickness: initial; text-decoration-style: initial; text-decoration-color: initial;"><ol class="NumberListStyle1 SCXW204127278 BCX0" role="list" start="6" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; cursor: text; list-style-type: decimal; overflow: visible;"><li aria-setsize="-1" data-leveltext="%1." data-font="" data-listid="3" data-list-defn-props="{&quot;335552541&quot;:0,&quot;335559685&quot;:720,&quot;335559991&quot;:360,&quot;469769242&quot;:[65533,0,46],&quot;469777803&quot;:&quot;left&quot;,&quot;469777804&quot;:&quot;%1.&quot;,&quot;469777815&quot;:&quot;hybridMultilevel&quot;}" data-aria-posinset="6" data-aria-level="1" role="listitem" class="OutlineElement Ltr SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px 0px 0px 24px; padding: 0px; user-select: text; clear: both; cursor: text; overflow: visible; position: relative; direction: ltr; display: block; font-size: 12pt; font-family: Aptos, Aptos_MSFontService, sans-serif; vertical-align: baseline;"><p class="Paragraph SCXW204127278 BCX0" paraid="1763241731" paraeid="{dc225749-75de-40ef-a9ba-ce0ccc24981d}{78}" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; overflow-wrap: break-word; white-space: pre-wrap; font-weight: normal; font-style: normal; vertical-align: baseline; font-kerning: none; background-color: transparent; color: windowtext; text-align: left; text-indent: 0px;"><span data-contrast="auto" xml:lang="EN-GB" lang="EN-GB" class="TextRun EmptyTextRun SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; font-variant-ligatures: none !important; font-size: 12pt; line-height: 22.0875px; font-family: Aptos, Aptos_EmbeddedFont, Aptos_MSFontService, sans-serif;"></span><a class="Hyperlink SCXW204127278 BCX0" href="https://matrix.org/" target="_blank" rel="noreferrer noopener" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; cursor: text; text-decoration: none; color: inherit;"><span data-contrast="none" xml:lang="EN-GB" lang="EN-GB" class="TextRun Underlined SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; font-variant-ligatures: none !important; color: rgb(70, 120, 134); font-size: 12pt; font-style: normal; text-decoration: underline; line-height: 22.0875px; font-family: Aptos, Aptos_EmbeddedFont, Aptos_MSFontService, sans-serif; font-weight: normal;"><span class="NormalTextRun SCXW204127278 BCX0" data-ccp-charstyle="Hyperlink" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text;">Linked</span></span></a><span data-contrast="auto" xml:lang="EN-GB" lang="EN-GB" class="TextRun EmptyTextRun SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; font-variant-ligatures: none !important; font-size: 12pt; font-style: normal; text-decoration: none; line-height: 22.0875px; font-family: Aptos, Aptos_EmbeddedFont, Aptos_MSFontService, sans-serif; font-weight: normal;"></span><span class="EOP SCXW204127278 BCX0" data-ccp-props="{}" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; font-size: 12pt; line-height: 22.0875px; font-family: Aptos, Aptos_EmbeddedFont, Aptos_MSFontService, sans-serif;"> </span></p></li></ol></div><div class="ListContainerWrapper SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; position: relative; color: rgb(0, 0, 0); font-family: Aptos, Aptos_MSFontService, sans-serif; font-size: 16px; font-style: normal; font-variant-ligatures: normal; font-variant-caps: normal; font-weight: 400; letter-spacing: normal; orphans: 2; text-align: left; text-indent: 0px; text-transform: none; widows: 2; word-spacing: 0px; -webkit-text-stroke-width: 0px; white-space: normal; background-color: rgb(255, 255, 255); text-decoration-thickness: initial; text-decoration-style: initial; text-decoration-color: initial;"><ul class="BulletListStyle2 SCXW204127278 BCX0" role="list" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; cursor: text; font-family: verdana; overflow: visible; list-style-type: circle;"><li aria-setsize="-1" data-leveltext="o" data-font="Courier New" data-listid="3" data-list-defn-props="{&quot;335552541&quot;:1,&quot;335559685&quot;:1440,&quot;335559991&quot;:360,&quot;469769226&quot;:&quot;Courier New&quot;,&quot;469769242&quot;:[9675],&quot;469777803&quot;:&quot;left&quot;,&quot;469777804&quot;:&quot;o&quot;,&quot;469777815&quot;:&quot;hybridMultilevel&quot;}" data-aria-posinset="1" data-aria-level="2" role="listitem" class="OutlineElement Ltr SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px 0px 0px 72px; padding: 0px; user-select: text; clear: both; cursor: text; overflow: visible; position: relative; direction: ltr; display: block; font-size: 12pt; font-family: Aptos, Aptos_MSFontService, sans-serif; vertical-align: baseline;"><p class="Paragraph SCXW204127278 BCX0" paraid="274900590" paraeid="{dc225749-75de-40ef-a9ba-ce0ccc24981d}{85}" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; overflow-wrap: break-word; white-space: pre-wrap; font-weight: normal; font-style: normal; vertical-align: baseline; font-kerning: none; background-color: transparent; color: windowtext; text-align: left; text-indent: 0px;"><span data-contrast="auto" xml:lang="EN-GB" lang="EN-GB" class="TextRun SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; font-variant-ligatures: none !important; font-size: 12pt; font-style: normal; text-decoration: none; line-height: 22.0875px; font-family: Aptos, Aptos_EmbeddedFont, Aptos_MSFontService, sans-serif; font-weight: normal;"><span class="NormalTextRun SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text;">N</span><span class="NormalTextRun SCXW204127278 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text;">ested</span></span></p></li></ul></div>"#;

#[cfg(feature = "sys")]
mod sys {
    use std::fmt;

    use matrix_mentions::Mention;

    use super::super::padom_node::PaDomNode;
    use super::super::PaNodeContainer;
    use super::super::{PaDom, PaDomCreationError, PaDomCreator};
    use super::*;
    use crate::dom::nodes::dom_node::DomNodeKind;
    use crate::dom::nodes::dom_node::DomNodeKind::CodeBlock;
    use crate::dom::nodes::{ContainerNode, DomNode};
    use crate::dom::parser::sys::PaNodeText;
    use crate::ListType;

    pub(super) struct HtmlParser {
        current_path: Vec<DomNodeKind>,
    }
    impl HtmlParser {
        pub(super) fn default() -> Self {
            Self {
                current_path: Vec::new(),
            }
        }

        pub(super) fn parse<S>(
            &mut self,
            html: &str,
        ) -> Result<Dom<S>, HtmlParseError>
        where
            S: UnicodeString,
        {
            self.parse_internal(html, HtmlSource::Matrix)
        }

        pub(super) fn parse_from_source<S>(
            &mut self,
            html: &str,
            source: HtmlSource,
        ) -> Result<Dom<S>, HtmlParseError>
        where
            S: UnicodeString,
        {
            self.parse_internal(html, source)
        }

        pub(super) fn parse_internal<S>(
            &mut self,
            html: &str,
            html_source: HtmlSource,
        ) -> Result<Dom<S>, HtmlParseError>
        where
            S: UnicodeString,
        {
            let pa_dom = PaDomCreator::parse(html).map_err(|err| {
                self.padom_creation_error_to_html_parse_error(err)
            })?;

            let dom =
                self.padom_to_dom(pa_dom, html_source).map_err(|err| {
                    HtmlParseError {
                        parse_errors: vec![err.to_string()],
                    }
                })?;
            let dom_blocks_done = post_process_blocks(dom);
            let dom_inline_blocks_done =
                post_process_for_block_and_inline_siblings(dom_blocks_done);
            let dom_adjacted_text_done =
                post_process_for_adjacent_text(dom_inline_blocks_done);
            Ok(dom_adjacted_text_done)
        }

        /// Convert a [PaDom] into a [Dom].
        ///
        /// [PaDom] is purely used within the parsing process (using html5ever) - in it,
        /// parents refer to their children by handles, and all the nodes are owned in
        /// a big list held by the PaDom itself. PaDoms may also contain garbage nodes
        /// that were created during parsing but are no longer needed. A garbage
        /// collection method was written for testing and is inside padom_creator's
        /// test code. The conversion process here ignores garbage nodes, so they do
        /// not appear in the final Dom.
        ///
        /// [Dom] is for general use. Parent nodes own their children, and Dom may be
        /// cloned, compared, and converted into an HTML string.
        fn padom_to_dom<S>(
            &mut self,
            padom: PaDom,
            html_source: HtmlSource,
        ) -> Result<Dom<S>, Error>
        where
            S: UnicodeString,
        {
            let mut ret = Dom::new(Vec::new());
            let doc = ret.document_mut();

            if let PaDomNode::Document(padoc) = padom.get_document() {
                self.convert(&padom, padoc, doc, html_source)?;
            } else {
                return Err(Error::NoBody);
            }
            Ok(ret)
        }

        /// Copy all panode's information into node.
        fn convert<S>(
            &mut self,
            padom: &PaDom,
            panode: &PaNodeContainer,
            node: &mut ContainerNode<S>,
            html_source: HtmlSource,
        ) -> Result<(), Error>
        where
            S: UnicodeString,
        {
            for child_handle in &panode.children {
                let child = padom.get_node(child_handle);
                match child {
                    PaDomNode::Container(child) => {
                        self.convert_container(
                            padom,
                            child,
                            node,
                            html_source,
                        )?;
                    }
                    PaDomNode::Document(_) => {
                        panic!("Found a document inside a document!")
                    }
                    PaDomNode::Text(text) => {
                        // Special case for code block, translate '\n' into <br /> nodes
                        let is_inside_code_block =
                            self.current_path.contains(&CodeBlock);
                        let is_only_child_in_parent =
                            panode.children.len() == 1;
                        convert_text(
                            &text.content,
                            node,
                            is_inside_code_block,
                            is_only_child_in_parent,
                        );
                    }
                }
            }
            Ok(())
        }

        /// Copy all panode's information into node (now we know it's a container).
        fn convert_container<S>(
            &mut self,
            padom: &PaDom,
            child: &PaNodeContainer,
            node_in: &mut ContainerNode<S>,
            html_source: HtmlSource,
        ) -> Result<(), Error>
        where
            S: UnicodeString,
        {
            let cur_path_idx = self.current_path.len();
            let tag = child.name.local.as_ref();
            let mut invalid_node_error: Option<Error> = None;
            let mut skip_children: bool = false;
            let mut node = node_in.clone();
            if node.is_list()
                && tag != "li"
                && html_source != HtmlSource::GoogleDoc
            {
                // If we are inside a list, we can only have list items.
                invalid_node_error = Some(Error::InvalidListItemNode);
                skip_children = true;
            }

            if invalid_node_error.is_none() {
                match tag {
                    "b" | "code" | "del" | "em" | "i" | "strong" | "u" => {
                        let formatting_node = Self::new_formatting(tag);
                        if tag == "code"
                            && self.current_path.contains(&CodeBlock)
                        {
                            self.convert_children(
                                padom,
                                child,
                                Some(&mut node),
                                html_source,
                            )?;
                        } else {
                            self.current_path.push(formatting_node.kind());
                            node.append_child(formatting_node);
                            self.convert_children(
                                padom,
                                child,
                                last_container_mut_in(&mut node),
                                html_source,
                            )?;
                            self.current_path.remove(cur_path_idx);
                        }
                    }
                    "span" => 'span: {
                        if html_source == HtmlSource::Matrix {
                            invalid_node_error =
                                Some(Error::UnknownNode(tag.to_string()));
                            break 'span;
                        }

                        // For external sources, we check for common formatting styles for spans
                        // and convert them to appropriate formatting nodes.
                        let mut formatting_tag = None;
                        if child.contains_style("font-weight", "bold")
                            || child.contains_style("font-weight", "700")
                        {
                            formatting_tag = Some("b");
                        } else if child.contains_style("font-style", "italic") {
                            formatting_tag = Some("i");
                        } else if child
                            .contains_style("text-decoration", "underline")
                        {
                            formatting_tag = Some("u");
                        } else if child
                            .contains_style("text-decoration", "line-through")
                        {
                            formatting_tag = Some("del");
                        }

                        if let Some(tag) = formatting_tag {
                            let formatting_node = Self::new_formatting(tag);
                            self.current_path.push(formatting_node.kind());
                            node.append_child(formatting_node);
                            self.convert_children(
                                padom,
                                child,
                                last_container_mut_in(&mut node),
                                html_source,
                            )?;
                            self.current_path.remove(cur_path_idx);
                        } else {
                            // If no formatting tag was found, just skip and convert the children
                            invalid_node_error =
                                Some(Error::UnknownNode(tag.to_string()));
                        }
                    }
                    "br" => {
                        node.append_child(Self::new_line_break());
                    }
                    "ol" | "ul" => 'list: {
                        let target_node = if node.is_list() {
                            // Google docs adds nested lists as children of the list node, this breaks our invariants.
                            // For the google docs case, we can add the nested list to the last list item instead.
                            if html_source != HtmlSource::GoogleDoc
                                || node.last_child_mut().is_none()
                                || !node
                                    .last_child_mut()
                                    .unwrap()
                                    .is_list_item()
                            {
                                // If source is not Google Docs or the last child is not a list item, we return an error.
                                invalid_node_error =
                                    Some(Error::InvalidListItemNode);
                                break 'list;
                            }
                            node.last_child_mut()
                                .unwrap()
                                .as_container_mut()
                                .unwrap()
                        } else {
                            &mut node
                        };
                        self.current_path.push(DomNodeKind::List);
                        if tag == "ol" {
                            let custom_start = child
                                .get_attr("start")
                                .and_then(|start| start.parse::<usize>().ok());
                            target_node.append_child(Self::new_ordered_list(
                                custom_start,
                            ));
                        } else {
                            target_node
                                .append_child(Self::new_unordered_list());
                        }
                        self.convert_children(
                            padom,
                            child,
                            last_container_mut_in(target_node),
                            html_source,
                        )?;
                        self.current_path.remove(cur_path_idx);
                    }
                    "li" => 'li: {
                        if !node.is_list() {
                            invalid_node_error = Some(Error::ParentNotAList);
                            break 'li;
                        }
                        self.current_path.push(DomNodeKind::ListItem);
                        node.append_child(Self::new_list_item());
                        self.convert_children(
                            padom,
                            child,
                            last_container_mut_in(&mut node),
                            html_source,
                        )?;
                        self.current_path.remove(cur_path_idx);
                    }
                    "a" => {
                        let is_mention = child.attrs.iter().any(|(k, v)| {
                            k == &String::from("href")
                                && Mention::is_valid_uri(v)
                        });

                        let text =
                            child.children.first().map(|gc| padom.get_node(gc));
                        let text = match text {
                            Some(PaDomNode::Text(text)) => Some(text),
                            _ => None,
                        };

                        match (is_mention, text) {
                            (true, Some(text)) => {
                                self.current_path.push(DomNodeKind::Mention);
                                let mention = Self::new_mention(child, text);
                                node.append_child(mention);
                            }
                            _ => {
                                self.current_path.push(DomNodeKind::Link);
                                let link = Self::new_link(child);
                                node.append_child(link);
                                self.convert_children(
                                    padom,
                                    child,
                                    last_container_mut_in(&mut node),
                                    html_source,
                                )?;
                            }
                        }
                        self.current_path.remove(cur_path_idx);
                    }
                    "pre" => {
                        self.current_path.push(DomNodeKind::CodeBlock);
                        node.append_child(Self::new_code_block());
                        self.convert_children(
                            padom,
                            child,
                            last_container_mut_in(&mut node),
                            html_source,
                        )?;
                        self.current_path.remove(cur_path_idx);
                    }
                    "blockquote" => {
                        self.current_path.push(DomNodeKind::Quote);
                        node.append_child(Self::new_quote());
                        self.convert_children(
                            padom,
                            child,
                            last_container_mut_in(&mut node),
                            html_source,
                        )?;

                        self.current_path.remove(cur_path_idx);
                    }
                    "html" => {
                        // Skip the html tag - add its children to the
                        // current node directly.
                        self.convert(padom, child, &mut node, html_source)?;
                    }
                    "p" => {
                        self.current_path.push(DomNodeKind::Paragraph);
                        node.append_child(Self::new_paragraph());
                        self.convert_children(
                            padom,
                            child,
                            last_container_mut_in(&mut node),
                            html_source,
                        )?;
                        self.current_path.remove(cur_path_idx);
                    }
                    _ => {
                        invalid_node_error =
                            Some(Error::UnknownNode(tag.to_string()));
                    }
                };
            }

            if let Some(err) = invalid_node_error {
                if html_source == HtmlSource::Matrix {
                    return Err(err);
                } else if !skip_children {
                    // If the source is not Matrix and we haven't explicitly flagged to skip the children continue to parse them.
                    self.convert(padom, child, &mut node, html_source)?;
                }
            }
            *node_in = node;
            Ok(())
        }

        /// Recurse into panode's children and convert them too
        fn convert_children<S>(
            &mut self,
            padom: &PaDom,
            child: &PaNodeContainer,
            new_node: Option<&mut ContainerNode<S>>,
            html_source: HtmlSource,
        ) -> Result<(), Error>
        where
            S: UnicodeString,
        {
            if let Some(new_node) = new_node {
                self.convert(padom, child, new_node, html_source)?;
            } else {
                panic!("Container became non-container!");
            }
            Ok(())
        }

        /// Create a formatting node
        fn new_formatting<S>(tag: &str) -> DomNode<S>
        where
            S: UnicodeString,
        {
            DomNode::Container(ContainerNode::new_formatting_from_tag(
                tag.into(),
                Vec::new(),
            ))
        }

        /// Create a br node
        fn new_line_break<S>() -> DomNode<S>
        where
            S: UnicodeString,
        {
            DomNode::new_line_break()
        }

        /// Create a link node
        fn new_link<S>(child: &PaNodeContainer) -> DomNode<S>
        where
            S: UnicodeString,
        {
            let attributes = child
                .attrs
                .iter()
                .filter(|(k, _)| k != &String::from("href"))
                .map(|(k, v)| (k.as_str().into(), v.as_str().into()))
                .collect();
            DomNode::Container(ContainerNode::new_link(
                child.get_attr("href").unwrap_or("").into(),
                Vec::new(),
                attributes,
            ))
        }

        fn new_mention<S>(
            link: &PaNodeContainer,
            text: &PaNodeText,
        ) -> DomNode<S>
        where
            S: UnicodeString,
        {
            let text = &text.content;

            // creating a mention node could fail if the uri is invalid
            let creation_result = DomNode::new_mention(
                link.get_attr("href").unwrap_or("").into(),
                text.as_str().into(),
                // custom attributes are not required when cfg feature != "js"
                vec![],
            );

            match creation_result {
                Ok(node) => DomNode::Mention(node),
                Err(_) => Self::new_link(link),
            }
        }

        /// Create an unordered list node
        fn new_unordered_list<S>() -> DomNode<S>
        where
            S: UnicodeString,
        {
            DomNode::Container(ContainerNode::new_list(
                ListType::Unordered,
                Vec::new(),
                None,
            ))
        }

        /// Create an ordered list node
        fn new_ordered_list<S>(custom_start: Option<usize>) -> DomNode<S>
        where
            S: UnicodeString,
        {
            DomNode::Container(ContainerNode::new_list(
                ListType::Ordered,
                Vec::new(),
                custom_start.map(|start| {
                    vec![("start".into(), start.to_string().into())]
                }),
            ))
        }

        /// Create a list item node
        fn new_list_item<S>() -> DomNode<S>
        where
            S: UnicodeString,
        {
            DomNode::Container(ContainerNode::new_list_item(Vec::new()))
        }

        /// Create a code block node
        fn new_code_block<S>() -> DomNode<S>
        where
            S: UnicodeString,
        {
            DomNode::Container(ContainerNode::new_code_block(Vec::new()))
        }

        /// Create a quote node
        fn new_quote<S>() -> DomNode<S>
        where
            S: UnicodeString,
        {
            DomNode::Container(ContainerNode::new_quote(Vec::new()))
        }

        /// Create a paragraph
        fn new_paragraph<S>() -> DomNode<S>
        where
            S: UnicodeString,
        {
            DomNode::Container(ContainerNode::new_paragraph(Vec::new()))
        }

        fn padom_creation_error_to_html_parse_error(
            &mut self,
            e: PaDomCreationError,
        ) -> HtmlParseError {
            HtmlParseError {
                parse_errors: e.parse_errors,
            }
        }
    }

    enum Error {
        NoBody,
        UnknownNode(String),
        InvalidListItemNode,
        ParentNotAList,
    }

    impl fmt::Display for Error {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::NoBody => {
                    write!(
                        formatter,
                        "The `Document` does not have a `<body>` element"
                    )
                }
                Self::UnknownNode(node_name) => {
                    write!(formatter, "Node `{node_name}` is not supported")
                }
                Self::InvalidListItemNode => {
                    write!(
                        formatter,
                        "Invalid list item node: a list must only contain list items"
                    )
                }
                Self::ParentNotAList => {
                    write!(formatter, "Parent node is not a list")
                }
            }
        }
    }

    #[cfg(test)]
    mod test {
        use crate::dom::parser::parse::sys::HtmlParser;
        use crate::dom::Dom;
        use indoc::indoc;
        use speculoos::{assert_that, AssertionFailure, Spec};
        use widestring::Utf16String;

        use super::*;
        use crate::tests::testutils_composer_model::restore_whitespace;
        use crate::{ToHtml, ToMarkdown, ToTree};

        trait Roundtrips<T> {
            fn roundtrips(&self);
        }

        impl<'s, T> Roundtrips<T> for Spec<'s, T>
        where
            T: AsRef<str>,
        {
            fn roundtrips(&self) {
                let subject = self.subject.as_ref();
                let dom = parse::<Utf16String>(subject).unwrap();

                // After parsing all our invariants should be satisifed
                dom.explicitly_assert_invariants();

                let output = restore_whitespace(&dom.to_html().to_string());
                if output != subject {
                    AssertionFailure::from_spec(self)
                        .with_expected(String::from(subject))
                        .with_actual(output)
                        .fail();
                }
            }
        }

        #[test]
        fn parse_plain_text() {
            assert_that!("some text").roundtrips();
        }

        #[test]
        fn parse_simple_tag() {
            assert_that!("<strong>sdfds</strong>").roundtrips();
        }

        #[test]
        fn parse_tag_with_surrounding_text() {
            assert_that!("before <strong> within </strong> after").roundtrips();
            assert_that!("before<strong>within</strong>after").roundtrips();
        }

        #[test]
        fn parse_nested_tags() {
            assert_that!("<b><em>ZZ</em></b>").roundtrips();
            assert_that!("X<b>Y<em>ZZ</em>0</b>1").roundtrips();
            assert_that!(" X <b> Y <em> ZZ </em> 0 </b> 1 ").roundtrips();
        }

        #[test]
        fn parse_tags_with_attributes() {
            assert_that!(r#"<b><a href="http://example.com">ZZ</a></b>"#)
                .roundtrips();
        }

        #[test]
        fn parse_br_tag() {
            let html = "<br />";
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                ├>p
                └>p
                "#}
            );
        }

        #[test]
        fn parse_code_block_keeps_internal_code_tag() {
            let html = "\
                <p>foo</p>\
                <pre><code>Some code</code></pre>\
                <p>bar</p>";
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                ├>p
                │ └>"foo"
                ├>codeblock
                │ └>p
                │   └>"Some code"
                └>p
                  └>"bar"
                "#}
            );
        }

        #[test]
        fn parse_line_breaks_none() {
            let html = r#"foo"#;
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                └>"foo"
                "#}
            );
        }

        #[test]
        fn parse_line_breaks_br_end() {
            let html = r#"foo<br />"#;
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                ├>p
                │ └>"foo"
                └>p
                "#}
            );
        }

        #[test]
        fn parse_line_breaks_br_start() {
            let html = r#"<br />foo"#;
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                ├>p
                └>p
                  └>"foo"
                "#}
            );
        }

        #[test]
        fn parse_line_breaks_br_before_inline_format() {
            let html = "abc<br /><strong>def<br />gh</strong>ijk";
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                ├>p
                │ └>"abc"
                ├>p
                │ └>strong
                │   └>"def"
                └>p
                  ├>strong
                  │ └>"gh"
                  └>"ijk"
                "#}
            );
        }

        #[test]
        fn parse_line_breaks_br_before_p() {
            let html = "abc<br /><p>def<br />gh</p>ijk";
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                ├>p
                │ └>"abc"
                ├>p
                │ └>"def"
                ├>p
                │ └>"gh"
                └>p
                  └>"ijk"
                "#}
            );
        }

        #[test]
        fn parse_line_breaks_br_in_bold() {
            let html = r#"<b>foo<br /></b>"#;
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                ├>p
                │ └>b
                │   └>"foo"
                └>p
                  └>b
                "#}
            );
        }

        #[test]
        fn parse_line_breaks_br_in_code() {
            let html = r#"<pre><code>foo<br /></code></pre>"#;
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                └>codeblock
                  ├>p
                  │ └>"foo"
                  └>p
                "#}
            );
        }

        #[test]
        fn parse_line_breaks_br_in_quote() {
            let html = r#"<blockquote>foo<br />bar<br /></blockquote>"#;
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                └>blockquote
                  ├>p
                  │ └>"foo"
                  ├>p
                  │ └>"bar"
                  └>p
                "#}
            );
        }

        #[test]
        fn parse_line_breaks_br_in_list() {
            let html = r#"<ul><li>foo<br />bar<br /><p>baz</p></li></ul>"#;
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                └>ul
                  └>li
                    ├>p
                    │ └>"foo"
                    ├>p
                    │ └>"bar"
                    └>p
                      └>"baz"
                "#}
            );
        }

        #[test]
        fn parse_line_breaks_br_in_p() {
            let html = r#"<p>foo<br />bar<br />baz<br /></p>"#;
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                ├>p
                │ └>"foo"
                ├>p
                │ └>"bar"
                ├>p
                │ └>"baz"
                └>p
                "#}
            );
        }

        #[test]
        fn parse_line_breaks_in_nested_p_in_blockquote() {
            let html = r#"<blockquote><p><b>foo<br />bar</b><i>foo<br /></i></p></blockquote>"#;
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                └>blockquote
                  ├>p
                  │ └>b
                  │   └>"foo"
                  ├>p
                  │ ├>b
                  │ │ └>"bar"
                  │ └>i
                  │   └>"foo"
                  └>p
                    └>i
                "#}
            );
        }

        #[test]
        fn parse_line_breaks_in_nested_blocks() {
            let html = r#"<blockquote><p><b>foo<br />bar</b><i>foo<br /></i></p><pre><code><br /></code></pre><ol><li><b>a<br />b</b></li></ol></blockquote>"#;
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                └>blockquote
                  ├>p
                  │ └>b
                  │   └>"foo"
                  ├>p
                  │ ├>b
                  │ │ └>"bar"
                  │ └>i
                  │   └>"foo"
                  ├>p
                  │ └>i
                  ├>codeblock
                  │ ├>p
                  │ └>p
                  └>ol
                    └>li
                      ├>p
                      │ └>b
                      │   └>"a"
                      └>p
                        └>b
                          └>"b"
                "#}
            );
        }

        #[test]
        fn parse_code_block_roundtrips() {
            assert_that!(
                "<p>foo</p><pre><code>Some code</code></pre><p>bar</p>"
            )
            .roundtrips();
        }

        #[test]
        fn parse_code_block_post_processes_it() {
            let mut parser = HtmlParser::default();
            let html = "<pre><code><b>Test\nCode</b></code></pre>";
            let pa_dom = PaDomCreator::parse(html).unwrap();
            let dom: Dom<Utf16String> = parser
                .padom_to_dom(pa_dom, HtmlSource::Matrix)
                .ok()
                .unwrap();
            // First, line breaks are added as placeholders for paragraphs
            assert_eq!(
                dom.to_html().to_string(),
                "<pre><code><b>Test<br />Code</b></code></pre>"
            );
            // Then these line breaks are post-processed and we get the actual paragraphs
            let dom =
                post_process_block_lines(dom, &DomHandle::from_raw(vec![0]));
            assert_eq!(
                dom.to_html().to_string(),
                "<pre><code><b>Test</b>\n<b>Code</b></code></pre>"
            );
        }

        #[test]
        fn parse_quote() {
            assert_that!(
                "<p>foo</p><blockquote><p>A quote</p></blockquote><p>bar</p>"
            )
            .roundtrips();
        }

        #[test]
        fn parse_quote_2() {
            assert_that!(
                "<p>foo</p><blockquote>A quote</blockquote><p>bar</p>"
            )
            .roundtrips();
        }

        #[test]
        fn parse_paragraph() {
            assert_that!("<p>foo</p><p>A paragraph</p><p>bar</p>").roundtrips();
        }

        #[test]
        fn nbsp_chars_are_removed() {
            let html = "\
                <p>\u{A0}</p>\
                <pre><code>\u{A0}\n\u{A0}</code></pre>\
                <p>\u{A0}</p>";
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                ├>p
                ├>codeblock
                │ ├>p
                │ └>p
                └>p
                "#}
            );
        }

        #[test]
        fn nbsp_text_is_removed() {
            let html = "\
                <p>&nbsp;</p>\
                <pre><code>&nbsp;\n&nbsp;</code></pre>\
                <p>&nbsp;</p>";
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                ├>p
                ├>codeblock
                │ ├>p
                │ └>p
                └>p
                "#}
            );
        }

        #[test]
        fn parse_at_room_mentions() {
            let html = "\
                <p>@room hello!</p>\
                <pre><code>@room hello!</code></pre>\
                <p>@room@room</p>";
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                ├>p
                │ ├>mention "@room"
                │ └>" hello!"
                ├>codeblock
                │ └>p
                │   └>"@room hello!"
                └>p
                  ├>mention "@room"
                  └>mention "@room"
                "#}
            );
        }

        #[test]
        fn parse_mentions() {
            let html = r#"<p><a href="https://matrix.to/#/@test:example.org">test</a> hello!</p>"#;
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"

                └>p
                  ├>mention "test", https://matrix.to/#/@test:example.org
                  └>" hello!"
                "#}
            );
        }

        #[test]
        fn parse_nbsp_after_container_keeps_it() {
            let html = r#"<a href="https://matrix.to/#/@test:example.org">test</a>&nbsp;"#;
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"

                  ├>mention "test", https://matrix.to/#/@test:example.org
                  └>" "
                "#}
            );
        }

        #[test]
        fn parse_insert_text_directly_into_a_list() {
            let html = r#"<ul><li>hello</li><b>list item</b></ul>"#;
            let dom: Dom<Utf16String> = HtmlParser::default()
                .parse_from_source(html, HtmlSource::UnknownExternal)
                .unwrap();
            assert_eq!(dom.to_html(), r#"<ul><li>hello</li></ul>"#);
        }

        #[test]
        fn parse_google_doc_rich_text() {
            let dom: Dom<Utf16String> = HtmlParser::default()
                .parse_from_source(
                    GOOGLE_DOC_HTML_PASTEBOARD,
                    HtmlSource::GoogleDoc,
                )
                .unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                └>ol
                  ├>li
                  │ └>p
                  │   └>i
                  │     └>"Italic"
                  ├>li
                  │ └>p
                  │   └>b
                  │     └>"Bold"
                  ├>li
                  │ └>p
                  │   └>"Unformatted"
                  ├>li
                  │ └>p
                  │   └>del
                  │     └>"Strikethrough"
                  ├>li
                  │ └>p
                  │   └>u
                  │     └>"Underlined"
                  └>li
                    ├>p
                    │ └>a "http://matrix.org"
                    │   └>u
                    │     └>"Linked"
                    └>ul
                      └>li
                        └>p
                          └>"Nested"
                "#
                }
            );
            assert_eq!(
                dom.to_markdown().unwrap().to_string(),
                indoc! {r#"
                1. *Italic*
                2. __Bold__
                3. Unformatted
                4. ~~Strikethrough~~
                5. <u>Underlined</u>
                6. [<u>Linked</u>](<http://matrix.org>)
                  * Nested"#
                }
            );
        }

        #[test]
        fn parse_ms_doc_rich_text() {
            let dom: Dom<Utf16String> = HtmlParser::default()
                .parse_from_source(
                    MS_DOC_HTML_PASTEBOARD,
                    HtmlSource::UnknownExternal,
                )
                .unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                ├>ol
                │ ├>li
                │ │ └>p
                │ │   └>i
                │ │     └>"Italic"
                │ ├>li
                │ │ └>p
                │ │   └>b
                │ │     └>"Bold"
                │ ├>li
                │ │ └>p
                │ │   └>"Unformatted"
                │ ├>li
                │ │ └>p
                │ │   └>del
                │ │     └>"Strikethrough"
                │ ├>li
                │ │ └>p
                │ │   └>u
                │ │     └>"Underlined"
                │ └>li
                │   └>p
                │     └>a "https://matrix.org/"
                │       └>u
                │         └>"Linked"
                └>ul
                  └>li
                    └>p
                      └>"Nested"
                "#
                }
            );
        }
    }
}

fn post_process_for_adjacent_text<S: UnicodeString>(mut dom: Dom<S>) -> Dom<S> {
    let text_handles = find_text_nodes(&dom);
    for handle in text_handles.iter().rev() {
        dom = post_process_adjacent_text(dom, handle);
    }
    dom
}

fn find_text_nodes<S: UnicodeString>(dom: &Dom<S>) -> Vec<DomHandle> {
    dom.iter_text().map(|n| n.handle()).collect::<Vec<_>>()
}

fn post_process_adjacent_text<S: UnicodeString>(
    mut dom: Dom<S>,
    handle: &DomHandle,
) -> Dom<S> {
    dom.merge_text_nodes_around(handle);
    dom
}

fn post_process_for_block_and_inline_siblings<S: UnicodeString>(
    mut dom: Dom<S>,
) -> Dom<S> {
    dom.wrap_inline_nodes_into_paragraphs_if_needed(&DomHandle::root());
    dom
}

fn post_process_blocks<S: UnicodeString>(mut dom: Dom<S>) -> Dom<S> {
    let block_handles = find_blocks(&dom);
    for handle in block_handles.iter().rev() {
        dom = post_process_block_lines(dom, handle);
        dom.join_nodes_in_container(handle);
    }
    dom
}

fn find_blocks<S: UnicodeString>(dom: &Dom<S>) -> Vec<DomHandle> {
    dom.iter()
        .filter(|n| n.is_block_node())
        .map(|n| n.handle())
        .collect::<Vec<_>>()
}

// Process block nodes by converting line breaks into paragraphs.
fn post_process_block_lines<S: UnicodeString>(
    mut dom: Dom<S>,
    handle: &DomHandle,
) -> Dom<S> {
    assert!(dom.lookup_node(handle).is_container_node());
    let container_node = dom.lookup_node(handle).as_container().unwrap();
    let last_handle = dom.last_node_handle_in_sub_tree(handle);

    // Collect the positions of all the line breaks and the lines following them
    let (line_breaks, lines) = {
        let mut line_breaks: Vec<Option<DomHandle>> = Vec::new();
        let mut next_lines: Vec<DomHandle> = Vec::new();

        let nodes = dom
            .iter_from_handle(&last_handle)
            .filter(|n| n.is_leaf() && handle.is_ancestor_of(&n.handle()))
            .rev()
            .collect::<Vec<_>>();
        let mut next_handle = if nodes.is_empty() {
            last_handle.clone()
        } else {
            last_handle.next_sibling()
        };

        for node in nodes {
            if node.is_line_break() {
                line_breaks.push(Some(node.handle()));
                next_lines.push(next_handle.clone());
            }
            next_handle = node.handle();
        }

        line_breaks.push(None);
        next_lines.push(next_handle.clone());

        (line_breaks, next_lines)
    };

    // If there were no line breaks we might stop here
    if lines.len() <= 1 // (<= 1 because lines will always contain at least the container)
        // Code blocks require all inline content to be wrapped in a paragraph
        && dom.lookup_node(handle).kind() != DomNodeKind::CodeBlock
    {
        return dom;
    }

    // Create a new node to hold the processed contents if necessary
    let new_node = match container_node.kind() {
        ContainerNodeKind::Paragraph => None,
        _ => Some(container_node.clone_with_new_children(vec![])),
    };

    // Remove each line from the DOM and collect it in a vector
    let contents = {
        let mut contents = Vec::new();
        for (i, line_handle) in lines.iter().enumerate() {
            let mut sub_tree =
                dom.split_sub_tree_from(line_handle, 0, handle.depth());

            if let Some(line_break_handle) = &line_breaks[i] {
                dom.remove(line_break_handle);
            }

            // If the nodes following the line break start with inline nodes,
            // ensure they are wrapped in a paragraph in order to add an
            // implicit line break here.
            group_inline_nodes(sub_tree.document_mut().remove_children())
                .iter()
                .rev()
                .for_each(|n| contents.insert(0, n.clone()));
        }
        contents
    };

    if handle.is_root() {
        return Dom::new(contents);
    }

    let needs_removal = if dom.contains(handle) {
        let block = dom.lookup_node(handle);
        block.is_empty()
    } else {
        false
    };

    if needs_removal {
        dom.remove(handle);
    }

    // Insert the processed contents back into the dom
    match new_node {
        Some(mut n) => {
            n.set_handle(handle.clone());
            n.append_children(contents);
            dom.insert_at(handle, DomNode::Container(n));
        }
        None => {
            dom.insert(handle, contents);
        }
    }
    dom
}

// Group consecutive inline nodes into paragraphs.
//
// This function accepts a list of nodes of any type, inline or block.
// For example: [b, codeblock, b, i, p].
//
// It will first group any inline nodes: [[b], codeblock, [b, i], p].
// And wrap each group in a pararaph: [p, codeblock, p, p].
//
// Always returns at least one empty paragraph.
fn group_inline_nodes<S: UnicodeString>(
    nodes: Vec<DomNode<S>>,
) -> Vec<DomNode<S>> {
    let mut output: Vec<DomNode<S>> = Vec::new();
    let mut cur_group: Vec<DomNode<S>> = Vec::new();

    for node in nodes.clone() {
        if node.is_block_node() {
            // If there are inline elements waiting to be grouped, create a new block with them
            if !cur_group.is_empty() {
                output.push(DomNode::new_paragraph(cur_group.clone()));
                cur_group.clear();
            }

            // Then add the current block
            output.push(node);
        } else {
            cur_group.push(node)
        }
    }

    if !cur_group.is_empty() || output.is_empty() {
        output.push(DomNode::new_paragraph(cur_group));
    }

    output
}

#[cfg(feature = "sys")]
fn last_container_mut_in<S: UnicodeString>(
    node: &mut ContainerNode<S>,
) -> Option<&mut ContainerNode<S>> {
    node.last_child_mut().and_then(|n| n.as_container_mut())
}

fn convert_text<S: UnicodeString>(
    text: &str,
    node: &mut ContainerNode<S>,
    is_inside_code_block: bool,
    is_only_child_in_parent: bool,
) {
    if is_inside_code_block {
        let text_nodes: Vec<_> = text.split('\n').collect();
        let text_nodes_len = text_nodes.len();
        for (i, str) in text_nodes.into_iter().enumerate() {
            let is_nbsp = str == "\u{A0}" || str == "&nbsp;";
            if !str.is_empty() && !is_nbsp {
                node.append_child(DomNode::new_text(str.into()));
            }
            if i + 1 < text_nodes_len {
                node.append_child(DomNode::new_line_break());
            }
        }
    } else {
        let contents = text;
        let is_nbsp = contents == "\u{A0}" || contents == "&nbsp;";
        if is_nbsp && is_only_child_in_parent {
            return;
        }

        // Trim any surrounding indentation
        let surrounding_indent =
            Regex::new(r"^(\s*\n\s*)+|(\s*\n\s*)+$").unwrap();
        let contents = &surrounding_indent.replace_all(contents, "");

        // Replace any internal indentation with a single space
        let internal_indent = Regex::new(r"s*\n\s*").unwrap();
        let contents = &internal_indent.replace_all(contents, " ");

        for (i, part) in contents.split("@room").enumerate() {
            if i > 0 {
                node.append_child(DomNode::Mention(
                    DomNode::new_at_room_mention(vec![]),
                ));
            }
            if !part.is_empty() {
                node.append_child(DomNode::new_text(part.into()));
            }
        }
    }
}

#[cfg(all(feature = "js", target_arch = "wasm32"))]
mod js {
    use super::*;
    use crate::dom::nodes::dom_node::DomNodeKind;
    use crate::dom::nodes::dom_node::DomNodeKind::CodeBlock;
    use crate::{
        dom::nodes::{ContainerNode, DomNode},
        InlineFormatType, ListType,
    };
    use matrix_mentions::Mention;
    use std::fmt;

    use wasm_bindgen::JsCast;
    use web_sys::{
        Document, DomParser, Element, HtmlElement, NodeList, SupportedType,
    };

    pub(super) struct HtmlParser {
        current_path: Vec<DomNodeKind>,
    }
    impl HtmlParser {
        pub(super) fn default() -> Self {
            Self {
                current_path: Vec::new(),
            }
        }

        pub(super) fn parse<S>(
            &mut self,
            html: &str,
        ) -> Result<Dom<S>, HtmlParseError>
        where
            S: UnicodeString,
        {
            self.parse_internal(html, HtmlSource::Matrix)
        }

        pub(super) fn parse_from_source<S>(
            &mut self,
            html: &str,
            html_source: HtmlSource,
        ) -> Result<Dom<S>, HtmlParseError>
        where
            S: UnicodeString,
        {
            self.parse_internal(html, html_source)
        }

        fn parse_internal<S>(
            &mut self,
            html: &str,
            html_source: HtmlSource,
        ) -> Result<Dom<S>, HtmlParseError>
        where
            S: UnicodeString,
        {
            let parser: DomParser = DomParser::new().map_err(|_| {
                to_dom_creation_error(
                    "Failed to create the `DOMParser` from JavaScript",
                )
            })?;

            let document = parser
                .parse_from_string(html, SupportedType::TextHtml)
                .map_err(|_| {
                    to_dom_creation_error(
                        "Failed to convert the Web `Document` to internal `Dom`",
                    )
                })?;

            self.webdom_to_dom(document, html_source)
                .map_err(to_dom_creation_error)
                .map(post_process_blocks)
                .map(post_process_for_block_and_inline_siblings)
                .map(post_process_for_adjacent_text)
        }

        fn webdom_to_dom<S>(
            &mut self,
            webdoc: Document,
            html_source: HtmlSource,
        ) -> Result<Dom<S>, Error>
        where
            S: UnicodeString,
        {
            let body = webdoc.body().ok_or_else(|| Error::NoBody)?;
            self.convert(body.child_nodes(), DomNodeKind::Generic, html_source)
        }

        fn convert<S>(
            &mut self,
            nodes: NodeList,
            parent_kind: DomNodeKind,
            html_source: HtmlSource,
        ) -> Result<Dom<S>, Error>
        where
            S: UnicodeString,
        {
            let number_of_nodes = nodes.length() as usize;
            let mut dom = Dom::new(Vec::with_capacity(number_of_nodes));
            let dom_document = dom.document_mut();

            self.convert_container(
                nodes,
                dom_document,
                parent_kind,
                html_source,
            )?;

            Ok(dom)
        }

        fn convert_container<S>(
            &mut self,
            nodes: NodeList,
            dom: &mut ContainerNode<S>,
            parent_kind: DomNodeKind,
            html_source: HtmlSource,
        ) -> Result<(), Error>
        where
            S: UnicodeString,
        {
            let number_of_nodes = nodes.length() as usize;

            for nth in 0..number_of_nodes {
                let node = nodes.get(nth as _).unwrap();
                let node_name = node.node_name();
                let tag = node_name.as_str();

                let mut invalid_node_error: Option<Error> = None;
                let mut skip_children: bool = false;

                // Check if we're inside a list and this node is not a list item
                if parent_kind == DomNodeKind::List
                    && tag != "LI"
                    && html_source != HtmlSource::GoogleDoc
                {
                    // If we are inside a list, we can only have list items.
                    invalid_node_error = Some(Error::InvalidListItemNode);
                    skip_children = true;
                }

                if invalid_node_error.is_none() {
                    match tag {
                        "BR" => {
                            dom.append_child(DomNode::new_line_break());
                        }

                        "#text" => match node.node_value() {
                            Some(value) => {
                                let is_inside_code_block =
                                    self.current_path.contains(&CodeBlock);
                                let is_only_child_in_parent =
                                    number_of_nodes == 1;
                                convert_text(
                                    value.as_str(),
                                    dom,
                                    is_inside_code_block,
                                    is_only_child_in_parent,
                                );
                            }
                            _ => {}
                        },

                        "A" => {
                            self.current_path.push(DomNodeKind::Link);

                            let mut attributes = vec![];
                            // we only need to pass in a style attribute from web to allow CSS variable insertion
                            let valid_attributes = ["style"];

                            for attr in valid_attributes.into_iter() {
                                if node
                                    .unchecked_ref::<Element>()
                                    .has_attribute(attr)
                                {
                                    attributes.push((
                                        attr.into(),
                                        node.unchecked_ref::<Element>()
                                            .get_attribute(attr)
                                            .unwrap_or_default()
                                            .into(),
                                    ))
                                }
                            }

                            let url = node
                                .unchecked_ref::<Element>()
                                .get_attribute("href")
                                .unwrap_or_default();

                            let is_mention =
                                Mention::is_valid_uri(&url.to_string());
                            let text = node.child_nodes().get(0);
                            let has_text = match text.clone() {
                                Some(node) => {
                                    node.node_type() == web_sys::Node::TEXT_NODE
                                }
                                None => false,
                            };
                            if has_text && is_mention {
                                dom.append_child(
                                    DomNode::Mention(
                                        DomNode::new_mention(
                                            url.into(),
                                            text.unwrap()
                                                .node_value()
                                                .unwrap_or_default()
                                                .into(),
                                            attributes,
                                        )
                                        .unwrap(),
                                    ), // we unwrap because we have already confirmed the uri is valid
                                );
                            } else {
                                let children = self
                                    .convert(
                                        node.child_nodes(),
                                        DomNodeKind::Link,
                                        html_source,
                                    )?
                                    .take_children();
                                dom.append_child(DomNode::new_link(
                                    url.into(),
                                    children,
                                    attributes,
                                ));
                            }

                            self.current_path.pop();
                        }
                        "UL" | "OL" => {
                            let custom_start = node
                                .unchecked_ref::<Element>()
                                .get_attribute("start");

                            let attributes: Option<Vec<(S, S)>> =
                                if tag == "OL" && custom_start.is_some() {
                                    Some(vec![(
                                        "start".into(),
                                        custom_start.unwrap().into(),
                                    )])
                                } else {
                                    None
                                };

                            let list_type = if tag == "OL" {
                                ListType::Ordered
                            } else {
                                ListType::Unordered
                            };

                            if parent_kind == DomNodeKind::List {
                                // Google docs adds nested lists as children of the list node, this breaks our invariants.
                                // For the google docs case, we can add the nested list to the last list item instead.
                                if html_source != HtmlSource::GoogleDoc
                                    || dom.last_child_mut().is_none()
                                    || dom
                                        .last_child_mut()
                                        .unwrap()
                                        .is_list_item()
                                        == false
                                {
                                    // If source is not Google Docs or the last child is not a list item, we return an error.
                                    invalid_node_error =
                                        Some(Error::InvalidListItemNode);
                                } else {
                                    self.current_path.push(DomNodeKind::List);
                                    let target = dom
                                        .last_child_mut()
                                        .unwrap()
                                        .as_container_mut()
                                        .unwrap();
                                    target.append_child(DomNode::Container(
                                        ContainerNode::new_list(
                                            list_type,
                                            self.convert(
                                                node.child_nodes(),
                                                DomNodeKind::List,
                                                html_source,
                                            )?
                                            .take_children(),
                                            attributes,
                                        ),
                                    ));
                                    self.current_path.pop();
                                }
                            } else {
                                self.current_path.push(DomNodeKind::List);
                                dom.append_child(DomNode::Container(
                                    ContainerNode::new_list(
                                        list_type,
                                        self.convert(
                                            node.child_nodes(),
                                            DomNodeKind::List,
                                            html_source,
                                        )?
                                        .take_children(),
                                        attributes,
                                    ),
                                ));
                                self.current_path.pop();
                            }
                        }

                        "LI" => {
                            if parent_kind != DomNodeKind::List {
                                invalid_node_error =
                                    Some(Error::ParentNotAList);
                            } else {
                                self.current_path.push(DomNodeKind::ListItem);
                                dom.append_child(DomNode::Container(
                                    ContainerNode::new_list_item(
                                        self.convert(
                                            node.child_nodes(),
                                            DomNodeKind::ListItem,
                                            html_source,
                                        )?
                                        .take_children(),
                                    ),
                                ));
                                self.current_path.pop();
                            }
                        }

                        "PRE" => {
                            self.current_path.push(DomNodeKind::CodeBlock);
                            let children = node.child_nodes();
                            let children = if children.length() == 1
                                && children.get(0).unwrap().node_name().as_str()
                                    == "CODE"
                            {
                                let code_node = children.get(0).unwrap();
                                code_node.child_nodes()
                            } else {
                                children
                            };
                            dom.append_child(DomNode::Container(
                                ContainerNode::new_code_block(
                                    self.convert(
                                        children,
                                        DomNodeKind::CodeBlock,
                                        html_source,
                                    )?
                                    .take_children(),
                                ),
                            ));
                            self.current_path.pop();
                        }

                        "BLOCKQUOTE" => {
                            self.current_path.push(DomNodeKind::Quote);
                            dom.append_child(DomNode::Container(
                                ContainerNode::new_quote(
                                    self.convert(
                                        node.child_nodes(),
                                        DomNodeKind::Quote,
                                        html_source,
                                    )?
                                    .take_children(),
                                ),
                            ));
                            self.current_path.pop();
                        }

                        "P" => {
                            self.current_path.push(DomNodeKind::Paragraph);
                            dom.append_child(DomNode::Container(
                                ContainerNode::new_paragraph(
                                    self.convert(
                                        node.child_nodes(),
                                        DomNodeKind::Paragraph,
                                        html_source,
                                    )?
                                    .take_children(),
                                ),
                            ));
                            self.current_path.pop();
                        }
                        node_name => {
                            let formatting_kind = match node_name {
                                "STRONG" | "B" => Some(InlineFormatType::Bold),
                                "EM" | "I" => Some(InlineFormatType::Italic),
                                "DEL" => Some(InlineFormatType::StrikeThrough),
                                "U" => Some(InlineFormatType::Underline),
                                "CODE" => Some(InlineFormatType::InlineCode),
                                "SPAN" => {
                                    if html_source == HtmlSource::Matrix {
                                        invalid_node_error =
                                            Some(Error::UnknownNode(
                                                node_name.to_owned(),
                                            ));
                                        None
                                    } else {
                                        // For external sources, we check for common formatting styles for spans
                                        // and convert them to appropriate formatting nodes.
                                        let style = node
                                            .unchecked_ref::<HtmlElement>()
                                            .style();
                                        if style
                                            .get_property_value("font-weight")
                                            .unwrap_or_default()
                                            == "bold"
                                            || style
                                                .get_property_value(
                                                    "font-weight",
                                                )
                                                .unwrap_or_default()
                                                == "700"
                                        {
                                            Some(InlineFormatType::Bold)
                                        } else if style
                                            .get_property_value("font-style")
                                            .unwrap_or_default()
                                            == "italic"
                                        {
                                            Some(InlineFormatType::Italic)
                                        } else if style
                                            .get_property_value(
                                                "text-decoration",
                                            )
                                            .unwrap_or_default()
                                            == "underline"
                                        {
                                            Some(InlineFormatType::Underline)
                                        } else if style
                                            .get_property_value(
                                                "text-decoration",
                                            )
                                            .unwrap_or_default()
                                            == "line-through"
                                        {
                                            Some(
                                                InlineFormatType::StrikeThrough,
                                            )
                                        } else {
                                            invalid_node_error =
                                                Some(Error::UnknownNode(
                                                    node_name.to_owned(),
                                                ));
                                            None
                                        }
                                    }
                                }
                                _ => {
                                    invalid_node_error =
                                        Some(Error::UnknownNode(
                                            node_name.to_owned(),
                                        ));
                                    None
                                }
                            };

                            if let Some(formatting_kind) = formatting_kind {
                                // Special case for code inside code blocks - skip the inline code formatting
                                if formatting_kind
                                    == InlineFormatType::InlineCode
                                    && self.current_path.contains(&CodeBlock)
                                {
                                    let children_nodes = self
                                        .convert(
                                            node.child_nodes(),
                                            parent_kind.clone(),
                                            html_source,
                                        )?
                                        .take_children();
                                    if !children_nodes.is_empty() {
                                        dom.append_children(children_nodes);
                                    }
                                } else {
                                    self.current_path.push(
                                        DomNodeKind::Formatting(
                                            formatting_kind.clone(),
                                        ),
                                    );
                                    let children_nodes = self
                                        .convert(
                                            node.child_nodes(),
                                            DomNodeKind::Formatting(
                                                formatting_kind.clone(),
                                            ),
                                            html_source,
                                        )?
                                        .take_children();

                                    dom.append_child(DomNode::Container(
                                        ContainerNode::new_formatting(
                                            formatting_kind.clone(),
                                            children_nodes,
                                        ),
                                    ));
                                    self.current_path.pop();
                                }
                            }
                        }
                    }
                }

                // Handle invalid node errors
                if let Some(err) = invalid_node_error {
                    if html_source == HtmlSource::Matrix {
                        return Err(err);
                    } else if !skip_children {
                        // If the source is not Matrix and we haven't explicitly flagged to skip the children continue to parse them.
                        let children_nodes = self
                            .convert(
                                node.child_nodes(),
                                parent_kind.clone(),
                                html_source,
                            )?
                            .take_children();
                        if !children_nodes.is_empty() {
                            dom.append_children(children_nodes);
                        }
                    }
                }
            }

            Ok(())
        }
    }

    fn to_dom_creation_error<E>(error: E) -> HtmlParseError
    where
        E: ToString,
    {
        HtmlParseError {
            parse_errors: vec![error.to_string()],
        }
    }

    enum Error {
        NoBody,
        UnknownNode(String),
        InvalidListItemNode,
        ParentNotAList,
    }

    impl fmt::Display for Error {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::NoBody => {
                    write!(
                        formatter,
                        "The `Document` does not have a `<body>` element"
                    )
                }

                Self::UnknownNode(node_name) => {
                    write!(formatter, "Node `{node_name}` is not supported")
                }
                Self::InvalidListItemNode => {
                    write!(
                        formatter,
                        "Invalid list item node: a list must only contain list items"
                    )
                }
                Self::ParentNotAList => {
                    write!(formatter, "Parent node is not a list")
                }
            }
        }
    }

    #[cfg(all(test, target_arch = "wasm32"))]
    mod tests {
        use super::*;
        use crate::{
            tests::testutils_composer_model::restore_whitespace, ToHtml,
            ToMarkdown, ToTree,
        };
        use indoc::indoc;
        use wasm_bindgen_test::*;
        use widestring::Utf16String;

        wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

        fn roundtrip(html: &str) {
            let parse = HtmlParser::default().parse::<Utf16String>(html);

            assert!(
                parse.is_ok(),
                "Failed to parse the following HTML fragment: `{html}`"
            );

            let dom = parse.unwrap();
            let html_again = restore_whitespace(&dom.to_html().to_string());

            assert_eq!(html, html_again);
        }

        #[wasm_bindgen_test]
        fn formatting() {
            roundtrip("foo <strong>bar</strong> baz");
            roundtrip("foo <em>bar</em> baz");
            roundtrip("foo <del>bar</del> baz");
            roundtrip("foo <u>bar</u> baz");
            roundtrip("foo <code>bar</code> baz");
        }

        #[wasm_bindgen_test]
        fn parse_insert_text_directly_into_a_list() {
            let html = r#"<ul><li>hello</li><b>list item</b></ul>"#;
            let dom: Dom<Utf16String> = HtmlParser::default()
                .parse_from_source(html, HtmlSource::UnknownExternal)
                .unwrap();
            assert_eq!(dom.to_html(), r#"<ul><li>hello</li></ul>"#);
        }

        #[wasm_bindgen_test]
        fn google_doc_rich_text() {
            let dom = HtmlParser::default()
                .parse_from_source::<Utf16String>(
                    GOOGLE_DOC_HTML_PASTEBOARD,
                    HtmlSource::GoogleDoc,
                )
                .unwrap();
            assert_eq!(dom.to_string(), "<ol><li><p><em>Italic</em></p></li><li><p><strong>Bold</strong></p></li><li><p>Unformatted</p></li><li><p><del>Strikethrough</del></p></li><li><p><u>Underlined</u></p></li><li><p><a style=\"text-decoration:none;\" href=\"http://matrix.org\"><u>Linked</u></a></p><ul><li><p>Nested</p></li></ul></li></ol>");
            assert_eq!(
                dom.to_markdown().unwrap().to_string(),
                indoc! {r#"
                1. *Italic*
                2. __Bold__
                3. Unformatted
                4. ~~Strikethrough~~
                5. <u>Underlined</u>
                6. [<u>Linked</u>](<http://matrix.org>)
                  * Nested"#
                }
            );
        }

        #[wasm_bindgen_test]
        fn ms_rich_text() {
            let dom = HtmlParser::default()
                .parse_from_source::<Utf16String>(
                    MS_DOC_HTML_PASTEBOARD,
                    HtmlSource::UnknownExternal,
                )
                .unwrap();
            assert_eq!(dom.to_string(), "<ol start=\"1\"><li><p><em>Italic</em></p></li><li><p><strong>Bold</strong></p></li><li><p>Unformatted</p></li><li><p><del>Strikethrough</del></p></li><li><p><u>Underlined</u></p></li><li><p><a style=\"-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; cursor: text; text-decoration: none; color: inherit;\" href=\"https://matrix.org/\"><u>Linked</u></a></p></li></ol><ul><li><p>Nested</p></li></ul>");
        }

        #[wasm_bindgen_test]
        fn unknown_tag_errors() {
            let html = r#"
            <span style="font-weight: bold;">Bold</span>
        "#;
            let result = HtmlParser::default().parse::<Utf16String>(html);
            assert_eq!(result.is_err(), true);
        }

        #[wasm_bindgen_test]
        fn br() {
            let html = "foo<br />bar";
            let dom = HtmlParser::default().parse::<Utf16String>(html).unwrap();
            assert_eq!(dom.to_string(), "<p>foo</p><p>bar</p>");
        }

        #[wasm_bindgen_test]
        fn a() {
            roundtrip(r#"foo <a href="url">bar</a> baz"#);
            roundtrip(r#"foo <a href="">bar</a> baz"#);
        }

        #[wasm_bindgen_test]
        fn mention_with_attributes() {
            roundtrip(
                r#"<a style="something" href="https://matrix.to/@test:example.org">test</a>"#,
            );
        }

        #[wasm_bindgen_test]
        fn mention_with_bad_attribute() {
            let html = r#"<a invalidattribute="true" href="https://matrix.to/#/@test:example.org">test</a>"#;
            let dom = HtmlParser::default().parse::<Utf16String>(html).unwrap();
            assert_eq!(
                dom.to_string(),
                r#"<a data-mention-type="user" href="https://matrix.to/#/@test:example.org" contenteditable="false">test</a>"#
            );
        }

        #[wasm_bindgen_test]
        fn ul() {
            roundtrip(
                "<p>foo </p><ul><li>item1</li><li>item2</li></ul><p> bar</p>",
            );
        }

        #[wasm_bindgen_test]
        fn ol() {
            roundtrip(
                "<p>foo </p><ol><li>item1</li><li>item2</li></ol><p> bar</p>",
            );
        }

        #[wasm_bindgen_test]
        fn pre() {
            roundtrip(
                "<p>foo </p><pre><code>~Some code</code></pre><p> bar</p>",
            );
        }

        #[wasm_bindgen_test]
        fn paragraph() {
            roundtrip("<p>foo</p><p>Text</p><p>bar</p>");
        }

        #[wasm_bindgen_test]
        fn pre_removes_internal_code() {
            let html = "<p>foo</p><pre><code>Some code</code></pre><p>bar</p>";
            let dom = HtmlParser::default().parse::<Utf16String>(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
        
                ├>p
                │ └>"foo"
                ├>codeblock
                │ └>p
                │   └>"Some code"
                └>p
                  └>"bar"
                "#}
            );
        }

        #[wasm_bindgen_test]
        fn blockquote() {
            roundtrip(
                "<p>foo </p><blockquote>~Some code</blockquote><p> bar</p>",
            );
        }

        #[wasm_bindgen_test]
        fn nbsp_chars_are_removed() {
            let html = "\
                <p>\u{A0}</p>\
                <pre><code>\u{A0}\n\u{A0}</code></pre>\
                <p>\u{A0}</p>";
            let dom = HtmlParser::default().parse::<Utf16String>(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                ├>p
                ├>codeblock
                │ ├>p
                │ └>p
                └>p
                "#}
            );
        }

        #[wasm_bindgen_test]
        fn nbsp_text_is_removed() {
            let html = "\
                <p>&nbsp;</p>\
                <pre><code>&nbsp;\n&nbsp;</code></pre>\
                <p>&nbsp;</p>";
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse::<Utf16String>(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                ├>p
                ├>codeblock
                │ ├>p
                │ └>p
                └>p
                "#}
            );
        }
    }
}
