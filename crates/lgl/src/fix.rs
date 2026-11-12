use std::ops::Range;

use serde::Serialize;
use typst::World;
use typst::syntax::ast::{self, AstNode};
use typst::syntax::{LinkedNode, Source, Span, SyntaxKind};

use crate::captions::DOUBLED_CAPTION_NUMBER;
use crate::diag::{Diagnostic, SourceLoc};
use crate::lint::{DISABLED_PROTECTION, NO_REFLOW, STRING_ESCAPE};
use crate::pagecheck::{COLUMN_IMBALANCE, EMPTY_COLUMN};
use crate::quality::{ASPECT, MIN_SIZE};
use crate::rules::{CONTAINER_OVERFLOW, MARGIN_INTRUSION, PAGE_OVERFLOW};
use crate::spacing::SPACING;
use crate::whitespace::PAGE_UNDERFILL;
use crate::world::LabWorld;

/// A machine-applicable change to the source.
#[derive(Debug, Clone, Serialize)]
pub struct Fix {
    pub description: String,
    pub edits: Vec<Edit>,
}

/// Replaces a range of a source file.
#[derive(Debug, Clone, Serialize)]
pub struct Edit {
    pub loc: SourceLoc,
    pub replacement: String,
    #[serde(skip)]
    pub range: Range<usize>,
}

/// Attaches fixes to the diagnostics for which a safe, mechanical change is
/// known.
pub fn suggest(world: &LabWorld, diagnostics: &mut [Diagnostic]) {
    for diagnostic in diagnostics {
        let Some(anchor) = diagnostic.anchor.filter(|s| !s.is_detached()) else {
            continue;
        };
        let Some(source) = anchor.id().and_then(|id| world.source(id).ok()) else {
            continue;
        };
        let side = diagnostic.side;
        diagnostic.fix = match diagnostic.rule.as_str() {
            CONTAINER_OVERFLOW | PAGE_OVERFLOW | MARGIN_INTRUSION => {
                overflow_fix(world, &source, anchor, side)
            }
            PAGE_UNDERFILL => underfill_fix(world, &source, anchor),
            SPACING => spacing_fix(world, &source, anchor),
            ASPECT => call_at(&source, anchor, &["image"]).and_then(|call| {
                Some(Fix {
                    description: "keep the image's aspect ratio".into(),
                    edits: vec![set_named(world, &source, &call, "fit", "\"contain\"")?],
                })
            }),
            COLUMN_IMBALANCE | EMPTY_COLUMN => enclosing_call(&source, anchor, "columns")
                .and_then(|call| {
                    Some(Fix {
                        description: "balance the columns".into(),
                        edits: vec![set_named(
                            world, &source, &call, "balanced", "true",
                        )?],
                    })
                }),
            NO_REFLOW => {
                call_at(&source, anchor, &["rotate", "scale", "skew"]).and_then(|call| {
                    Some(Fix {
                        description: "reserve space for the transformed content".into(),
                        edits: vec![set_named(world, &source, &call, "reflow", "true")?],
                    })
                })
            }
            DISABLED_PROTECTION => protection_fix(world, &source, anchor),
            STRING_ESCAPE => string_fix(world, &source, anchor),
            DOUBLED_CAPTION_NUMBER => caption_fix(world, &source, anchor),
            MIN_SIZE => enclosing_call(&source, anchor, "text").and_then(|call| {
                named_arg(&call, "size")?;
                Some(Fix {
                    description: "raise the text to the readable minimum".into(),
                    edits: vec![set_named(world, &source, &call, "size", "6pt")?],
                })
            }),
            _ => None,
        };
    }
}

fn overflow_fix(
    world: &LabWorld,
    source: &Source,
    anchor: Span,
    side: Option<&str>,
) -> Option<Fix> {
    let call = call_at(source, anchor, &["block", "box", "rect", "image"])?;
    match side? {
        "left" | "right" => {
            let named = named_arg(&call, "width")?;
            Some(Fix {
                description: "limit the width to the available space".into(),
                edits: vec![edit_range(world, source, named.value, "100%")?],
            })
        }
        _ => {
            let named = named_arg(&call, "height")?;
            Some(Fix {
                description: "let the height follow the content".into(),
                edits: vec![edit_range(world, source, named.value, "auto")?],
            })
        }
    }
}

fn underfill_fix(world: &LabWorld, source: &Source, anchor: Span) -> Option<Fix> {
    if let Some(call) = call_at(source, anchor, &["figure"]) {
        return Some(Fix {
            description: "let the figure float to where it fits".into(),
            edits: vec![set_named(world, source, &call, "placement", "auto")?],
        });
    }
    let call = call_at(source, anchor, &["block"])?;
    if named_arg(&call, "height").is_some() {
        return None;
    }
    Some(Fix {
        description: "allow the block to break across pages".into(),
        edits: vec![set_named(world, source, &call, "breakable", "true")?],
    })
}

/// Removes a set rule that only disables a protection, together with the show
/// rule it is the body of, if any. Rules that also set other properties are
/// left for the author.
fn protection_fix(world: &LabWorld, source: &Source, anchor: Span) -> Option<Fix> {
    let node = LinkedNode::new(source.root()).find(anchor)?;
    let rule = ast::SetRule::from_untyped(&node)?;
    if rule.args().items().count() != 1 || rule.condition().is_some() {
        return None;
    }
    let parent = node.parent()?;
    let target =
        if ast::ShowRule::from_untyped(parent).is_some() { parent } else { &node };
    let mut start = target.range().start;
    if let Some(hash) = target.prev_sibling_with_trivia()
        && hash.kind() == SyntaxKind::Hash
    {
        start = hash.range().start;
    }
    let mut end = target.range().end;
    if source.text()[end..].starts_with("\r\n") {
        end += 2;
    } else if source.text()[end..].starts_with('\n') {
        end += 1;
    }
    Some(Fix {
        description: "remove the rule so that Typst's protection applies".into(),
        edits: vec![edit_range(world, source, start..end, "")?],
    })
}

/// Removes the backslashes that a quoted string would print.
fn string_fix(world: &LabWorld, source: &Source, anchor: Span) -> Option<Fix> {
    let node = LinkedNode::new(source.root()).find(anchor)?;
    if node.kind() != SyntaxKind::Str {
        return None;
    }
    let fixed = crate::lint::without_markup_escapes(node.get().leaf_text());
    Some(Fix {
        description: "remove the backslashes the quoted string would print".into(),
        edits: vec![edit_range(world, source, node.range(), &fixed)?],
    })
}

/// Removes the hand-written label at the start of a figure's caption
/// ("*Table 1.*", "Figure 2:"), which repeats the label the figure prints.
/// Only a label written as plain or strong/emphasised text directly in the
/// caption argument is removed.
fn caption_fix(world: &LabWorld, source: &Source, anchor: Span) -> Option<Fix> {
    let figure = enclosing_call(source, anchor, "figure")?;
    let args = figure.children().find(|child| child.kind() == SyntaxKind::Args)?;
    let named = args.children().find(|child| {
        ast::Named::from_untyped(child).is_some_and(|n| n.name().get() == "caption")
    })?;
    let value = named.children().next_back()?;
    let leaf = first_text(&value)?;
    let text = source.text();
    let start = leaf.offset();
    let (length, _) = crate::captions::caption_label(&text[start..], true)?;
    // When the whole strong or emphasised text is the label, it goes with its
    // markers and the space after it; otherwise only the label goes.
    let mut range = start..start + length;
    if let Some(markup) = leaf.parent()
        && markup.kind() == SyntaxKind::Markup
        && markup.range().start == start
        && text[markup.range()].trim_end().len() <= length
        && let Some(marked) = markup.parent()
        && matches!(marked.kind(), SyntaxKind::Strong | SyntaxKind::Emph)
    {
        let end = marked.range().end;
        let after = text[end..].len() - text[end..].trim_start_matches([' ', '\t']).len();
        range = marked.range().start..end + after;
    }
    if text[range.clone()].contains(['#', '[', ']', '\\', '$', '\n']) {
        return None;
    }
    Some(Fix {
        description: "remove the hand-written caption label".into(),
        edits: vec![edit_range(world, source, range, "")?],
    })
}

/// The first text of the markup inside a node, in source order.
fn first_text<'a>(node: &LinkedNode<'a>) -> Option<LinkedNode<'a>> {
    if node.kind() == SyntaxKind::Text {
        return Some(node.clone());
    }
    node.children().find_map(|child| first_text(&child))
}

/// Removes a manual vertical spacing call right before the anchored content.
fn spacing_fix(world: &LabWorld, source: &Source, anchor: Span) -> Option<Fix> {
    let node = LinkedNode::new(source.root()).find(anchor)?;
    let previous = node.prev_sibling()?;
    let call = ast::FuncCall::from_untyped(&previous)?;
    let ast::Expr::Ident(callee) = call.callee() else { return None };
    if callee.get() != "v" {
        return None;
    }
    let mut start = previous.range().start;
    if let Some(hash) = previous.prev_sibling_with_trivia()
        && hash.kind() == SyntaxKind::Hash
    {
        start = hash.range().start;
    }
    let mut end = previous.range().end;
    if let Some(after) = previous.next_sibling_with_trivia()
        && after.kind().is_trivia()
    {
        end = after.range().end;
    }
    Some(Fix {
        description: "remove the manual vertical space".into(),
        edits: vec![edit_range(world, source, start..end, "")?],
    })
}

/// The nearest call of the given function that contains the span.
fn enclosing_call<'a>(
    source: &'a Source,
    span: Span,
    name: &str,
) -> Option<LinkedNode<'a>> {
    let mut node = LinkedNode::new(source.root()).find(span)?;
    loop {
        if let Some(call) = ast::FuncCall::from_untyped(&node)
            && let ast::Expr::Ident(callee) = call.callee()
            && callee.get() == name
        {
            return Some(node);
        }
        node = node.parent()?.clone();
    }
}

/// The call of one of the given functions at the span.
fn call_at<'a>(source: &'a Source, span: Span, names: &[&str]) -> Option<LinkedNode<'a>> {
    let node = LinkedNode::new(source.root()).find(span)?;
    let call = ast::FuncCall::from_untyped(&node)?;
    let ast::Expr::Ident(callee) = call.callee() else { return None };
    names.contains(&callee.get().as_str()).then_some(node)
}

struct NamedArg {
    value: Range<usize>,
}

fn named_arg(call: &LinkedNode, name: &str) -> Option<NamedArg> {
    let args = call.children().find(|child| child.kind() == SyntaxKind::Args)?;
    let named = args.children().find(|child| {
        ast::Named::from_untyped(child).is_some_and(|n| n.name().get() == name)
    })?;
    let value = named.children().next_back()?;
    Some(NamedArg { value: value.range() })
}

/// Sets a named argument, replacing its value or inserting it.
fn set_named(
    world: &LabWorld,
    source: &Source,
    call: &LinkedNode,
    name: &str,
    value: &str,
) -> Option<Edit> {
    if let Some(existing) = named_arg(call, name) {
        return edit_range(world, source, existing.value, value);
    }
    let args = call.children().find(|child| child.kind() == SyntaxKind::Args)?;
    let start = args.range().start;
    if source.text()[start..].starts_with('(') {
        let inner = &source.text()[start + 1..args.range().end];
        let separator = if inner.trim_start().starts_with(')') { "" } else { ", " };
        edit_range(
            world,
            source,
            start + 1..start + 1,
            &format!("{name}: {value}{separator}"),
        )
    } else {
        edit_range(world, source, start..start, &format!("({name}: {value})"))
    }
}

fn edit_range(
    world: &LabWorld,
    source: &Source,
    range: Range<usize>,
    text: &str,
) -> Option<Edit> {
    let lines = source.lines();
    let (line, col) = lines.byte_to_line_column(range.start)?;
    let (end_line, end_col) = lines.byte_to_line_column(range.end)?;
    Some(Edit {
        loc: SourceLoc {
            file: world.display_name(source.id()),
            line: line + 1,
            col: col + 1,
            end_line: end_line + 1,
            end_col: end_col + 1,
        },
        replacement: text.into(),
        range,
    })
}

/// Applies non-overlapping edits to a text. Overlapping edits after the first
/// are dropped.
pub fn apply(text: &str, edits: &[&Edit]) -> (String, usize) {
    let mut sorted: Vec<&&Edit> = edits.iter().collect();
    sorted.sort_by_key(|e| std::cmp::Reverse(e.range.start));
    let mut out = text.to_string();
    let mut applied = 0;
    let mut floor = usize::MAX;
    for edit in sorted {
        if edit.range.end > floor || edit.range.end > out.len() {
            continue;
        }
        out.replace_range(edit.range.clone(), &edit.replacement);
        floor = edit.range.start;
        applied += 1;
    }
    (out, applied)
}
