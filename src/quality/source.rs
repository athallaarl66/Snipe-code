use std::collections::BTreeSet;
use std::path::Path;

use proc_macro2::{TokenStream, TokenTree};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};

use super::{Comparison, Metric, Missing, Policy, State, excluded};

pub(super) fn outcome(p: &Policy, input: Result<(u64, Vec<String>), &'static str>) -> Metric {
    let (value, files, reason) = match input {
        Ok((value, files)) => (Some(value), files, None),
        Err(reason) => (None, vec![], Some(reason.to_string())),
    };
    let state = match value {
        Some(v) => {
            if match p.comparison {
                Comparison::Maximum => v <= p.threshold,
                Comparison::Minimum => v >= p.threshold,
            } {
                State::Pass
            } else {
                State::Fail
            }
        }
        None => {
            if p.unknown == Missing::Fail {
                State::Fail
            } else {
                State::Unknown
            }
        }
    };
    Metric {
        policy: p.clone(),
        value,
        state,
        reason,
        files,
        coverage: None,
    }
}

#[derive(Default)]
struct Functions {
    maximum: u64,
    unsupported: bool,
}
impl Functions {
    fn block(&mut self, b: &syn::Block) {
        self.maximum = self
            .maximum
            .max((b.span().end().line - b.span().start().line + 1) as u64);
    }
}
impl<'a> Visit<'a> for Functions {
    fn visit_item_fn(&mut self, f: &'a syn::ItemFn) {
        self.block(&f.block);
        visit::visit_item_fn(self, f);
    }
    fn visit_impl_item_fn(&mut self, f: &'a syn::ImplItemFn) {
        self.block(&f.block);
        visit::visit_impl_item_fn(self, f);
    }
    fn visit_trait_item_fn(&mut self, f: &'a syn::TraitItemFn) {
        if let Some(b) = &f.default {
            self.block(b);
        }
        visit::visit_trait_item_fn(self, f);
    }
    fn visit_item_macro(&mut self, _: &'a syn::ItemMacro) {
        self.unsupported = true;
    }
    fn visit_expr_closure(&mut self, _: &'a syn::ExprClosure) {
        self.unsupported = true;
    }
    fn visit_item_mod(&mut self, m: &'a syn::ItemMod) {
        if m.content.is_none() {
            self.unsupported = true;
        }
        visit::visit_item_mod(self, m);
    }
}

fn tokens(stream: TokenStream, out: &mut Vec<String>, depth: usize) -> Result<(), &'static str> {
    if depth > 64 {
        return Err("Rust nesting budget exceeded");
    }
    for t in stream {
        if out.len() >= 100_000 {
            return Err("Rust token budget exceeded");
        }
        match t {
            TokenTree::Group(g) => {
                out.push(format!("open:{:?}", g.delimiter()));
                tokens(g.stream(), out, depth + 1)?;
                out.push(format!("close:{:?}", g.delimiter()));
            }
            TokenTree::Ident(i) => out.push(format!("id:{i}")),
            TokenTree::Literal(l) => out.push(format!("literal:{l}")),
            TokenTree::Punct(p) => out.push(format!("punct:{}:{:?}", p.as_char(), p.spacing())),
        }
    }
    Ok(())
}

pub(super) fn rust(root: &Path, p: &Policy) -> Metric {
    outcome(p, measure(root, p))
}
fn measure(root: &Path, p: &Policy) -> Result<(u64, Vec<String>), &'static str> {
    let root = root.canonicalize().map_err(|_| "source root unavailable")?;
    let mut paths = crate::audit::walk_files(&root);
    paths.sort();
    if paths.len() > 4096 {
        return Err("source file budget exceeded");
    }
    let mut files = vec![];
    let mut maximum = 0;
    let mut duplicate = 0;
    let mut seen = BTreeSet::new();
    let mut total = 0;
    for file in paths {
        let relative = file
            .strip_prefix(&root)
            .map_err(|_| "source outside root")?
            .to_string_lossy()
            .replace('\\', "/");
        if excluded(&relative, &p.exclusions) {
            continue;
        }
        let extension = file.extension().and_then(|e| e.to_str()).unwrap_or("");
        if extension != "rs" {
            if [
                "js", "jsx", "ts", "tsx", "vue", "php", "go", "java", "cs", "py", "c", "cpp", "rb",
                "swift", "kt",
            ]
            .contains(&extension)
            {
                return Err("source language outside supported Rust slice");
            }
            continue;
        }
        let text = crate::audit::read_bounded_within_root(&root, &file, 262_144)
            .ok_or("source unreadable or exceeds budget")?;
        // ponytail: cheap conservative delimiter budget before recursive Rust parsing; literals may yield unknown.
        let mut depth = 0usize;
        for c in text.chars() {
            if matches!(c, '(' | '{' | '[') {
                depth += 1;
                if depth > 64 {
                    return Err("Rust nesting budget exceeded");
                }
            } else if matches!(c, ')' | '}' | ']') {
                depth = depth.saturating_sub(1);
            }
        }
        let parsed = syn::parse_file(&text).map_err(|_| "unsupported or invalid Rust source")?;
        if p.id == "rust-function-lines" {
            let mut functions = Functions::default();
            functions.visit_file(&parsed);
            if functions.unsupported {
                return Err(
                    "macro items, external modules or closures outside function metric slice",
                );
            }
            maximum = maximum.max(functions.maximum);
        } else {
            let stream: TokenStream = text.parse().map_err(|_| "invalid Rust tokens")?;
            let mut ts = vec![];
            tokens(stream, &mut ts, 0)?;
            total += ts.len();
            if total > 100_000 {
                return Err("Rust token budget exceeded");
            }
            let mut i = 0;
            while i + 50 <= ts.len() {
                let block = ts[i..i + 50].to_vec();
                if !seen.insert(block) {
                    duplicate += 1;
                }
                i += 50;
            }
        }
        files.push(relative);
    }
    if files.is_empty() {
        return Err("no supported Rust files");
    }
    Ok((
        if p.id == "rust-function-lines" {
            maximum
        } else {
            duplicate
        },
        files,
    ))
}
