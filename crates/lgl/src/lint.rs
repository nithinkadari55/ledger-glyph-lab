use typst::syntax::ast::{self, AstNode, Unit};
use typst::syntax::{LinkedNode, Source};

use crate::diag::{self, Diagnostic, Severity};
use crate::overlap::ALLOW_OVERLAP;
use crate::world::LabWorld;

pub const NEGATIVE_SPACING: &str = "S001.negative-spacing";
pub const ABSOLUTE_PLACEMENT: &str = "S002.absolute-placement";
pub const NO_REFLOW: &str = "S003.transform-without-reflow";
pub const DISABLED_PROTECTION: &str = "S004.disabled-protection";

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
    out
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
    for child in node.children() {
        visit(world, &child, out);
    }
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
        "place" | "move" if !allowed(node) && !in_page_setting(node) => {
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
                format!("{name}() without reflow: true does not reserve space for its result"),
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
    }

    #[test]
    fn transforms_need_reflow() {
        assert_eq!(rules_for("#rotate(90deg)[x]"), [NO_REFLOW]);
        assert!(rules_for("#rotate(90deg, reflow: true)[x]").is_empty());
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
