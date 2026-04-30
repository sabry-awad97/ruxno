//! Code generation for HTML macro

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use rstml::node::{Node, NodeAttribute, NodeElement, NodeName};

/// Generate Rust code from parsed HTML nodes
pub fn generate_html(nodes: &[Node]) -> TokenStream {
    let node_codes: Vec<_> = nodes
        .iter()
        .map(generate_node::<rstml::Infallible>)
        .collect();

    quote! {
        {
            let mut __html = String::new();
            #(#node_codes)*
            __html
        }
    }
}

/// Generate code for a single node
fn generate_node<C>(node: &Node<C>) -> TokenStream
where
    C: ToTokens,
{
    match node {
        Node::Element(element) => generate_element(element),
        Node::Text(text) => {
            let value = text.value_string();
            quote! {
                __html.push_str(#value);
            }
        }
        Node::Block(block) => {
            // Get the block statements
            if let Some(syn_block) = block.try_block() {
                let stmts = &syn_block.stmts;
                quote! {
                    {
                        use std::fmt::Write;
                        let __value = { #(#stmts)* };
                        write!(&mut __html, "{}", __value).unwrap();
                    }
                }
            } else {
                // Invalid block - skip it
                quote! {}
            }
        }
        Node::Fragment(fragment) => {
            let children: Vec<TokenStream> = fragment
                .children
                .iter()
                .map(|n| generate_node::<C>(n))
                .collect();
            quote! {
                #(#children)*
            }
        }
        Node::Doctype(doctype) => {
            let value = &doctype.value;
            quote! {
                __html.push_str("<!DOCTYPE ");
                __html.push_str(#value);
                __html.push_str(">");
            }
        }
        Node::Comment(comment) => {
            let value = comment.value.to_token_stream().to_string();
            quote! {
                __html.push_str("<!--");
                __html.push_str(#value);
                __html.push_str("-->");
            }
        }
        Node::RawText(raw) => {
            let text = raw.to_token_stream_string();
            quote! {
                __html.push_str(#text);
            }
        }
        Node::Custom(_custom) => {
            // Custom nodes are not supported yet
            // Return empty token stream
            quote! {}
        }
    }
}

/// Generate code for an HTML element
fn generate_element<C>(element: &NodeElement<C>) -> TokenStream
where
    C: ToTokens,
{
    let tag_name = get_tag_name(&element.open_tag.name);
    let is_void = is_void_element(&tag_name);

    // Generate opening tag
    let mut opening = quote! {
        __html.push('<');
        __html.push_str(#tag_name);
    };

    // Generate attributes
    for attr in &element.open_tag.attributes {
        let attr_code = generate_attribute(attr);
        opening.extend(attr_code);
    }

    if is_void {
        // Self-closing tag
        opening.extend(quote! {
            __html.push_str(" />");
        });
        return opening;
    }

    opening.extend(quote! {
        __html.push('>');
    });

    // Generate children
    let children: Vec<TokenStream> = element
        .children
        .iter()
        .map(|n| generate_node::<C>(n))
        .collect();

    // Generate closing tag
    let closing = quote! {
        __html.push_str("</");
        __html.push_str(#tag_name);
        __html.push('>');
    };

    quote! {
        #opening
        #(#children)*
        #closing
    }
}

/// Generate code for an attribute
fn generate_attribute(attr: &NodeAttribute) -> TokenStream {
    match attr {
        NodeAttribute::Block(block) => {
            // Get block statements for dynamic attributes
            if let Some(syn_block) = block.try_block() {
                let stmts = &syn_block.stmts;
                quote! {
                    {
                        let __attrs = { #(#stmts)* };
                        // Assume __attrs implements IntoIterator<Item = (String, String)>
                        for (__key, __val) in __attrs {
                            __html.push(' ');
                            __html.push_str(&__key);
                            __html.push_str("=\"");
                            __html.push_str(&__val);
                            __html.push('"');
                        }
                    }
                }
            } else {
                // Invalid block - skip it
                quote! {}
            }
        }
        NodeAttribute::Attribute(attr) => {
            let key = attr.key.to_string();

            // Check if there's a value using the to_value() method
            if let Some(attr_value_expr) = attr.possible_value.to_value() {
                // Extract the actual expression from KVAttributeValue
                match &attr_value_expr.value {
                    rstml::node::KVAttributeValue::Expr(expr) => {
                        // For now, treat all values as dynamic (will be refined in Slice #5)
                        quote! {
                            {
                                __html.push(' ');
                                __html.push_str(#key);
                                __html.push_str("=\"");
                                let __attr_value = #expr;
                                use std::fmt::Write;
                                write!(&mut __html, "{}", __attr_value).unwrap();
                                __html.push('"');
                            }
                        }
                    }
                    rstml::node::KVAttributeValue::InvalidBraced(_) => {
                        // Skip invalid braced expressions
                        quote! {}
                    }
                }
            } else {
                // Boolean attribute (no value)
                quote! {
                    __html.push(' ');
                    __html.push_str(#key);
                }
            }
        }
    }
}

/// Get tag name as string
fn get_tag_name(name: &NodeName) -> String {
    name.to_string()
}

/// Check if element is a void element (self-closing)
fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

/// Helper function to escape HTML entities
/// This will be available at runtime
fn _escape_html_doc() -> TokenStream {
    quote! {
        fn escape_html(s: &str) -> String {
            s.chars()
                .map(|c| match c {
                    '&' => "&amp;".to_string(),
                    '<' => "&lt;".to_string(),
                    '>' => "&gt;".to_string(),
                    '"' => "&quot;".to_string(),
                    '\'' => "&#x27;".to_string(),
                    _ => c.to_string(),
                })
                .collect()
        }
    }
}
