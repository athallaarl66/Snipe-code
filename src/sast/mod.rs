use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprCall, ExprMacro, ExprMethodCall, ItemFn, Stmt};

pub const RULES: &str = include_str!("rules.json");
pub const PACK_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SastStatus {
    Evaluated,
    Unknown,
    Error,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceEvidence {
    pub kind: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SastFinding {
    pub rule_id: String,
    pub rule_version: String,
    pub severity: String,
    pub confidence: String,
    pub candidate: bool,
    pub file: String,
    pub line: usize,
    pub evidence: Vec<SourceEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SastReport {
    #[serde(default)]
    pub file: String,
    pub status: SastStatus,
    pub language: String,
    pub rule_pack_version: String,
    pub findings: Vec<SastFinding>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RuleCatalog {
    pack_version: String,
    rules: Vec<RuleMetadata>,
}

#[derive(Debug, Deserialize)]
struct RuleMetadata {
    id: String,
    version: String,
    severity: String,
    confidence: String,
}

#[derive(Debug, Clone, Default)]
struct Taint {
    sources: Vec<usize>,
    unknown: bool,
    sanitized: bool,
}

impl Taint {
    fn merge(&mut self, other: Self) {
        self.unknown |= other.unknown;
        self.sanitized = false;
        self.sources.extend(other.sources);
        self.sources.sort_unstable();
        self.sources.dedup();
    }

    fn is_tainted(&self) -> bool {
        !self.sources.is_empty()
    }
}

/// Analyze only Rust constructs explicitly listed in the candidate rule pack.
/// This adapter does not claim interprocedural or whole-program data flow.
pub fn analyze_rust_source(file: &str, source: &str) -> SastReport {
    // ponytail: bounded size/depth instead of a wall-clock parser timeout.
    if source.len() > 262_144 || nesting_exceeded(source) {
        let mut report = error_report(file, "parser input budget exceeded".into());
        report.status = SastStatus::Unknown;
        return report;
    }
    let catalog = match serde_json::from_str::<RuleCatalog>(RULES) {
        Ok(catalog) => catalog,
        Err(_) => return error_report(file, "invalid rule catalog".into()),
    };
    let parsed = match syn::parse_file(source) {
        Ok(parsed) => parsed,
        Err(_) => return error_report(file, "Rust parse error".into()),
    };
    let mut analyzer = FunctionVisitor {
        file,
        rules: &catalog.rules,
        findings: Vec::new(),
        unknown: false,
    };
    analyzer.visit_file(&parsed);
    analyzer.findings.sort_by(|left, right| {
        (&left.rule_id, left.line, &left.file).cmp(&(&right.rule_id, right.line, &right.file))
    });
    analyzer.findings.dedup_by(|left, right| {
        left.rule_id == right.rule_id && left.line == right.line && left.file == right.file
    });
    SastReport {
        file: file.to_string(),
        status: if analyzer.unknown {
            SastStatus::Unknown
        } else {
            SastStatus::Evaluated
        },
        language: "rust".to_string(),
        rule_pack_version: catalog.pack_version,
        findings: analyzer.findings,
        error: None,
    }
}

fn error_report(file: &str, error: String) -> SastReport {
    SastReport {
        file: file.to_string(),
        status: SastStatus::Error,
        language: "rust".to_string(),
        rule_pack_version: "1.0.0".to_string(),
        findings: Vec::new(),
        error: Some(error),
    }
}

struct FunctionVisitor<'a> {
    file: &'a str,
    rules: &'a [RuleMetadata],
    findings: Vec<SastFinding>,
    unknown: bool,
}

impl FunctionVisitor<'_> {
    fn analyze_block(&mut self, block: &syn::Block) {
        let mut vars = BTreeMap::<String, Taint>::new();
        for statement in &block.stmts {
            let mut expressions = ExpressionVisitor {
                file: self.file,
                rules: self.rules,
                vars: &vars,
                findings: &mut self.findings,
                unknown: &mut self.unknown,
            };
            match statement {
                Stmt::Local(local) => {
                    if let Some(initializer) = &local.init {
                        expressions.visit_expr(&initializer.expr);
                        let value = evaluate(&initializer.expr, &vars);
                        if let syn::Pat::Ident(pattern) = &local.pat {
                            self.unknown |= value.unknown;
                            vars.insert(pattern.ident.to_string(), value);
                        } else if !matches!(local.pat, syn::Pat::Wild(_)) {
                            self.unknown = true;
                        }
                    }
                }
                Stmt::Expr(expression, _) => expressions.visit_expr(expression),
                Stmt::Item(_) | Stmt::Macro(_) => self.unknown = true,
            }
        }
    }
}

impl<'ast> Visit<'ast> for FunctionVisitor<'_> {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.analyze_block(&node.block);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.analyze_block(&node.block);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        if let Some(block) = &node.default {
            self.analyze_block(block);
        }
    }
}

struct ExpressionVisitor<'a> {
    file: &'a str,
    rules: &'a [RuleMetadata],
    vars: &'a BTreeMap<String, Taint>,
    findings: &'a mut Vec<SastFinding>,
    unknown: &'a mut bool,
}

impl ExpressionVisitor<'_> {
    fn inspect(&mut self, expression: &Expr) {
        if let Expr::MethodCall(call) = expression {
            if matches!(
                call.method.to_string().as_str(),
                "query" | "query_row" | "execute" | "prepare" | "arg"
            ) && call
                .args
                .first()
                .is_some_and(|arg| evaluate(arg, self.vars).unknown)
            {
                *self.unknown = true;
            }
        }
        if let Some((rule_id, evidence)) = shell_sink(expression, self.vars) {
            self.push(rule_id, expression.span().start().line, evidence);
        }
        if let Some((rule_id, evidence)) = sql_sink(expression, self.vars) {
            self.push(rule_id, expression.span().start().line, evidence);
        }
        if let Some((rule_id, evidence)) = html_sink(expression, self.vars) {
            self.push(rule_id, expression.span().start().line, evidence);
        }
    }

    fn push(&mut self, rule_id: &str, line: usize, mut evidence: Vec<SourceEvidence>) {
        let Some(rule) = self.rules.iter().find(|rule| rule.id == rule_id) else {
            *self.unknown = true;
            return;
        };
        evidence.push(SourceEvidence {
            kind: "sink".to_string(),
            line,
        });
        evidence.sort_by(|left, right| (left.line, &left.kind).cmp(&(right.line, &right.kind)));
        evidence.dedup_by(|left, right| left.line == right.line && left.kind == right.kind);
        self.findings.push(SastFinding {
            rule_id: rule.id.clone(),
            rule_version: rule.version.clone(),
            severity: rule.severity.clone(),
            confidence: rule.confidence.clone(),
            candidate: true,
            file: self.file.to_string(),
            line,
            evidence,
        });
    }
}

impl<'ast> Visit<'ast> for ExpressionVisitor<'_> {
    fn visit_expr(&mut self, expression: &'ast Expr) {
        if let Expr::Call(call) = expression {
            if call
                .args
                .iter()
                .any(|arg| evaluate(arg, self.vars).is_tainted())
            {
                let name = if let Expr::Path(path) = call.func.as_ref() {
                    path_text(&path.path)
                } else {
                    String::new()
                };
                if !matches!(
                    name.as_str(),
                    "axum::response::Html"
                        | "warp::reply::html"
                        | "ammonia::clean"
                        | "html_escape::encode_safe"
                ) {
                    *self.unknown = true;
                }
            }
        }
        if matches!(
            expression,
            Expr::If(_)
                | Expr::Match(_)
                | Expr::Loop(_)
                | Expr::ForLoop(_)
                | Expr::While(_)
                | Expr::Closure(_)
                | Expr::Assign(_)
                | Expr::Block(_)
        ) {
            *self.unknown = true;
            return;
        }
        if let Expr::Macro(mac) = expression {
            if path_text(&mac.mac.path) != "format" {
                *self.unknown = true;
            }
        }
        self.inspect(expression);
        visit::visit_expr(self, expression);
    }
}

fn shell_sink(
    expression: &Expr,
    vars: &BTreeMap<String, Taint>,
) -> Option<(&'static str, Vec<SourceEvidence>)> {
    let (base, methods) = method_chain(expression)?;

    let Expr::Call(call) = base else { return None };
    let Expr::Path(path) = call.func.as_ref() else {
        return None;
    };
    if !matches!(
        path_text(&path.path).as_str(),
        "Command::new" | "std::process::Command::new"
    ) {
        return None;
    }
    let executable = call.args.first()?;
    if !matches!(literal_string(executable)?.as_str(), "sh" | "bash") {
        return None;
    }
    let mut arguments = methods
        .iter()
        .filter(|(name, _)| name == "arg")
        .filter_map(|(_, args)| args.first());
    if arguments.next().and_then(literal_string).as_deref() != Some("-c") {
        return None;
    }
    // Only the argument immediately after -c is shell source; later args are positional.
    let argument = arguments.next()?;
    let taint = evaluate(argument, vars);
    if taint.unknown || !taint.is_tainted() {
        return None;
    }
    let mut evidence = taint_evidence(&taint);
    evidence.push(SourceEvidence {
        kind: "shell-argument".to_string(),
        line: argument.span().start().line,
    });
    Some(("sast-rust-shell-tainted", evidence))
}

fn sql_sink(
    expression: &Expr,
    vars: &BTreeMap<String, Taint>,
) -> Option<(&'static str, Vec<SourceEvidence>)> {
    let Expr::MethodCall(ExprMethodCall { method, args, .. }) = expression else {
        return None;
    };
    if !matches!(
        method.to_string().as_str(),
        "query" | "query_row" | "execute" | "prepare"
    ) {
        return None;
    }
    let query = args.first()?;
    let taint = evaluate(query, vars);
    if !taint.is_tainted() || taint.unknown {
        return None;
    }
    let mut evidence = taint_evidence(&taint);
    evidence.push(SourceEvidence {
        kind: "sql-format".to_string(),
        line: query.span().start().line,
    });
    evidence.push(SourceEvidence {
        kind: "query-method-type-unverified".to_string(),
        line: expression.span().start().line,
    });
    Some(("sast-rust-sql-tainted-format", evidence))
}

fn html_sink(
    expression: &Expr,
    vars: &BTreeMap<String, Taint>,
) -> Option<(&'static str, Vec<SourceEvidence>)> {
    let Expr::Call(ExprCall { func, args, .. }) = expression else {
        return None;
    };
    let Expr::Path(path) = func.as_ref() else {
        return None;
    };
    if !matches!(
        path_text(&path.path).as_str(),
        "axum::response::Html" | "warp::reply::html"
    ) {
        return None;
    }
    let input = args.first()?;
    let taint = evaluate(input, vars);
    if !taint.is_tainted() || taint.sanitized || taint.unknown {
        return None;
    }
    let mut evidence = taint_evidence(&taint);
    evidence.push(SourceEvidence {
        kind: "html-response".to_string(),
        line: expression.span().start().line,
    });
    Some(("sast-rust-html-tainted-output", evidence))
}

type MethodChain = Vec<(String, Vec<Expr>)>;

fn method_chain(expression: &Expr) -> Option<(&Expr, MethodChain)> {
    let Expr::MethodCall(call) = expression else {
        return Some((expression, Vec::new()));
    };
    let (base, mut methods) = method_chain(&call.receiver)?;
    methods.push((call.method.to_string(), call.args.iter().cloned().collect()));
    Some((base, methods))
}

fn evaluate(expression: &Expr, vars: &BTreeMap<String, Taint>) -> Taint {
    match expression {
        Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => vars
            .get(&path.path.segments[0].ident.to_string())
            .cloned()
            .unwrap_or(Taint {
                unknown: true,
                ..Taint::default()
            }),
        Expr::Call(call) => {
            let name = match call.func.as_ref() {
                Expr::Path(path) => path_text(&path.path),
                _ => String::new(),
            };
            if matches!(
                name.as_str(),
                "std::env::var" | "std::env::args" | "env::var" | "env::args"
            ) {
                return Taint {
                    sources: vec![call.span().start().line],
                    ..Taint::default()
                };
            }
            if matches!(name.as_str(), "ammonia::clean" | "html_escape::encode_safe") {
                let input = call
                    .args
                    .first()
                    .map(|arg| evaluate(arg, vars))
                    .unwrap_or_default();
                return Taint {
                    sanitized: input.is_tainted(),
                    ..input
                };
            }
            if name == "format" {
                return evaluate_macro_tokens(&call.args, vars, call.span().start().line);
            }
            let mut result = evaluate(&call.func, vars);
            for argument in &call.args {
                result.merge(evaluate(argument, vars));
            }
            if result.is_tainted() {
                result.unknown = true;
            }
            result
        }
        Expr::MethodCall(call) => {
            let mut result = evaluate(&call.receiver, vars);
            for argument in &call.args {
                result.merge(evaluate(argument, vars));
            }
            let method = call.method.to_string();
            if result.is_tainted()
                && !matches!(
                    method.as_str(),
                    "unwrap"
                        | "expect"
                        | "unwrap_or"
                        | "unwrap_or_default"
                        | "nth"
                        | "next"
                        | "to_owned"
                        | "to_string"
                        | "as_str"
                        | "as_ref"
                        | "into"
                )
            {
                result.unknown = true;
            }
            result
        }
        Expr::Macro(mac) if path_text(&mac.mac.path) == "format" => evaluate_macro(mac, vars),
        Expr::Try(value) => evaluate(&value.expr, vars),
        Expr::Reference(reference) => evaluate(&reference.expr, vars),
        Expr::Paren(paren) => evaluate(&paren.expr, vars),
        Expr::Group(group) => evaluate(&group.expr, vars),
        Expr::Unary(unary) => evaluate(&unary.expr, vars),
        Expr::Cast(cast) => evaluate(&cast.expr, vars),
        Expr::Binary(binary) => {
            let mut left = evaluate(&binary.left, vars);
            left.merge(evaluate(&binary.right, vars));
            left
        }
        Expr::Block(block) => evaluate_block(&block.block, vars),
        Expr::If(expr) => {
            let mut result = evaluate(&expr.cond, vars);
            if let Some((_, block)) = &expr.else_branch {
                result.merge(evaluate(block, vars));
            }
            for statement in &expr.then_branch.stmts {
                result.merge(evaluate_statement(statement, vars));
            }
            if result.is_tainted() {
                result.unknown = true;
            }
            result
        }
        Expr::Array(array) => merge_exprs(array.elems.iter(), vars),
        Expr::Tuple(tuple) => merge_exprs(tuple.elems.iter(), vars),
        Expr::Field(field) => {
            let mut result = evaluate(&field.base, vars);
            if result.is_tainted() {
                result.unknown = true;
            }
            result
        }
        _ => Taint::default(),
    }
}

fn evaluate_block(block: &syn::Block, vars: &BTreeMap<String, Taint>) -> Taint {
    let mut result = Taint::default();
    for statement in &block.stmts {
        result.merge(evaluate_statement(statement, vars));
    }
    result
}

fn evaluate_statement(statement: &Stmt, vars: &BTreeMap<String, Taint>) -> Taint {
    match statement {
        Stmt::Local(local) => local
            .init
            .as_ref()
            .map(|init| evaluate(&init.expr, vars))
            .unwrap_or_default(),
        Stmt::Expr(expression, _) => evaluate(expression, vars),
        Stmt::Item(_) | Stmt::Macro(_) => Taint::default(),
    }
}

fn merge_exprs<'a>(
    expressions: impl Iterator<Item = &'a Expr>,
    vars: &BTreeMap<String, Taint>,
) -> Taint {
    let mut result = Taint::default();
    for expression in expressions {
        result.merge(evaluate(expression, vars));
    }
    if result.is_tainted() {
        result.unknown = true;
    }
    result
}

fn evaluate_macro_tokens(
    args: &syn::punctuated::Punctuated<Expr, syn::Token![,]>,
    vars: &BTreeMap<String, Taint>,
    line: usize,
) -> Taint {
    let mut result = Taint::default();
    for argument in args {
        result.merge(evaluate(argument, vars));
    }
    if result.is_tainted() {
        result.sources.push(line);
    }
    result
}

fn evaluate_macro(mac: &ExprMacro, vars: &BTreeMap<String, Taint>) -> Taint {
    use syn::parse::Parser;
    let Ok(args) = syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated
        .parse2(mac.mac.tokens.clone())
    else {
        return Taint {
            unknown: true,
            ..Taint::default()
        };
    };
    let Some(format) = args.first().and_then(literal_string) else {
        return Taint {
            unknown: true,
            ..Taint::default()
        };
    };
    let mut result = Taint::default();
    let mut chars = format.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '{' {
            continue;
        }
        if chars.peek() == Some(&'{') {
            chars.next();
            continue;
        }
        let field: String = chars.by_ref().take_while(|c| *c != '}').collect();
        let name = field.split([':', '!']).next().unwrap_or("");
        if !name.is_empty() && !name.chars().all(|c| c.is_ascii_digit()) {
            match vars.get(name) {
                Some(taint) => result.merge(taint.clone()),
                None => result.unknown = true,
            }
        }
    }
    for argument in args.iter().skip(1) {
        result.merge(evaluate(argument, vars));
    }
    result
}

fn nesting_exceeded(source: &str) -> bool {
    let mut depth = 0usize;
    let mut operations = 0usize;
    for byte in source.bytes() {
        if matches!(byte, b'.' | b'!' | b'+' | b'-' | b'&' | b'*' | b'=') {
            operations += 1;
            if operations > 512 {
                return true;
            }
        }
        if matches!(byte, b'(' | b'[' | b'{') {
            depth += 1;
        }
        if depth > 64 {
            return true;
        }
        if matches!(byte, b')' | b']' | b'}') {
            depth = depth.saturating_sub(1);
        }
    }
    false
}

fn literal_string(expression: &Expr) -> Option<String> {
    let Expr::Lit(literal) = expression else {
        return None;
    };
    let syn::Lit::Str(value) = &literal.lit else {
        return None;
    };
    Some(value.value())
}

fn path_text(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn taint_evidence(taint: &Taint) -> Vec<SourceEvidence> {
    taint
        .sources
        .iter()
        .map(|line| SourceEvidence {
            kind: "taint-source".to_string(),
            line: *line,
        })
        .collect()
}
