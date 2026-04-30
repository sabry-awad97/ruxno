//! Parser for HTML macro input using rstml

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};

/// Parsed HTML input
pub struct HtmlInput {
    pub nodes: Vec<rstml::node::Node>,
}

impl Parse for HtmlInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Convert ParseStream to TokenStream for rstml
        let tokens: TokenStream = input.parse()?;

        // Parse as rstml nodes
        let parser = rstml::Parser::new(rstml::ParserConfig::default());
        let nodes = parser.parse_simple(tokens)?;

        Ok(HtmlInput { nodes })
    }
}

impl ToTokens for HtmlInput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for node in &self.nodes {
            node.to_tokens(tokens);
        }
    }
}
