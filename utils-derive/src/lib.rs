use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, visit_mut::VisitMut, Expr};

#[proc_macro_hack]
pub fn auto_traceback(input: TokenStream) -> TokenStream {
    // Use the input TokenStream to apply #[utils_derive::traceback] to functions.
    // You can implement your logic here to traverse and modify the syntax tree.
    TokenStream::from(input)
}

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
            _ => {
                syn::visit_mut::visit_expr_mut(self, expr);
            }
        }
    }
}
