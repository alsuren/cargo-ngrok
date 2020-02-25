//! Parsing Rust source code.
//! Original code is lifted from cargo-fixeq, and then adapted
//! for my purposes.

use proc_macro2::LineColumn;
use syn::{spanned::Spanned, visit::Visit, Attribute, Ident};

/// Find locations of `#[get("/")]`s from source code.
pub(crate) fn find_route_attrs(code: &str) -> Vec<Location> {
    let mut visitor = RouteAttrVisitor::default();
    if let Ok(syntax_tree) = syn::parse_file(&code) {
        visitor.visit_file(&syntax_tree);
    }
    visitor.out
}

#[derive(Clone)]
pub(crate) struct Location {
    pub(crate) start: LineColumn,
    pub(crate) end: LineColumn,
}

#[derive(Default)]
struct RouteAttrVisitor {
    out: Vec<Location>,
}

impl<'ast> Visit<'ast> for RouteAttrVisitor {
    fn visit_attribute(&mut self, i: &'ast Attribute) {
        let path = &i.path;
        if path.is_ident(&Ident::new("get", path.span())) {
            i.pound_token.span().start();
            self.out.push(Location {
                start: i.pound_token.span().start(),
                end: i.bracket_token.span.end(),
            });
        }
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
    fn test_find_route_attr() {
        assert_eq!(
            format!(
                "{:#?}",
                find_route_attrs(
                    r#"
use actix_web::{get, middleware::Logger, web, App, HttpServer, Responder};

#[get("/")]
async fn index(query: web::Query<HashMap<String, String>>) -> impl Responder {
    IndexTemplate { query }
}
                    "#
                )
            )
            .replace("\n", "")
            .replace(" ", "")
            .replace(",]", "]"),
            "[4,0-4,11]"
        );
    }
}
