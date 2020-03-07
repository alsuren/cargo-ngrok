//! Parsing Rust source code.
//! Original code is lifted from cargo-fixeq, and then adapted
//! for my purposes.

use anyhow::{Context, Result};
use proc_macro2::LineColumn;
use quote::ToTokens;
use syn::visit::visit_expr_method_call;
use syn::{spanned::Spanned, visit::Visit, Attribute, ExprMethodCall, ItemFn};

/// there can be only one
fn highlander<T>(mut things: Vec<T>) -> Result<T> {
    if things.len() != 1 {
        anyhow::bail!("Found {} things. There should be only one.", things.len())
    }
    things.pop().ok_or_else(|| panic!())
}

/// Find locations of `#[get("/")]`s from source code.
pub(crate) fn find_handler_attrs(code: &str) -> Result<Location> {
    let mut visitor = AttrVisitor::new(r#"#[get("/")"#.into());
    if let Ok(syntax_tree) = syn::parse_file(&code) {
        visitor.visit_file(&syntax_tree);
    }

    highlander(visitor.out).context("should be exactly one '/' handler")
}

/// Find locations of `#[actix_rt::test]`s from source code.
pub(crate) fn find_test_attrs(code: &str) -> Result<Location> {
    let mut visitor = AttrVisitor::new("#[actix_rt::test]".into());
    if let Ok(syntax_tree) = syn::parse_file(&code) {
        visitor.visit_file(&syntax_tree);
    }

    visitor
        .out
        .pop()
        .ok_or_else(|| anyhow::anyhow!("Could not find any #[actix_rt::test] annotations"))
}

#[derive(Clone)]
pub(crate) struct Location {
    pub(crate) start: LineColumn,
    pub(crate) end: LineColumn,
}

struct AttrVisitor {
    searching_for: String,
    out: Vec<Location>,
}

impl AttrVisitor {
    fn new(searching_for: String) -> Self {
        AttrVisitor {
            searching_for,
            out: Vec::default(),
        }
    }
}

impl<'ast> Visit<'ast> for AttrVisitor {
    fn visit_attribute(&mut self, i: &'ast Attribute) {
        if i.to_token_stream()
            .to_string()
            .replace(" ", "")
            .starts_with(&self.searching_for)
        {
            self.out.push(Location {
                start: i.pound_token.span().start(),
                end: i.bracket_token.span.end(),
            });
        }
    }
}

/// Find names of #[get("/route")] handler functions from source code.
pub(crate) fn find_handler_function_names(code: &str, route: &str) -> Result<String> {
    let mut visitor = FnVisitor::new(format!(r#"#[get("{}")]"#, route));
    if let Ok(syntax_tree) = syn::parse_file(&code) {
        visitor.visit_file(&syntax_tree);
    }

    highlander(visitor.out).with_context(|| format!("should be only one {} handler", route))
}

struct FnVisitor {
    searching_for: String,
    out: Vec<String>,
}

impl FnVisitor {
    fn new(searching_for: String) -> Self {
        FnVisitor {
            searching_for,
            out: Vec::default(),
        }
    }
}

impl<'ast> Visit<'ast> for FnVisitor {
    fn visit_item_fn(&mut self, i: &'ast ItemFn) {
        for attr in &i.attrs {
            if attr
                .to_token_stream()
                .to_string()
                .replace(" ", "")
                .starts_with(&self.searching_for)
            {
                self.out.push(i.sig.ident.to_token_stream().to_string());
            }
        }
    }
}

/// Find locations of `.service(index)` from source code.
pub(crate) fn find_service_registrations(code: &str) -> Result<Location> {
    let mut visitor = MethodCallVisitor::new("service".into());
    if let Ok(syntax_tree) = syn::parse_file(&code) {
        visitor.visit_file(&syntax_tree);
    }
    visitor
        .out
        .pop()
        .ok_or_else(|| anyhow::anyhow!("should be at least one call to .service(...)"))
}

struct MethodCallVisitor {
    searching_for: String,
    out: Vec<Location>,
}

impl MethodCallVisitor {
    fn new(searching_for: String) -> Self {
        MethodCallVisitor {
            searching_for,
            out: Vec::default(),
        }
    }
}

impl<'ast> Visit<'ast> for MethodCallVisitor {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if i.method.to_string() == self.searching_for {
            self.out.push(Location {
                start: i.dot_token.span().start(),
                end: i.span().end(),
            });
        }
        visit_expr_method_call(self, i);
    }
}

use std::fmt;
impl fmt::Debug for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{},{}-{},{}",
            self.start.line, self.start.column, self.end.line, self.end.column
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_handler_attr() {
        assert_eq!(
            format!(
                "{:#?}",
                find_handler_attrs(
                    r#"
use actix_web::{get, middleware::Logger, web, App, HttpServer, Responder};

#[get("/")]
async fn index(query: web::Query<HashMap<String, String>>) -> impl Responder {
    IndexTemplate { query }
}
                    "#
                )
                .unwrap()
            ),
            "4,0-4,11"
        );
    }

    #[test]
    fn test_find_test_attr() {
        assert_eq!(
            format!(
                "{:#?}",
                find_test_attrs(
                    r#"

    #[actix_rt::test]
    async fn test_faviconico() {
        let mut app = atest::init_service(App::new().service(index)).await;

        let req = atest::TestRequest::with_uri("/favicon.ico").to_request();
        let resp = atest::call_service(&mut app, req).await;

        assert!(resp.status().is_success());
    }

                    "#
                )
                .unwrap()
            ),
            "3,4-3,21"
        );
    }

    #[test]
    fn test_find_service_registrations() {
        assert_eq!(
            format!(
                "{:#?}",
                find_service_registrations(
                    r#"


#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    // start http server
    HttpServer::new(move || App::new().wrap(Logger::default()).service(index))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

                    "#
                )
                .unwrap()
            ),
            "10,62-10,77"
        );
    }
}
