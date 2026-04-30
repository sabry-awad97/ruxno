//! # Ruxno HTML Macro
//!
//! A procedural macro for writing HTML templates in Rust using JSX-like syntax.
//! Built on top of `rstml` for robust HTML parsing.
//!
//! ## Features
//!
//! - JSX-like syntax for HTML templates
//! - Type-safe HTML generation
//! - Automatic escaping of dynamic content
//! - Support for attributes, classes, and inline styles
//! - Conditional rendering with `if` expressions
//! - Iteration with `for` loops
//! - Fragment support with `<></>`
//!
//! ## Example
//!
//! ```rust,ignore
//! use ruxno_html::html;
//!
//! let name = "World";
//! let items = vec!["Apple", "Banana", "Cherry"];
//!
//! let markup = html! {
//!     <html>
//!         <head>
//!             <title>Hello {name}</title>
//!         </head>
//!         <body>
//!             <h1>Hello, {name}!</h1>
//!             <ul>
//!                 {items.iter().map(|item| html! {
//!                     <li>{item}</li>
//!                 }).collect::<Vec<_>>()}
//!             </ul>
//!         </body>
//!     </html>
//! };
//! ```

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod codegen;
mod parser;

/// HTML template macro using JSX-like syntax.
///
/// # Syntax
///
/// ## Elements
///
/// ```rust,ignore
/// html! { <div>Content</div> }
/// html! { <img src="image.jpg" /> }
/// ```
///
/// ## Attributes
///
/// ```rust,ignore
/// html! { <div class="container" id="main">Content</div> }
/// html! { <input type="text" value={some_value} /> }
/// ```
///
/// ## Dynamic Content
///
/// ```rust,ignore
/// let name = "Alice";
/// html! { <p>Hello, {name}!</p> }
/// ```
///
/// ## Conditionals
///
/// ```rust,ignore
/// html! {
///     <div>
///         {if show_message {
///             html! { <p>Message</p> }
///         } else {
///             html! { <p>No message</p> }
///         }}
///     </div>
/// }
/// ```
///
/// ## Iteration
///
/// ```rust,ignore
/// let items = vec!["A", "B", "C"];
/// html! {
///     <ul>
///         {items.iter().map(|item| html! {
///             <li>{item}</li>
///         }).collect::<Vec<_>>()}
///     </ul>
/// }
/// ```
///
/// ## Fragments
///
/// ```rust,ignore
/// html! {
///     <>
///         <h1>Title</h1>
///         <p>Paragraph</p>
///     </>
/// }
/// ```
///
/// ## Raw HTML
///
/// ```rust,ignore
/// html! { <div>{raw("<strong>Bold</strong>")}</div> }
/// ```
#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    let nodes = parse_macro_input!(input as parser::HtmlInput);
    let output = codegen::generate_html(&nodes.nodes);
    TokenStream::from(output)
}
