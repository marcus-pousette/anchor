use crate::Program;
use heck::SnakeCase;
use quote::{quote, ToTokens};

/// Build `crate::<parent_path>::__client_accounts_<snake>`
/// from a path like `ix::deposit_usd_into_pool::DepositUsdIntoPool`.
fn helper_path_for_client_accounts(
    anchor_ident: &proc_macro2::TokenStream,
    snake: &str,
) -> proc_macro2::TokenStream {
    let full = anchor_ident.to_string();
    let mut parts: Vec<&str> = full
        .split("::")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    // last segment is the Accounts type name
    let _ = parts.pop();

    let helper = format!("__client_accounts_{}", snake);
    if parts.is_empty() {
        // Accounts at crate root
        format!("crate::{helper}").parse().unwrap()
    } else {
        // Accounts inside nested module(s)
        format!("crate::{}::{helper}", parts.join("::")).parse().unwrap()
    }
}

pub fn generate(program: &Program) -> proc_macro2::TokenStream {
    let account_structs: Vec<proc_macro2::TokenStream> = program
        .ixs
        .iter()
        .map(|ix| {
            let snake = ix.anchor_ident.to_string().to_snake_case();
            let helper_path =
                helper_path_for_client_accounts(&ix.anchor_ident.to_token_stream(), &snake);
            let cfgs = &ix.cfgs;

            quote! {
                #(#cfgs)*
                pub use #helper_path::*;
            }
        })
        .collect();

    quote! {
        /// An Anchor generated module, providing a set of structs
        /// mirroring the structs deriving `Accounts`, where each field is
        /// a `Pubkey`. This is useful for specifying accounts for a client.
        pub mod accounts {
            #(#account_structs)*
        }
    }
}