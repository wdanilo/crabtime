//! <img width="680" alt="banner" src="https://github.com/user-attachments/assets/e7dab624-6e88-41f6-b681-a7a430f96b50">
//!
//! <br/>
//! <br/>
//!
//! # 🦀⏱️ Crabtime
//!
//! **Crabtime** introduces a new way to write Rust macros in a similar spirit to [Zig's comptime](...).
//! It provides even greater flexibility and power than procedural macros, yet it is easy and natural to
//! read and write. Below you can find the comparison of the most important aspects of Rust macro systems:
//!
//! |                                          | Proc Macro | Crabtime | Macro Rules |
//! | :---                                     | :---       | :---     | :---        |
//! | Input as [Token Stream][1]               | ✅         | ✅       | ❌          |
//! | Input as [Macro Fragments][2]            | ❌         | ✅       | ✅          |
//! | Input as Rust Code (String)              | ❌         | ✅       | ❌          |
//! | Output as [Token Stream][1]              | ✅         | ✅       | ❌          |
//! | Output as [Macro Fragments Template][2]  | ❌         | ✅       | ✅          |
//! | Output as Rust Code (String)             | ❌         | ✅       | ❌          |
//! | Advanced transformations                 | ✅         | ✅       | ❌          |
//! | Easy to define (inline, the same crate)  | ❌         | ✅       | ✅          |
//! | Easy to read                             | ❌         | ✅       | ⚠️          |
//! | [Space-aware interpolation](...)         | ❌         | ✅       | ❌          |
//! | Reusable across modules and crates       | ✅         | ✅       | ✅          |
//! | Can define [fn-like macros](...)         | ✅         | ✅       | ✅          |
//! | Can define [derive macros](...)          | ✅         | 🚧       | ❌          |
//! | Can define [attribute macros](...)       | ✅         | 🚧       | ❌          |
//! | [Hygienic](...)                          | ❌         | ❌       | ✅          |
//! | Works with [rustfmt](...)                | ✅         | ✅       | ❌          |
//! | Provides code hints in IDEs              | ✅         | ✅       | ❌          |
//!
//! [1]: https://doc.rust-lang.org/proc_macro/struct.TokenStream.html
//! [2]: https://doc.rust-lang.org/reference/macros-by-example.html#metavariables
//!
//! <br/>
//! <br/>
//!
//! # 🤩 Defining new function-like macros
//!
//! Use the `crabtime::function` attribute to define a new [function-like macro](...). The body of
//! the macro is a regular Rust code, which will be compiled and executed at build time. To output
//! code from the macro, you can use one of the following methods. Let's see how each method works
//! based on a simple example, generating the following Rust code:
//!
//! ```
//! enum Position1 { X }
//! enum Position2 { X, Y }
//! enum Position3 { X, Y, Z }
//! enum Position4 { X, Y, Z, W }
//! ```
//!
//! ### Output String
//!
//! You can simply return a string from the function. It will be pasted as the macro result.
//!
//! ```
//! #[crabtime::function]
//! fn generate() {
//!     let components = ["X", "Y", "Z", "W"];
//!     components.iter().enumerate().map(|(ix, name)| {
//!         let dim = ix + 1;
//!         let cons = components[0..dim].join(",");
//!         format!("enum Position{dim} {{ {cons} }}")
//!     }).collect::<Vec<_>>().join("\n")
//! }
//! generate!();
//! # fn main() {}
//! ```
//!
//! ### Output by `crabtime.output` function
//!
//! Alternatively, you can use the `crabtime.output` function to write strings to the output buffer:
//!
//! ```
//! #[crabtime::function]
//! fn generate() {
//!     let components = ["X", "Y", "Z", "W"];
//!     for (ix, name) in components.iter().enumerate() {
//!         let dim = ix + 1;
//!         let cons = components[0..dim].join(",");
//!         crabtime.output(format!("enum Position{dim} {{ {cons} }}"))
//!     }
//! }
//! generate!();
//! # fn main() {}
//! ```
//!
//! ### Output by the `output!` macro
//!
//! You can also use the `output!` macro, which allows for space-aware variable interpolation. It's like
//! the `format!` macro, but with inversed rules regarding curly braces – it preserves single braces and
//! uses double braces for interpolation. Please note that it preserves spaces, so `Position{{ix}}` can
//! be used to generate `Position1`, `Position2`, etc.
//!
//! ```
//! #[crabtime::function]
//! fn generate() {
//!     let components = ["X", "Y", "Z", "W"];
//!     for (ix, name) in components.iter().enumerate() {
//!         let dim = ix + 1;
//!         let cons = components[0..dim].join(",");
//!         output! {
//!             enum Position{{dim}} {
//!                 {{cons}}
//!             }
//!         }
//!     }
//! }
//! generate!();
//! # fn main() {}
//! ```
//!
//! ### Output `TokenStream`
//!
//! Finally, you can output `TokenStream` from the macro. Please note that for brevity the below example uses
//! [inline dependency injection](...), which is described later.
//!
//! ```
//! #[crabtime::function]
//! fn generate() {
//!     // Inline dependencies used for brevity.
//!     // You should use [build-dependencies] section in your Cargo.toml instead.
//!     #![dependency(proc-macro2 = "1")]
//!     #![dependency(syn = "2")]
//!     #![dependency(quote = "1")]
//!     use proc_macro2::Span;
//!     use quote::quote;
//!
//!     let components = ["X", "Y", "Z", "W"];
//!     let defs = components.iter().enumerate().map(|(ix, name)| {
//!         let dim = ix + 1;
//!         let cons = components[0..dim].iter().map(|t|
//!             syn::Ident::new(t, Span::call_site())
//!         );
//!         let ident = syn::Ident::new(&format!("Position{dim}"), Span::call_site());
//!         quote! {
//!             enum #ident {
//!                 #(#cons),*
//!             }
//!         }
//!     }).collect::<Vec<_>>();
//!     quote! {
//!         #(#defs)*
//!     }
//! }
//! # fn main() {}
//! ```
//!
//!
//! - Return [TokenTree] from the function.
//! - Use `crabtime::output` function.
//! - Use the `output!` macro, which supports double-brace space-sensitive interpolation, allowing
//!   you to embed variables directly into the generated code.
//! -
//!
//!
//! Inside the `eval!`
//! block, you can use the `output!` macro to emit Rust code. `output!` supports double-brace
//! interpolation, allowing you to embed variables directly into the generated code.
//!
//! Example:
//!
//! ```
//! use crabtime::eval;
//!
//! eval! {
//!     let components = ["X", "Y", "Z", "W"];
//!     for (ix, name) in components.iter().enumerate() {
//!
//!         // === Structs Definitions ===
//!         let dim = ix + 1;
//!         let cons = components[0..dim].join(",");
//!         output! {
//!             enum Position{{dim}} {
//!                 {{cons}}
//!             }
//!         }
//!
//!         // === Conversions ===
//!         for ix2 in (dim + 1)..=components.len() {
//!             let source = format!("Position{dim}");
//!             let branches = components[0..dim].iter().map(|comp|
//!                 format!("{source}::{comp} => Self::{comp}")
//!             ).collect::<Vec<_>>().join(",");
//!             output! {
//!                 impl From<{{source}}> for Position{{ix2}} {
//!                     fn from(src: {{source}}) -> Self {
//!                         match src {
//!                             {{branches}}
//!                         }
//!                     }
//!                 }
//!             }
//!         }
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! This will generate:
//!
//! ```
//! enum Position1 { X }
//! enum Position2 { X, Y }
//! enum Position3 { X, Y, Z }
//! enum Position4 { X, Y, Z, W }
//!
//! impl From<Position1> for Position2 {
//!     fn from(src: Position1) -> Self {
//!         match src {
//!             Position1::X => Self::X
//!         }
//!     }
//! }
//! impl From<Position1> for Position3 {
//!     fn from(src: Position1) -> Self {
//!         match src {
//!             Position1::X => Self::X
//!         }
//!     }
//! }
//! impl From<Position1> for Position4 {
//!     fn from(src: Position1) -> Self {
//!         match src {
//!             Position1::X => Self::X
//!         }
//!     }
//! }
//!
//! impl From<Position2> for Position3 {
//!     fn from(src: Position2) -> Self {
//!         match src {
//!             Position2::X => Self::X,
//!             Position2::Y => Self::Y
//!         }
//!     }
//! }
//! impl From<Position2> for Position4 {
//!     fn from(src: Position2) -> Self {
//!         match src {
//!             Position2::X => Self::X,
//!             Position2::Y => Self::Y
//!         }
//!     }
//! }
//!
//! impl From<Position3> for Position4 {
//!     fn from(src: Position3) -> Self {
//!         match src {
//!             Position3::X => Self::X,
//!             Position3::Y => Self::Y,
//!             Position3::Z => Self::Z
//!         }
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! Doing this with `macro_rules!` or procedural macros would be far more complex!
//!
//! Also, please note that you can use the `eval!` macro to perform complex type-level
//! calculations:
//!
//! ```
//! use crabtime::eval;
//!
//! const MY_NUM: usize = eval! { (std::f32::consts::PI.sqrt() * 10.0).round() as usize };
//! # fn main() {}
//! ```
//!
//! <br/>
//! <br/>
//!
//! # 🪲 Logging and Output Protocol
//!
//! During compilation, `eval!` blocks can print messages directly to `stdout` and `stderr`.
//! This allows you to emit debug information, diagnostics, and the generated code itself during
//! macro evaluation. Communication between the `eval!` macro and the build system follows a
//! simple line-based protocol. Each printed line can optionally begin with a prefix,
//! indicating its purpose:
//!
//! | Prefix     | Meaning |
//! | :---       | :---    |
//! | _(none)_   | Debug log message (informational output). |
//! | `OUTPUT:`  | A line of generated Rust code to be included in the final macro output. |
//! | `WARNING:` | A compilation warning. This is printed to `stdout` until [Procedural Macro Diagnostics][3] is stabilized. |
//! | `ERROR:`   | A compilation error. This is printed to `stdout` until [Procedural Macro Diagnostics][3] is stabilized. |
//!
//! ### Utility Functions and Macros
//!
//! To simplify working with this protocol, `eval!` blocks have access to a set of helper
//! functions and macros that automatically apply the correct prefixes to each line.
//!
//! #### Functions
//!
//! These functions allow you to transform multi-line strings by adding the appropriate prefixes:
//!
//! ```rust
//! fn prefix_lines_with(prefix: &str, input: &str) -> String {
//!     // Adds the given prefix to each line of the input string.
//!     # panic!()
//! }
//!
//! fn prefix_lines_with_output(input: &str) -> String {
//!     // Adds `OUTPUT:` to each line of the input string.
//!     # panic!()
//! }
//!
//! fn prefix_lines_with_warning(input: &str) -> String {
//!     // Adds `WARNING:` to each line of the input string.
//!     # panic!()
//! }
//!
//! fn prefix_lines_with_error(input: &str) -> String {
//!     // Adds `ERROR:` to each line of the input string.
//!     # panic!()
//! }
//! ```
//!
//! #### Macros
//!
//! These macros allow you to directly print prefixed lines to `stdout`, following the
//! protocol:
//!
//! ```rust
//! macro_rules! println_output {
//!     // Prints a line prefixed with `OUTPUT:`.
//!     # () => {};
//! }
//!
//! macro_rules! println_warning {
//!     // Prints a line prefixed with `WARNING:`.
//!     # () => {};
//! }
//!
//! macro_rules! println_error {
//!     // Prints a line prefixed with `ERROR:`.
//!     # () => {};
//! }
//! ```
//!
//! These tools ensure consistent formatting and correct communication between `eval!` blocks
//! and the build system, reducing the risk of malformed output.
//!
//! [3]: https://github.com/rust-lang/rust/issues/54140
//!
//! <br/>
//! <br/>
//!
//! # 📚 Attributes
//!
//! The `eval!` macro supports global attributes that can be placed at the top of the block.
//! These attributes allow you to customize both the project's Cargo configuration and its
//! project-wide attributes.
//!
//! ### Supported Cargo Configuration Attributes
//!
//! | Attribute            | Default |
//! | :---                  | :---    |
//! | `#![edition(...)]`   | `2024`  |
//! | `#![resolver(...)]`  | `3`     |
//! | `#![dependency(...)]`| `[]`    |
//!
//! ### Supported Standard Attributes
//!
//! In addition to Cargo settings, the following standard Rust attributes are supported:
//!
//! - `#![feature(...)]`
//! - `#![allow(...)]`
//! - `#![expect(...)]`
//! - `#![warn(...)]`
//! - `#![deny(...)]`
//! - `#![forbid(...)]`
//!
//! Example:
//!
//! ```rust
//! use crabtime::eval;
//!
//! eval! {
//!     #![edition(2024)]
//!     #![resolver(3)]
//!     #![dependency(anyhow = "1.0")]
//!
//!     type Result<T> = anyhow::Result<T>;
//!     // ...
//! }
//! # fn main() {}
//! ```
//!
//! This system allows each `eval!` macro block to define its own dependencies and configuration
//! without affecting your project's main `Cargo.toml` or global settings.
//!
//! <br/>
//! <br/>
//!
//! # 🧱 Working with Token Streams
//!
//! If you prefer to work directly with token streams instead of plain Rust code, you can
//! leverage the `proc-macro2` crate to parse source code into a `TokenStream` and then
//! generate output using the `quote` crate.
//!
//! This allows you to process and manipulate Rust code programmatically within an `eval!` block,
//! similar to how procedural macros operate — but with the flexibility of the `eval!` environment.
//!
//! ```
//! use crabtime::eval;
//!
//! eval! {
//!     #![dependency(proc-macro2 = "1")]
//!     #![dependency(quote = "1")]
//!     use proc_macro2::TokenStream;
//!     use quote::quote;
//!     let tokens: TokenStream = SOURCE_CODE.parse().unwrap();
//!     // ...
//!     let out = quote! {
//!         pub struct Test {}
//!     };
//!     println_output!("{}", out.to_string());
//! }
//!
//! type Alias = Test;
//!
//! # fn main() {}
//! ```
//!
//! <br/>
//! <br/>
//!
//! # 📖 How It Works Under The Hood
//!
//! The content inside `eval!` is pasted into the `main` function of a temporary Rust project
//! created in `$HOME/.cargo/eval-macro/<project-id>`. This project is created, compiled,
//! executed, and removed at build time, and its `stdout` becomes the generated Rust code. The
//! generated `main` function looks something like this:
//!
//! ```
//! const SOURCE_CODE: &str = "..."; // Your code as a string.
//!
//! fn main() {
//!     let mut output_buffer = String::new();
//!     let result = {{
//!         // Your code.
//!     }};
//!     push_as_str(&mut output_buffer, &result);
//!     println!("{}", prefix_lines_with_output(&output_buffer));
//! }
//! # fn push_as_str(str: &mut String, result: &()) {}
//! # fn prefix_lines_with_output(input: &str) -> String { String::new() }
//! ```
//!
//! The `output!` macro is essentially a shortcut for writing to `output_buffer` using `format!`,
//! so this:
//!
//! ```
//! use crabtime::eval;
//!
//! eval! {
//!     let components = ["X", "Y", "Z", "W"];
//!     for (ix, name) in components.iter().enumerate() {
//!         let dim = ix + 1;
//!         let cons = components[0..dim].join(",");
//!         output! {
//!             enum Position{{dim}} {
//!                 {{cons}}
//!             }
//!         }
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! Is equivalent to:
//!
//! ```
//! use crabtime::eval;
//!
//! eval! {
//!     let components = ["X", "Y", "Z", "W"];
//!     for (ix, name) in components.iter().enumerate() {
//!         let dim = ix + 1;
//!         let cons = components[0..dim].join(",");
//!         // The `write_ln!` macro is delivered by this library.
//!         write_ln!(output_buffer, "
//!             enum Position{dim} {{
//!                 {cons}
//!             }}
//!         ");
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! And that, in turn, is just shorthand for:
//!
//! ```
//! use crabtime::eval;
//!
//! eval! {
//!     let components = ["X", "Y", "Z", "W"];
//!     for (ix, name) in components.iter().enumerate() {
//!         let dim = ix + 1;
//!         let cons = components[0..dim].join(",");
//!         output_buffer.push_str(
//!             &format!("
//!                 enum Position{dim} {{
//!                     {cons}
//!                 }}
//!             ")
//!         );
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! Which, ultimately, is equivalent to:
//!
//! ```
//! use crabtime::eval;
//!
//! eval! {
//!     let components = ["X", "Y", "Z", "W"];
//!     for (ix, name) in components.iter().enumerate() {
//!         let dim = ix + 1;
//!         let cons = components[0..dim].join(",");
//!         println!("OUTPUT: enum Position{dim} {{");
//!         println!("OUTPUT:     {cons}");
//!         println!("OUTPUT: }}");
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! <br/>
//! <br/>
//!
//! # ⚠️ Troubleshooting
//!
//! ⚠️ **Note:** Rust IDEs differ in how they handle macro expansion. This macro is tuned for
//! `RustRover’s` expansion engine.
//!
//! If your IDE struggles to correctly expand `eval!`, you can manually switch to the `write_ln!`
//! syntax described above. If you encounter issues, please
//! [open an issue](https://github.com/wdanilo/eval-macro/issues) to let us know!

pub use crabtime_internal::*;
