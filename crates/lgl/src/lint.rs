use typst::syntax::ast::{self, AstNode, Unit};
use typst::syntax::{LinkedNode, Source, SyntaxKind, SyntaxNode};

use crate::diag::{self, Diagnostic, Severity};
use crate::overlap::ALLOW_OVERLAP;
use crate::world::LabWorld;

pub const NEGATIVE_SPACING: &str = "S001.negative-spacing";
pub const ABSOLUTE_PLACEMENT: &str = "S002.absolute-placement";
pub const NO_REFLOW: &str = "S003.transform-without-reflow";
pub const DISABLED_PROTECTION: &str = "S004.disabled-protection";
pub const MATH_LETTER_CALL: &str = "S005.math-letter-call";
pub const STRING_ESCAPE: &str = "S006.string-escape";
pub const STRAY_DELIMITER: &str = "S007.stray-delimiter";

/// Characters that need a backslash in markup but not inside a quoted
/// string. A quoted string knows only \\, \", \n, \r, \t and \u{..}; any
/// other backslash stays in the string and is printed.
const MARKUP_ONLY_ESCAPES: &[char] =
    &['$', '#', '*', '_', '@', '~', '<', '>', '[', ']', '`', '='];

/// Functions whose string arguments are patterns, code, data or paths rather
/// than text to print; a backslash there is meant.
const LITERAL_FUNCTIONS: &[&str] = &[
    "regex",
    "raw",
    "eval",
    "read",
    "json",
    "yaml",
    "toml",
    "csv",
    "xml",
    "cbor",
    "plugin",
    "image",
    "bibliography",
    "decode",
];

/// Negative spacing smaller than this is a typographic adjustment (kerning,
/// tightening a rule under a heading) rather than a layout hack.
const MAX_NEGATIVE_H_EM: f64 = 0.5;
const MAX_NEGATIVE_H_PT: f64 = 5.0;

/// Checks the source of the main file for constructs that tend to produce
/// layout defects. The findings are advisory; the geometry rules decide
/// whether a defect really occurred.
pub fn run(world: &LabWorld, source: &Source) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    visit(world, &LinkedNode::new(source.root()), &mut out);
    let mut letters = Vec::new();
    letter_functions(source.root(), &mut letters);
    if !letters.is_empty() {
        letter_calls(world, &LinkedNode::new(source.root()), &letters, &mut out);
    }
    out
}

/// One-letter names the document binds to functions (`let f(a, b) = ..` or
/// `let f = (a, b) => ..`).
fn letter_functions(node: &SyntaxNode, out: &mut Vec<String>) {
    if let Some(binding) = node.cast::<ast::LetBinding>() {
        let name = match binding.kind() {
            ast::LetBindingKind::Closure(name) => Some(name),
            ast::LetBindingKind::Normal(ast::Pattern::Normal(ast::Expr::Ident(name)))
                if matches!(binding.init(), Some(ast::Expr::Closure(_))) =>
            {
                Some(name)
            }
            _ => None,
        };
        if let Some(name) = name
            && name.get().chars().count() == 1
        {
            out.push(name.get().to_string());
        }
    }
    for child in node.children() {
        letter_functions(child, out);
    }
}

/// In math, a single letter is always a letter: `$f(2, 3)$` prints "f(2, 3)"
/// even when the document defines a function `f`. Only `#f(2, 3)` calls it.
fn letter_calls(
    world: &LabWorld,
    node: &LinkedNode,
    letters: &[String],
    out: &mut Vec<Diagnostic>,
) {
    if node.kind() == SyntaxKind::MathLetter
        && letters.iter().any(|l| l.as_str() == node.get().leaf_text().as_str())
        && let Some(next) = node.next_sibling()
        && next.kind() == SyntaxKind::MathDelimited
        && next.offset() == node.offset() + node.get().len()
        && next
            .children()
            .next()
            .is_some_and(|open| open.get().leaf_text() == "(")
    {
        let name = node.get().leaf_text();
        let mut diagnostic = finding(
            world,
            node,
            MATH_LETTER_CALL,
            format!(
                "{name}(..) inside math prints the letter {name} and the brackets: a                  one-letter name in math is never a function call"
            ),
            &format!("write #{name}(..) to call the function, or give it a longer name"),
        );
        diagnostic.severity = Severity::Error;
        out.push(diagnostic);
    }
    for child in node.children() {
        letter_calls(world, &child, letters, out);
    }
}

fn visit(world: &LabWorld, node: &LinkedNode, out: &mut Vec<Diagnostic>) {
    if let Some(call) = ast::FuncCall::from_untyped(node)
        && let ast::Expr::Ident(callee) = call.callee()
    {
        check_call(world, node, call, callee.get(), out);
    }
    if let Some(rule) = ast::SetRule::from_untyped(node) {
        check_set_rule(world, node, rule, out);
    }
    if node.kind() == SyntaxKind::Str {
        check_string(world, node, out);
    }
    if stray_delimiter(node) {
        let mut diagnostic = finding(
            world,
            node,
            STRAY_DELIMITER,
            format!(
                "a line holding only \"{}\" in markup is printed as text: it is left over \
                 from an edit that closed a call or block",
                node.get().leaf_text()
            ),
            "delete the line; in markup a closing bracket on its own is ordinary text",
        );
        diagnostic.anchor = Some(node.span());
        out.push(diagnostic);
    }
    for child in node.children() {
        visit(world, &child, out);
    }
}

/// Markup text that is nothing but closing brackets (and commas) on a line of
/// its own: a leftover of an edit that closed a call or a block twice. Typst
/// prints it as ordinary text, so the syntax check passes.
pub(crate) fn stray_delimiter(node: &LinkedNode) -> bool {
    let text = node.get().leaf_text();
    let breaks = |sibling: Option<LinkedNode>| match sibling {
        None => true,
        Some(s) => {
            s.kind() == SyntaxKind::Parbreak
                || (s.kind() == SyntaxKind::Space && s.get().leaf_text().contains('\n'))
        }
    };
    node.kind() == SyntaxKind::Text
        && node.parent().is_some_and(|p| p.kind() == SyntaxKind::Markup)
        && text.contains(')')
        && text.chars().all(|c| c == ')' || c == ',')
        && breaks(node.prev_sibling_with_trivia())
        && breaks(node.next_sibling_with_trivia())
        && !open_bracket_before(node)
}

/// Whether the text of the paragraph before the node leaves a bracket open,
/// which the node may close: "(see the note" wrapped onto a new line.
fn open_bracket_before(node: &LinkedNode) -> bool {
    let mut open = 0i64;
    let mut cursor = node.prev_sibling_with_trivia();
    while let Some(sibling) = cursor {
        if sibling.kind() == SyntaxKind::Parbreak {
            break;
        }
        if sibling.kind() == SyntaxKind::Text {
            let text = sibling.get().leaf_text();
            open += text.matches('(').count() as i64 - text.matches(')').count() as i64;
        }
        cursor = sibling.prev_sibling_with_trivia();
    }
    open > 0
}

/// A quoted string with a markup escape such as `\$`: the backslash is not an
/// escape inside quotes and is printed ("\$84.6 M" shows as \$84.6 M).
fn check_string(world: &LabWorld, node: &LinkedNode, out: &mut Vec<Diagnostic>) {
    let text = node.get().leaf_text();
    let fixed = without_markup_escapes(text);
    if fixed == text.as_str() || literal_argument(node) {
        return;
    }
    let shown: String = text.chars().take(40).collect();
    let mut diagnostic = finding(
        world,
        node,
        STRING_ESCAPE,
        format!(
            "the quoted string {shown} prints its backslashes: inside quotes only \\\\, \
             \\\", \\n, \\r, \\t and \\u{{..}} are escapes, so a character such as $ or # \
             needs no backslash there"
        ),
        "write the character without a backslash inside a quoted string (\"$84.6\"); \
         escape it only in markup",
    );
    diagnostic.anchor = Some(node.span());
    out.push(diagnostic);
}

/// The text of a quoted string without the backslashes that only markup
/// needs. Real escapes, including an escaped backslash, are kept.
pub(crate) fn without_markup_escapes(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\'
            && let Some(&next) = chars.peek()
        {
            if !MARKUP_ONLY_ESCAPES.contains(&next) {
                out.push(c);
                out.push(next);
            } else {
                out.push(next);
            }
            chars.next();
            continue;
        }
        out.push(c);
    }
    out
}

/// Whether the string is an argument of a function that reads it as a
/// pattern, code, data or a path.
fn literal_argument(node: &LinkedNode) -> bool {
    let mut cursor = node.parent();
    while let Some(parent) = cursor {
        if let Some(call) = ast::FuncCall::from_untyped(parent) {
            let name = match call.callee() {
                ast::Expr::Ident(ident) => ident.get().clone(),
                ast::Expr::FieldAccess(access) => access.field().get().clone(),
                _ => return false,
            };
            return LITERAL_FUNCTIONS.contains(&name.as_str());
        }
        if matches!(
            parent.kind(),
            SyntaxKind::ContentBlock | SyntaxKind::CodeBlock | SyntaxKind::Closure
        ) {
            return false;
        }
        cursor = parent.parent();
    }
    false
}

fn check_call(
    world: &LabWorld,
    node: &LinkedNode,
    call: ast::FuncCall,
    name: &str,
    out: &mut Vec<Diagnostic>,
) {
    match name {
        "h" | "v" => {
            let Some(ast::Arg::Pos(amount)) = call.args().items().next() else { return };
            let Some((value, unit)) = negative_length(amount) else { return };
            let large = match unit {
                Unit::Em => value >= MAX_NEGATIVE_H_EM,
                Unit::Pt => value >= MAX_NEGATIVE_H_PT,
                Unit::Mm => value * 72.0 / 25.4 >= MAX_NEGATIVE_H_PT,
                Unit::Cm => value * 72.0 / 2.54 >= MAX_NEGATIVE_H_PT,
                Unit::In => value * 72.0 >= MAX_NEGATIVE_H_PT,
                _ => true,
            };
            if large {
                out.push(finding(
                    world,
                    node,
                    NEGATIVE_SPACING,
                    format!("negative {name}() spacing pulls content onto other content"),
                    "remove the negative spacing and let layout place the content",
                ));
            }
        }
        "place" | "move"
            if !allowed(node) && !in_page_setting(node) && !in_sized_container(node) =>
        {
            if name == "place" && named_bool(call, "float") == Some(true) {
                return;
            }
            out.push(finding(
                world,
                node,
                ABSOLUTE_PLACEMENT,
                format!("{name}() positions content absolutely, so it can overlap other content"),
                "keep content in the normal flow, use place(float: true) for floats, \
                 or label an intended overlay with <allow-overlap>",
            ));
        }
        "rotate" | "scale" | "skew" => {
            if named_bool(call, "reflow") == Some(true) {
                return;
            }
            let mut diagnostic = finding(
                world,
                node,
                NO_REFLOW,
                format!(
                    "{name}() without reflow: true does not reserve space for its result"
                ),
                "add reflow: true so that surrounding content makes room",
            );
            diagnostic.anchor = Some(node.span());
            out.push(diagnostic);
        }
        _ => {}
    }
}

fn check_set_rule(
    world: &LabWorld,
    node: &LinkedNode,
    rule: ast::SetRule,
    out: &mut Vec<Diagnostic>,
) {
    let ast::Expr::Ident(target) = rule.target() else { return };
    let disabled = match target.get().as_str() {
        "text" => rule.args().items().any(|arg| match arg {
            ast::Arg::Named(named) => {
                named.name().get() == "costs"
                    && named.expr().to_untyped().full_text().contains("0%")
            }
            _ => false,
        }),
        "block" => {
            named_bool_args(rule.args(), "sticky") == Some(false)
                && in_heading_show_rule(node)
        }
        _ => false,
    };
    if disabled {
        let mut diagnostic = finding(
            world,
            node,
            DISABLED_PROTECTION,
            "this rule disables Typst's protection against orphans, widows or stranded \
             headings"
                .into(),
            "remove the rule and keep Typst's defaults",
        );
        diagnostic.anchor = Some(node.span());
        out.push(diagnostic);
    }
}

fn finding(
    world: &LabWorld,
    node: &LinkedNode,
    rule: &'static str,
    message: String,
    hint: &str,
) -> Diagnostic {
    let mut out = Diagnostic::new(rule, Severity::Warning, message);
    out.loc = diag::resolve_span(world, node.span());
    out.hints.push(hint.into());
    out
}

/// The magnitude and unit of a negative length literal such as `-1em`.
fn negative_length(expr: ast::Expr) -> Option<(f64, Unit)> {
    let ast::Expr::Unary(unary) = expr else { return None };
    if unary.op() != ast::UnOp::Neg {
        return None;
    }
    let ast::Expr::Numeric(numeric) = unary.expr() else { return None };
    let (value, unit) = numeric.get();
    (value > 0.0).then_some((value, unit))
}

fn named_bool(call: ast::FuncCall, name: &str) -> Option<bool> {
    named_bool_args(call.args(), name)
}

fn named_bool_args(args: ast::Args, name: &str) -> Option<bool> {
    args.items().find_map(|arg| match arg {
        ast::Arg::Named(named) if named.name().get() == name => match named.expr() {
            ast::Expr::Bool(value) => Some(value.get()),
            _ => None,
        },
        _ => None,
    })
}

/// Whether the call is followed by the label that marks an intended overlay.
fn allowed(node: &LinkedNode) -> bool {
    node.next_sibling().is_some_and(|next| {
        ast::Label::from_untyped(&next).is_some_and(|label| label.get() == ALLOW_OVERLAP)
    })
}

/// Whether the node is part of a page setting such as a background,
/// foreground, header or footer, where absolute placement is normal.
fn in_page_setting(node: &LinkedNode) -> bool {
    let mut cursor = node.parent();
    while let Some(parent) = cursor {
        if let Some(named) = ast::Named::from_untyped(parent)
            && matches!(
                named.name().get().as_str(),
                "background" | "foreground" | "header" | "footer"
            )
        {
            return true;
        }
        cursor = parent.parent();
    }
    false
}

/// Whether the node is inside a box or shape of fixed size: a small drawing
/// area in which absolute placement cannot reach other content. Content that
/// leaves the area is reported by L002.
fn in_sized_container(node: &LinkedNode) -> bool {
    let named = |call: ast::FuncCall, name: &str| {
        call.args()
            .items()
            .any(|arg| matches!(arg, ast::Arg::Named(n) if n.name().get() == name))
    };
    let mut cursor = node.parent();
    while let Some(parent) = cursor {
        if let Some(call) = ast::FuncCall::from_untyped(parent)
            && let ast::Expr::Ident(ident) = call.callee()
            && matches!(
                ident.get().as_str(),
                "box" | "block" | "rect" | "square" | "circle" | "ellipse"
            )
            && ((named(call, "width") && named(call, "height"))
                || named(call, "size")
                || named(call, "radius"))
        {
            return true;
        }
        cursor = parent.parent();
    }
    false
}

/// Whether the node is inside a show rule whose selector mentions headings.
fn in_heading_show_rule(node: &LinkedNode) -> bool {
    let mut cursor = node.parent();
    while let Some(parent) = cursor {
        if let Some(rule) = ast::ShowRule::from_untyped(parent)
            && rule
                .selector()
                .is_some_and(|s| s.to_untyped().full_text().contains("heading"))
        {
            return true;
        }
        cursor = parent.parent();
    }
    false
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    fn rules_for(text: &str) -> Vec<String> {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!("lgl-lint-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("case-{}.typ", NEXT.fetch_add(1, Ordering::Relaxed)));
        std::fs::write(&path, text).unwrap();
        let world = LabWorld::new(&path, &crate::world::WorldOptions::default()).unwrap();
        let source = typst::World::source(&world, typst::World::main(&world)).unwrap();
        run(&world, &source).into_iter().map(|d| d.rule).collect()
    }

    #[test]
    fn negative_vertical_spacing_is_flagged() {
        assert_eq!(rules_for("A\n#v(-2em)\nB"), [NEGATIVE_SPACING]);
    }

    #[test]
    fn small_negative_kerning_is_fine() {
        assert!(rules_for("A#h(-0.1em)B").is_empty());
        assert!(rules_for("A\n#v(-0.3em)\nB").is_empty());
    }

    #[test]
    fn absolute_place_is_flagged_unless_floating_or_labelled() {
        assert_eq!(rules_for("#place(top)[x]"), [ABSOLUTE_PLACEMENT]);
        assert!(rules_for("#place(top, float: true)[x]").is_empty());
        assert!(rules_for("#place(center)[x] <allow-overlap>").is_empty());
        assert!(rules_for("#set page(background: place(center)[x])").is_empty());
        assert!(
            rules_for(
                "#box(width: 40pt, height: 40pt)[#place(center, circle(radius: 5pt))]"
            )
            .is_empty()
        );
        assert_eq!(
            rules_for("#box(width: 40pt)[#place(center)[x]]"),
            [ABSOLUTE_PLACEMENT]
        );
    }

    #[test]
    fn transforms_need_reflow() {
        assert_eq!(rules_for("#rotate(90deg)[x]"), [NO_REFLOW]);
        assert!(rules_for("#rotate(90deg, reflow: true)[x]").is_empty());
    }

    #[test]
    fn markup_escapes_in_strings_are_flagged() {
        assert_eq!(rules_for("#let k = (\"\\$84.6 M\",)\n#k.at(0)"), [STRING_ESCAPE]);
        assert_eq!(rules_for("#table(columns: 1, \"Revenue (\\$ m)\")"), [STRING_ESCAPE]);
        assert!(rules_for("#\"\\\\$ and \\n\"").is_empty());
        assert!(rules_for("#\"plain $ text\"").is_empty());
        assert!(rules_for("#\"a$b\".replace(regex(\"\\$\"), \"\")").is_empty());
        assert!(rules_for("#raw(\"price: \\$5\")").is_empty());
        assert!(rules_for("Markup \\$5 is fine.").is_empty());
    }

    #[test]
    fn stray_delimiters_are_flagged() {
        assert_eq!(rules_for("#grid(\n  [a],\n)\n)\nText after."), [STRAY_DELIMITER]);
        assert_eq!(rules_for("Text before.\n\n  ),\n\nText after."), [STRAY_DELIMITER]);
        assert!(rules_for("A note (see the table\n)\nends here.").is_empty());
        assert!(rules_for("A note (see the table) ends here.").is_empty());
        assert!(rules_for("#grid(\n  [a],\n)\nText after.").is_empty());
    }

    #[test]
    fn markup_escapes_are_removed() {
        assert_eq!(without_markup_escapes("\"\\$84.6 \\#1\""), "\"$84.6 #1\"");
        assert_eq!(without_markup_escapes("\"\\\\$ \\n \\\"\""), "\"\\\\$ \\n \\\"\"");
        assert_eq!(without_markup_escapes("\"\\d+\""), "\"\\d+\"");
    }

    #[test]
    fn disabled_protections_are_flagged() {
        assert_eq!(rules_for("#set text(costs: (orphan: 0%))"), [DISABLED_PROTECTION]);
        assert_eq!(
            rules_for("#show heading: set block(sticky: false)"),
            [DISABLED_PROTECTION]
        );
        assert!(rules_for("#set block(sticky: false)").is_empty());
    }
}
