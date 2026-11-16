// Copyright 2026 New Vector Ltd.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

//! Random-operation fuzzer for the composer model: drives the public API
//! with random edit sequences and reports every distinct panic site, each
//! with a minimised reproducer printed as Rust code.
//!
//! ```sh
//! cargo test -p wysiwyg --test fuzz -- --ignored --nocapture
//! FUZZ_SEED=7 FUZZ_CASES=200000 FUZZ_OPS=60 cargo test ... (same as above)
//! ```
//!
//! `FUZZ_SKIP=ReplaceHtml,Mention,add_line_break` leaves out the operations
//! whose name (an `Op` variant or a model method) contains one of those
//! strings, e.g. to match what one client calls or to get past known bugs.
//!
//! Actions the model reports as disabled are not called, as clients grey
//! them out; `FUZZ_DISABLED=1` calls them anyway.
//!
//! Add `--features assert-invariants` to also treat DOM invariant
//! violations as failures (much noisier: these are not production panics).
//!
//! ponytail: not coverage-guided, and a hang or stack overflow kills the
//! run rather than being reported; port the `apply` driver to cargo-fuzz
//! if this stops finding things.

#![cfg(not(target_arch = "wasm32"))]
#![allow(deprecated)] // add_line_break is deprecated but still callable

use std::collections::BTreeMap;
use std::panic::{self, AssertUnwindSafe};
use std::sync::Mutex;

use widestring::Utf16String;
use wysiwyg::{
    ActionState, ComposerAction, ComposerModel, ComposerUpdate, DomNode,
    HtmlSource, Location, MenuAction, SuggestionPattern,
};

type Model = ComposerModel<Utf16String>;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() >> 33) as usize % n
    }
    fn pick<T: Copy>(&mut self, items: &[T]) -> T {
        items[self.below(items.len())]
    }
}

const TEXTS: &[&str] = &[
    "a",
    "abc",
    "foo bar",
    " ",
    "  ",
    "\n",
    "a\nb",
    "@al",
    "#room",
    "/cmd",
    ":)",
    "🙂",
    "👩‍👩‍👧",
    "é",
    "\u{200b}",
    "\u{a0}",
    "1. ",
    "* ",
    "> ",
    "`",
    "```",
    "https://matrix.org ",
    "www.a.b",
    "x@y.z ",
    "@alice:matrix.org",
    "<b>",
    "&",
    "",
];

const HTMLS: &[&str] = &[
    "",
    "plain",
    "a<br />b",
    "<p>a</p><p>b</p>",
    "<p></p>",
    "<p>&nbsp;</p>",
    "<b>x <i>y</i></b> <code>z</code>",
    "<del>s</del><u>u</u>",
    "text <strong>bold</strong> 🙂 <em>em</em>",
    "<a href=\"https://matrix.org\">link</a>",
    "<a href=\"https://matrix.to/#/@alice:matrix.org\">Alice</a>",
    "<a href=\"https://matrix.to/#/#room:matrix.org\">room</a> @room",
    "<ul><li>a</li><li>b</li></ul>",
    "<ul><li></li></ul>",
    "<ul><li>a</li><li>b<ul><li>c</li><li>d</li></ul></li><li>e</li></ul>",
    "<ol><li>a<ul><li>b<ol><li>c</li></ol></li></ul></li></ol>",
    "<ol><li><p>a</p><p>b</p></li><li>c</li></ol>",
    "<ol><li>a</li></ol><p>&nbsp;</p><ul><li>b</li></ul>",
    "<blockquote><p>q</p></blockquote>",
    "<blockquote><p>q</p><ul><li>x</li></ul><p>r</p></blockquote>",
    "<blockquote><pre><code>x</code></pre></blockquote>",
    "<pre><code>c1\nc2</code></pre>",
    "<pre><code><b>c</b>\n\n</code></pre><p>after</p>",
    "<ul><li><pre><code>c</code></pre></li><li><blockquote><p>q</p></blockquote></li></ul>",
    "<p><b>a</b></p><ul><li><i>b</i><br />c</li></ul>",
    "<h1>h</h1><hr><table><tr><td>t</td></tr></table><img src=\"x\"><span>s</span>",
    "<div>d<div>e</div></div>",
    "<meta charset=\"utf-8\"><b style=\"font-weight:normal\" id=\"docs-internal-guid-1\"><span style=\"font-weight:700\">g</span></b>",
    "<li>orphan</li>",
    "<b><p>block in inline</p></b>",
    "<a href=\"x\"><a href=\"y\">nested</a></a>",
];

const MARKDOWNS: &[&str] = &[
    "",
    "# h",
    "a\n\nb",
    "a  \nb",
    "* a\n* b",
    "1. a\n   * b\n2. c",
    "> q",
    "> * a\n>\n> b",
    "```\ncode\n```",
    "**b** _i_ `c` ~~s~~",
    "[l](https://x.y)",
    "a\n\n* b\n\n> c\n\n```\nd\n```\n\ne",
    "<b>raw</b> html",
];

const URLS: &[&str] = &[
    "https://matrix.org",
    "https://matrix.to/#/@alice:matrix.org",
    "https://matrix.to/#/#room:matrix.org",
    "mailto:x@y.z",
    "not a url",
    "",
];

#[derive(Clone, Debug)]
enum Op {
    Simple(usize),
    Text(&'static str),
    Select(usize, usize),
    ReplaceTextIn(&'static str, usize, usize),
    DeleteIn(usize, usize),
    SetLink(&'static str),
    SetLinkWithText(&'static str, &'static str),
    Mention(&'static str, &'static str),
    AtRoom,
    MentionAtSuggestion(&'static str, &'static str),
    AtRoomAtSuggestion,
    ReplaceSuggestion(&'static str, bool),
    ReplaceHtml(String, usize),
    SetHtml(String),
    SetMarkdown(&'static str),
}

/// The argument-less model methods, as (source code, call) pairs.
macro_rules! simple_ops {
    ($($name:ident),* $(,)?) => {
        const SIMPLE: &[(&str, fn(&mut Model) -> ComposerUpdate<Utf16String>)] =
            &[$((concat!("m.", stringify!($name), "();"), |m| m.$name())),*];
    };
}
simple_ops!(
    enter,
    enter,
    backspace,
    backspace,
    delete,
    backspace_word,
    delete_word,
    bold,
    italic,
    underline,
    strike_through,
    inline_code,
    code_block,
    quote,
    ordered_list,
    unordered_list,
    indent,
    unindent,
    undo,
    redo,
    remove_links,
    add_line_break,
    clear,
);

const SOURCES: &[(&str, HtmlSource)] = &[
    ("HtmlSource::Matrix", HtmlSource::Matrix),
    ("HtmlSource::GoogleDoc", HtmlSource::GoogleDoc),
    ("HtmlSource::UnknownExternal", HtmlSource::UnknownExternal),
];

fn u(s: &str) -> Utf16String {
    Utf16String::from_str(s)
}

fn html(rng: &mut Rng) -> String {
    (0..1 + rng.below(3)).map(|_| rng.pick(HTMLS)).collect()
}

fn random_op(rng: &mut Rng, len: usize) -> Op {
    // Mostly in range, occasionally past the end (the model claims to clamp).
    let pos = |rng: &mut Rng| {
        let slack = (rng.below(8) == 0) as usize * 3;
        rng.below(len + 1 + slack)
    };
    match rng.below(100) {
        0..=34 => Op::Simple(rng.below(SIMPLE.len())),
        35..=54 => Op::Text(rng.pick(TEXTS)),
        55..=59 => {
            let p = pos(rng);
            Op::Select(p, p)
        }
        60..=74 => Op::Select(pos(rng), pos(rng)),
        75..=76 => Op::Select(0, len),
        77..=78 => Op::ReplaceTextIn(rng.pick(TEXTS), pos(rng), pos(rng)),
        79 => Op::DeleteIn(pos(rng), pos(rng)),
        80..=82 => Op::SetLink(rng.pick(URLS)),
        83..=84 => Op::SetLinkWithText(rng.pick(URLS), rng.pick(TEXTS)),
        85..=86 => Op::Mention(rng.pick(URLS), rng.pick(TEXTS)),
        87 => Op::AtRoom,
        88..=89 => Op::MentionAtSuggestion(rng.pick(URLS), rng.pick(TEXTS)),
        90 => Op::AtRoomAtSuggestion,
        91..=92 => Op::ReplaceSuggestion(rng.pick(TEXTS), rng.below(2) == 0),
        93..=96 => Op::ReplaceHtml(html(rng), rng.below(SOURCES.len())),
        97..=98 => Op::SetHtml(html(rng)),
        _ => Op::SetMarkdown(rng.pick(MARKDOWNS)),
    }
}

/// Log the source of a model call, then make it (in that order, so that the
/// log names the call that panicked).
macro_rules! call {
    ($log:expr, $call:expr, $($fmt:tt)*) => {{
        $log.push(format!($($fmt)*));
        $call
    }};
}

/// Apply `op`, appending its Rust source to `log` (nothing if the op did not
/// apply). `sugg` is the suggestion the model last reported, which is what a
/// client would pass back to the at-suggestion methods.
fn apply(
    m: &mut Model,
    op: &Op,
    sugg: &mut Option<SuggestionPattern>,
    log: &mut Vec<String>,
) {
    let sugg_code = |s: &SuggestionPattern| {
        format!(
            "SuggestionPattern {{ key: PatternKey::{:?}, text: {:?}.into(), start: {}, end: {} }}",
            s.key, s.text, s.start, s.end
        )
    };
    let update = match op {
        Op::Simple(i) => call!(log, SIMPLE[*i].1(m), "{}", SIMPLE[*i].0),
        Op::Text(t) => {
            call!(log, m.replace_text(u(t)), "m.replace_text(u({t:?}));")
        }
        Op::Select(s, e) => call!(
            log,
            m.select(Location::from(*s), Location::from(*e)),
            "m.select(Location::from({s}), Location::from({e}));"
        ),
        Op::ReplaceTextIn(t, s, e) => {
            let (s, e) = (*s.min(e), *s.max(e));
            call!(
                log,
                m.replace_text_in(u(t), s, e),
                "m.replace_text_in(u({t:?}), {s}, {e});"
            )
        }
        Op::DeleteIn(s, e) => {
            let (s, e) = (*s.min(e), *s.max(e));
            call!(log, m.delete_in(s, e), "m.delete_in({s}, {e});")
        }
        Op::SetLink(url) => {
            call!(
                log,
                m.set_link(u(url), vec![]),
                "m.set_link(u({url:?}), vec![]);"
            )
        }
        Op::SetLinkWithText(url, t) => call!(
            log,
            m.set_link_with_text(u(url), u(t), vec![]),
            "m.set_link_with_text(u({url:?}), u({t:?}), vec![]);"
        ),
        Op::Mention(url, t) => call!(
            log,
            m.insert_mention(u(url), u(t), vec![]),
            "m.insert_mention(u({url:?}), u({t:?}), vec![]);"
        ),
        Op::AtRoom => {
            call!(
                log,
                m.insert_at_room_mention(vec![]),
                "m.insert_at_room_mention(vec![]);"
            )
        }
        Op::MentionAtSuggestion(..)
        | Op::AtRoomAtSuggestion
        | Op::ReplaceSuggestion(..) => {
            let Some(s) = sugg.clone() else {
                return;
            };
            let sc = sugg_code(&s);
            match op {
                Op::MentionAtSuggestion(url, t) => call!(
                    log,
                    m.insert_mention_at_suggestion(u(url), u(t), s, vec![]),
                    "m.insert_mention_at_suggestion(u({url:?}), u({t:?}), {sc}, vec![]);"
                ),
                Op::AtRoomAtSuggestion => call!(
                    log,
                    m.insert_at_room_mention_at_suggestion(s, vec![]),
                    "m.insert_at_room_mention_at_suggestion({sc}, vec![]);"
                ),
                Op::ReplaceSuggestion(t, space) => call!(
                    log,
                    m.replace_text_suggestion(u(t), s, *space),
                    "m.replace_text_suggestion(u({t:?}), {sc}, {space});"
                ),
                _ => unreachable!(),
            }
        }
        Op::ReplaceHtml(h, src) => call!(
            log,
            m.replace_html(u(h), SOURCES[*src].1),
            "m.replace_html(u({h:?}), {});",
            SOURCES[*src].0
        ),
        Op::SetHtml(h) => call!(
            log,
            m.set_content_from_html(&u(h))
                .unwrap_or_else(|_| ComposerUpdate::keep()),
            "let _ = m.set_content_from_html(&u({h:?}));"
        ),
        Op::SetMarkdown(md) => call!(
            log,
            m.set_content_from_markdown(&u(md))
                .unwrap_or_else(|_| ComposerUpdate::keep()),
            "let _ = m.set_content_from_markdown(&u({md:?}));"
        ),
    };
    match update.menu_action {
        MenuAction::Keep => {}
        MenuAction::None => *sugg = None,
        MenuAction::Suggestion(s) => *sugg = Some(s),
    }
}

/// Everything a client reads back after an edit, plus re-parsing our own
/// output (which is what happens when a draft or an edit is loaded).
const CHECK_CODE: &str = "\
/* panics in one of: */
m.get_content_as_html(); m.get_content_as_message_html(); m.get_content_as_markdown();
m.get_content_as_message_markdown(); m.get_content_as_plain_text(); m.to_tree();
m.get_link_action(); m.get_mentions_state();
let _ = ComposerModel::new().set_content_from_html(&m.get_content_as_html());
let _ = ComposerModel::new().set_content_from_html(&m.get_content_as_message_html());";

fn check(m: &Model, log: &mut Vec<String>) {
    log.push(CHECK_CODE.to_owned());
    let html = m.get_content_as_html();
    let message_html = m.get_content_as_message_html();
    m.get_content_as_markdown();
    m.get_content_as_message_markdown();
    m.get_content_as_plain_text();
    m.to_tree();
    m.get_link_action();
    m.get_mentions_state();
    let _ = Model::new().set_content_from_html(&html);
    let _ = Model::new().set_content_from_html(&message_html);
    log.pop();
}

static LAST_PANIC: Mutex<(String, String)> =
    Mutex::new((String::new(), String::new()));

/// The (site, message) of the panic raised by `f`, if any.
fn panic_of(f: impl FnOnce()) -> Option<(String, String)> {
    panic::catch_unwind(AssertUnwindSafe(f))
        .err()
        .map(|_| LAST_PANIC.lock().unwrap().clone())
}

/// Run `ops` on a fresh model. On panic returns (panic site, message,
/// reproducer source).
fn run(ops: &[Op]) -> Option<(String, String, String)> {
    let mut log = Vec::new();
    let panicked = panic_of(|| {
        let mut m = Model::new();
        let mut sugg = None;
        for op in ops {
            apply(&mut m, op, &mut sugg, &mut log);
            check(&m, &mut log);
        }
    });
    panicked.map(|(site, msg)| {
        let last = log.last_mut().unwrap();
        if last != CHECK_CODE {
            last.insert_str(0, "/* panics: */ ");
        }
        (site, msg, log.join("\n"))
    })
}

/// Whether text offset `p` falls between the two halves of a surrogate
/// pair. No client can put the cursor there, so the fuzzer does not either.
fn splits_surrogate_pair(m: &Model, p: usize) -> bool {
    let dom = &m.state.dom;
    // Looking up a range inside a surrogate pair can itself panic (as it can
    // if an earlier op corrupted the DOM, which is reported when an op trips
    // over it, not here).
    panic::catch_unwind(AssertUnwindSafe(|| {
        dom.find_range(p, p).locations.iter().any(|loc| {
            let DomNode::Text(t) = dom.lookup_node(&loc.node_handle) else {
                return false;
            };
            let unit = t.data().as_slice().get(loc.start_offset);
            unit.is_some_and(|u| (0xDC00..0xE000).contains(u))
        })
    }))
    .unwrap_or(true)
}

impl Op {
    /// The toolbar action this op corresponds to, if any.
    fn action(&self) -> Option<ComposerAction> {
        use ComposerAction::*;
        let Op::Simple(i) = self else {
            let is_link =
                matches!(self, Op::SetLink(_) | Op::SetLinkWithText(..));
            return is_link.then_some(Link);
        };
        Some(match SIMPLE[*i].0 {
            "m.bold();" => Bold,
            "m.italic();" => Italic,
            "m.underline();" => Underline,
            "m.strike_through();" => StrikeThrough,
            "m.inline_code();" => InlineCode,
            "m.code_block();" => CodeBlock,
            "m.quote();" => Quote,
            "m.ordered_list();" => OrderedList,
            "m.unordered_list();" => UnorderedList,
            "m.indent();" => Indent,
            "m.unindent();" => Unindent,
            "m.undo();" => Undo,
            "m.redo();" => Redo,
            _ => return None,
        })
    }

    /// Move any offsets in this op off the middle of a surrogate pair.
    fn snap(&mut self, m: &Model) {
        if let Op::Select(s, e)
        | Op::ReplaceTextIn(_, s, e)
        | Op::DeleteIn(s, e) = self
        {
            for p in [s, e] {
                *p = p.saturating_sub(splits_surrogate_pair(m, *p) as usize);
            }
        }
    }
}

/// Greedily drop ops while the panic site stays the same.
fn minimise(mut ops: Vec<Op>, site: &str) -> Vec<Op> {
    let mut i = 0;
    while i < ops.len() {
        let mut fewer = ops.clone();
        fewer.remove(i);
        if run(&fewer).is_some_and(|(s, ..)| s == site) {
            ops = fewer;
            i = 0;
        } else {
            i += 1;
        }
    }
    ops
}

fn env(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[test]
#[ignore = "long-running fuzzer; run explicitly with --ignored"]
fn fuzz() {
    let seed = env("FUZZ_SEED", 1);
    let cases = env("FUZZ_CASES", 20_000);
    let max_ops = env("FUZZ_OPS", 40) as usize;
    let call_disabled = env("FUZZ_DISABLED", 0) != 0;
    let skip = std::env::var("FUZZ_SKIP").unwrap_or_default();
    let skip: Vec<&str> = skip.split(',').filter(|s| !s.is_empty()).collect();

    panic::set_hook(Box::new(|info| {
        let site = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()));
        let payload = info.payload();
        let msg = payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()));
        *LAST_PANIC.lock().unwrap() =
            (site.unwrap_or_default(), msg.unwrap_or_default());
    }));

    // site -> (hits, message, shortest reproducer)
    let mut found: BTreeMap<String, (u64, String, String)> = BTreeMap::new();
    for case in 0..cases {
        let mut rng = Rng((seed + case).wrapping_mul(0x9E3779B97F4A7C15) | 1);
        // Ops are generated against a live model so that selections land
        // inside the current text.
        let mut m = Model::new();
        let mut sugg = None;
        let mut ops = Vec::new();
        for i in 0..1 + rng.below(max_ops) {
            let mut op = if i == 0 && rng.below(2) == 0 {
                Op::SetHtml(html(&mut rng))
            } else {
                random_op(&mut rng, m.state.dom.text_len())
            };
            op.snap(&m);
            let name = match &op {
                Op::Simple(i) => SIMPLE[*i].0.to_owned(),
                other => format!("{other:?}"),
            };
            if skip.iter().any(|s| name.contains(s)) {
                continue;
            }
            // Clients grey out disabled actions, so states only reachable
            // through them are not interesting by default.
            let state = op.action().and_then(|a| m.action_states().get(&a));
            if !call_disabled && state == Some(&ActionState::Disabled) {
                continue;
            }
            // A panicking op is recorded and then rolled back, so that it
            // does not hide whatever the rest of the case would have found.
            let before = (m.clone(), sugg.clone());
            ops.push(op.clone());
            let Some((site, _)) = panic_of(|| {
                let mut log = Vec::new();
                apply(&mut m, &op, &mut sugg, &mut log);
                check(&m, &mut log);
            }) else {
                continue;
            };
            if let Some(entry) = found.get_mut(&site) {
                entry.0 += 1;
            } else if run(&ops).is_some_and(|(s, ..)| s == site) {
                let (_, msg, code) =
                    run(&minimise(ops.clone(), &site)).unwrap();
                // Printed as found: a long run is worth reading before it ends.
                eprintln!(
                    "\nnew panic site {site} (case {case}, seed {seed})\n    {}\n{code}",
                    msg.chars().take(300).collect::<String>()
                );
                found.insert(site, (1, msg, code));
            } else {
                // ponytail: happens when the ops before this one only reach
                // this state via a rolled-back panic's side effects; ignored.
                eprintln!("case {case}: panic at {site} did not reproduce from scratch");
            }
            ops.pop();
            (m, sugg) = before;
        }
    }

    let _ = panic::take_hook();
    for (site, (hits, msg, code)) in &found {
        let msg: String = msg.chars().take(300).collect();
        println!("\n=== {site} ({hits} hits)\n    {msg}\n{code}");
    }
    assert!(
        found.is_empty(),
        "{} distinct panic sites in {cases} cases",
        found.len()
    );
}
