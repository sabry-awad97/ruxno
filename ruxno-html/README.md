# Ruxno HTML

A procedural macro for writing HTML templates in Rust using JSX-like syntax, built on top of `rstml`.

## Features

- 🎨 JSX-like syntax for HTML templates
- 🔒 Type-safe HTML generation
- 🛡️ Automatic escaping of dynamic content
- ⚡ Zero-runtime overhead (compile-time generation)
- 🎯 Support for attributes, classes, and inline styles
- 🔀 Conditional rendering with `if` expressions
- 🔁 Iteration with `for` loops
- 📦 Fragment support with `<></>`

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ruxno-html = { path = "../ruxno-html" }
```

## Usage

### Basic Example

```rust
use ruxno_html::html;

let name = "World";
let markup = html! {
    <html>
        <head>
            <title>Hello {name}</title>
        </head>
        <body>
            <h1>Hello, {name}!</h1>
        </body>
    </html>
};

println!("{}", markup);
```

### Attributes

```rust
let class_name = "container";
let id = "main";

html! {
    <div class={class_name} id={id}>
        <p>Content</p>
    </div>
}
```

### Conditional Rendering

```rust
let show_message = true;

html! {
    <div>
        {if show_message {
            html! { <p>Message is visible</p> }
        } else {
            html! { <p>Message is hidden</p> }
        }}
    </div>
}
```

### Iteration

```rust
let items = vec!["Apple", "Banana", "Cherry"];

html! {
    <ul>
        {items.iter().map(|item| html! {
            <li>{item}</li>
        }).collect::<Vec<_>>()}
    </ul>
}
```

### Fragments

```rust
html! {
    <>
        <h1>Title</h1>
        <p>Paragraph</p>
    </>
}
```

### Self-Closing Tags

```rust
html! {
    <div>
        <img src="image.jpg" alt="Description" />
        <br />
        <input type="text" />
    </div>
}
```

## Comparison with Maud

| Feature        | Ruxno HTML             | Maud          |
| -------------- | ---------------------- | ------------- |
| Syntax         | JSX-like               | Custom DSL    |
| Learning Curve | Familiar to React devs | Rust-specific |
| Escaping       | Automatic              | Automatic     |
| Type Safety    | ✅                     | ✅            |
| Performance    | Compile-time           | Compile-time  |

## Safety

All dynamic content is automatically escaped to prevent XSS attacks. If you need to render raw HTML, use the `raw()` function (to be implemented).

## License

MIT OR Apache-2.0
