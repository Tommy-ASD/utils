use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::ExprIndex;
use syn::{parse_macro_input, visit_mut::VisitMut, Expr};

#[proc_macro_attribute]
pub fn traceback(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let mut function = parse_macro_input!(input as syn::ItemFn);

    let mut visitor = TracingVisitor;
    visitor.visit_item_fn_mut(&mut function);

    TokenStream::from(quote! { #function })
}

struct TracingVisitor;

impl VisitMut for TracingVisitor {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Try(expr_try) => {
                let span = expr_try.question_token.span();
                let inner_expr = &expr_try.expr;
                let new_expr = syn::parse2(quote_spanned! { span=>{
                    match #inner_expr {
                        Ok(val) => Ok(val),
                        Err(e) => Err(traceback!(err e))
                    }
                }?
                })
                .expect("Failed to create traceback match expression");

                *expr = new_expr;
            }
            Expr::Index(index) => {
                // Extract the parts of the index expression
                let ExprIndex {
                    attrs: _,
                    expr: inner_expr,
                    bracket_token: _,
                    index,
                } = index.clone();

                // Create a new expression for safe indexing
                let safe_indexing_expr = quote_spanned!(expr.span() =>
                    match #inner_expr.get(#index) {
                        Some(value) => value,
                        None => {
                            return Err(traceback!(format!("Error while indexing into {} in variable {:?}", #index, #inner_expr)));
                        },
                    }
                );

                // Replace the current expression with the safe indexing expression
                *expr = syn::parse2(safe_indexing_expr).unwrap();
            }
            _ => {
                syn::visit_mut::visit_expr_mut(self, expr);
            }
        }
    }
}
