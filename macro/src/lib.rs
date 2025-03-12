#![cfg_attr(nightly, feature(proc_macro_span))]
#![cfg_attr(nightly, feature(proc_macro_diagnostic))]

#![cfg_attr(not(nightly), allow(dead_code))]
#![cfg_attr(not(nightly), allow(unused_macros))]
#![cfg_attr(not(nightly), allow(unused_imports))]

mod error;

use std::fmt::Debug;
use proc_macro2::Delimiter;
use proc_macro2::LineColumn;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use quote::ToTokens;
use quote::quote;
use std::fs::File;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::default::Default;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use error::*;

// =================
// === Constants ===
// =================

/// Set to 'true' to enable debug prints.
const DEBUG: bool = false;

const CRATE: &str = "crabtime";
const DEFAULT_EDITION: &str = "2024";
const DEFAULT_RESOLVER: &str = "3";
const GEN_MOD: &str = CRATE;
const OUTPUT_PREFIX: &str = "[OUTPUT]";
const OUT_DIR: &str = env!("OUT_DIR");

// ==================
// === TokenRange ===
// ==================

#[derive(Debug)]
struct TokenRange {
    start: TokenTree,
    end: TokenTree,
}

impl TokenRange {
    fn new(start: TokenTree, end: TokenTree) -> Self {
        Self { start, end }
    }

    #[cfg(nightly)]
    fn span(&self) -> Span {
        let first_span = self.start.span();
        let last_span = self.end.span();
        first_span.join(last_span).unwrap_or(first_span)
    }
}

// ==============================
// === Generated Code Prelude ===
// ==============================

fn gen_prelude(include_token_stream_impl: bool) -> String {
    let warning_prefix = Level::WARNING_PREFIX;
    let error_prefix = Level::ERROR_PREFIX;
    let prelude_tok_stream = if include_token_stream_impl { PRELUDE_FOR_TOKEN_STREAM } else { "" };
    format!("
        #[allow(unused_macros)]
        #[allow(unused_imports)]
        #[allow(clippy)]
        mod {GEN_MOD} {{
            #![allow(clippy::all)]

            const OUTPUT_PREFIX: &'static str = \"{OUTPUT_PREFIX}\";
            const WARNING_PREFIX: &'static str = \"{warning_prefix}\";
            const ERROR_PREFIX: &'static str = \"{error_prefix}\";

            macro_rules! output_str {{
                ($($ts:tt)*) => {{
                    println!(\"{{}}\", {GEN_MOD}::prefix_lines_with_output(&format!($($ts)*)));
                }};
            }}
            pub(super) use output_str;

            macro_rules! warning {{
                ($($ts:tt)*) => {{
                    println!(\"{{}}\", {GEN_MOD}::prefix_lines_with_warning(&format!($($ts)*)));
                }};
            }}
            pub(super) use warning;

            macro_rules! error {{
                ($($ts:tt)*) => {{
                    println!(\"{{}}\", {GEN_MOD}::prefix_lines_with_error(&format!($($ts)*)));
                }};
            }}
            pub(super) use error;

            {PRELUDE_STATIC}
            {prelude_tok_stream}
        }}

        {PRELUDE_MAGIC}
    ")
}

const PRELUDE_FOR_TOKEN_STREAM: &str = "
    impl CodeFromOutput for proc_macro2::TokenStream {
        fn code_from_output(output: Self) -> String {
            output.to_string()
        }
    }
";

const PRELUDE_STATIC: &str = "
    pub(super) trait CodeFromOutput {
        fn code_from_output(output: Self) -> String;
    }

    impl CodeFromOutput for () {
        fn code_from_output(_output: Self) -> String {
            String::new()
        }
    }

    impl<'t> CodeFromOutput for &'t str {
        fn code_from_output(output: Self) -> String {
            output.to_string()
        }
    }

    impl CodeFromOutput for String {
        fn code_from_output(output: Self) -> String {
            output
        }
    }

    impl CodeFromOutput for usize {
        fn code_from_output(output: Self) -> String {
            format!(\"{output}\")
        }
    }

    impl CodeFromOutput for u8 {
        fn code_from_output(output: Self) -> String {
            format!(\"{output}\")
        }
    }

    impl CodeFromOutput for u16 {
        fn code_from_output(output: Self) -> String {
            format!(\"{output}\")
        }
    }

    impl CodeFromOutput for u32 {
        fn code_from_output(output: Self) -> String {
            format!(\"{output}\")
        }
    }

    impl CodeFromOutput for u64 {
        fn code_from_output(output: Self) -> String {
            format!(\"{output}\")
        }
    }

    impl CodeFromOutput for u128 {
        fn code_from_output(output: Self) -> String {
            format!(\"{output}\")
        }
    }

    impl CodeFromOutput for f32 {
        fn code_from_output(output: Self) -> String {
            format!(\"{output}\")
        }
    }

    impl CodeFromOutput for f64 {
        fn code_from_output(output: Self) -> String {
            format!(\"{output}\")
        }
    }

    pub(super) fn code_from_output<T: CodeFromOutput>(output: T) -> String {
        <T as CodeFromOutput>::code_from_output(output)
    }

    pub(super) fn prefix_lines_with(prefix: &str, input: &str) -> String {
        input
            .lines()
            .map(|line| format!(\"{prefix} {line}\"))
            .collect::<Vec<_>>()
            .join(\"\\n\")
    }

    pub(super) fn prefix_lines_with_output(input: &str) -> String {
        prefix_lines_with(OUTPUT_PREFIX, input)
    }

    pub(super) fn prefix_lines_with_warning(input: &str) -> String {
        prefix_lines_with(WARNING_PREFIX, input)
    }

    pub(super) fn prefix_lines_with_error(input: &str) -> String {
        prefix_lines_with(ERROR_PREFIX, input)
    }

    macro_rules! write_ln {
        ($target:expr, $($ts:tt)*) => {
            $target.push_str(&format!( $($ts)* ));
            $target.push_str(\"\n\");
        };
    }
    pub(super) use write_ln;

    macro_rules! stringify_if_needed {
        ($t:literal) => { $t };
        ($t:expr) => { stringify!($t) };
    }
    pub(super) use stringify_if_needed;
";

/// To be removed one day.
const PRELUDE_MAGIC: &str = "
    #[allow(clippy)]
    fn sum_combinations(n: usize) -> Vec<Vec<usize>> {
        let mut result = Vec::new();

        fn generate(n: usize, current: Vec<usize>, result: &mut Vec<Vec<usize>>) {
            if n == 0 {
                if current.len() > 1 {
                    result.push(current);
                }
                return;
            }

            for i in 1..=n {
                let mut next = current.clone();
                next.push(i);
                generate(n - i, next, result);
            }
        }

        generate(n, vec![], &mut result);
        result
    }
";

// =============
// === Paths ===
// =============

#[derive(Debug)]
struct Paths {
    output_dir: PathBuf,
    /// None if we are on stable.
    cargo_toml_path: Option<CargoConfigPaths>,
}

fn parent_dir(path: &Path) -> Result<&Path> {
    path.parent().context(|| error!("Path '{}' does not have a parent.", path.display()))
}

fn find_parent_dir<'t>(path: &'t Path, dir_name: &str) -> Result<&'t Path> {
    let dir_name_os = std::ffi::OsStr::new(dir_name);
    path.ancestors()
        .find(|p| p.file_name() == Some(dir_name_os))
        .context(|| error!(
            "Path '{}' does not have parent '{dir_name}' directory.",
            path.display()
        ))
}

impl Paths {
    #[cfg(nightly)]
    fn new(options: MacroOptions, macro_name: &str, input_str: &str) -> Result<Self> {
        let name = if options.content_base_name {
            project_name_from_input(input_str)
        } else {
            macro_name.to_string()
        };

        let mut call_site_path = proc_macro::Span::call_site().source_file().path();
        call_site_path.set_extension("");

        let output_dir = Self::get_output_root()?.join(&call_site_path).join(&name);
        let crate_out_str = OUT_DIR;
        let crate_out = Path::new(&crate_out_str);
        let target = find_parent_dir(crate_out, "target")?;
        let workspace = parent_dir(target)?;
        let file_path = workspace.join(&call_site_path);
        let cargo_toml_path = Some(find_cargo_configs(&file_path)?);
        Ok(Self { output_dir, cargo_toml_path })
    }

    #[cfg(not(nightly))]
    fn new(_options: MacroOptions, _macro_name: &str, input_str: &str) -> Result<Self> {
        let name = project_name_from_input(input_str);
        let output_dir = Self::get_output_root()?.join(CRATE).join(&name);
        let cargo_toml_path = None;
        Ok(Self { output_dir, cargo_toml_path })
    }

    fn get_output_root() -> Result<PathBuf> {
        let crate_out_str = OUT_DIR;
        let crate_out = Path::new(&crate_out_str);
        let build_dir = find_parent_dir(crate_out, "build")?;
        Ok(build_dir.join(CRATE))
    }

    fn with_output_dir<T>(&self, cache: bool, f: impl FnOnce(&PathBuf) -> Result<T>) -> Result<T> {
        if !self.output_dir.exists() {
            fs::create_dir_all(&self.output_dir).context("Failed to create project directory.")?;
        }
        let out = f(&self.output_dir);
        // We cache projects on nightly. On stable, the project name is based on the input code.
        if cfg!(not(nightly)) || !cache {
            fs::remove_dir_all(&self.output_dir).ok();
        }
        out
    }
}

fn project_name_from_input(input_str: &str) -> String {
    let mut hasher = DefaultHasher::new();
    input_str.hash(&mut hasher);
    format!("project_{:016x}", hasher.finish())
}

// ========================
// === CargoConfigPaths ===
// ========================

#[derive(Debug)]
struct CargoConfigPaths {
    crate_config: PathBuf,
    _workspace_config: Option<PathBuf>,
}

fn find_cargo_configs(path: &Path) -> Result<CargoConfigPaths> {
    let mut current_path = path.to_path_buf();
    let mut out = Vec::new();
    loop {
        let candidate = current_path.join("Cargo.toml");
        if candidate.is_file() { out.push(candidate) }
        if !current_path.pop() { break }
    }
    if out.len() >= 2 {
        Ok(CargoConfigPaths {
            crate_config: out[0].clone(),
            _workspace_config: Some(out[1].clone()),
        })
    } else if !out.is_empty() {
        Ok(CargoConfigPaths {
            crate_config: out[0].clone(),
            _workspace_config: None,
        })
    } else {
        err!("No 'Cargo.toml' files found in parent directories of '{}'.", path.display())
    }
}

// ===================
// === CargoConfig ===
// ===================

#[derive(Debug)]
struct Dependency {
    label: String,
    tokens_str: String,
    token_range: Option<TokenRange>,
}

impl Dependency {
    fn new(label: String, tokens_str: String, token_range: Option<TokenRange>) -> Self {
        Self { label, tokens_str, token_range }
    }

    #[cfg(nightly)]
    fn span(&self) -> Span {
        self.token_range.as_ref().map_or(Span::call_site(), |t| t.span())
    }
}

#[derive(Debug, Default)]
struct CargoConfig {
    edition: Option<String>,
    resolver: Option<String>,
    dependencies: Vec<Dependency>,
}

impl CargoConfig {
    fn contains_dependency(&self, name: &str) -> bool {
        self.dependencies.iter().any(|d| d.label == name)
    }

    fn print(&self) -> String {
        let edition = self.edition.as_ref().map_or(DEFAULT_EDITION, |t| t.as_str());
        let resolver = self.resolver.as_ref().map_or(DEFAULT_RESOLVER, |t| t.as_str());
        let dependencies = self.dependencies.iter()
            .map(|t| format!("{} = {}", t.label.clone(), t.tokens_str.clone())) // FIXME: move to dependency method
            .collect::<Vec<_>>()
            .join("\n");
        format!("
            [workspace]
            [package]
            name     = \"eval_project\"
            version  = \"1.0.0\"
            edition  = \"{edition}\"
            resolver = \"{resolver}\"

            [dependencies]
            {dependencies}
        ")
    }

    fn fill_from_cargo_toml(&mut self, cargo_config_paths: &CargoConfigPaths) -> Result {
        let cargo_toml_content = fs::read_to_string(&cargo_config_paths.crate_config)?;
        let parsed: toml::Value = toml::from_str(&cargo_toml_content)?;
        let dependencies = parsed
            .get("build-dependencies")
            .and_then(|v| v.as_table())
            .map_or(vec![], |t| t.iter().map(|(k, v)| Dependency::new(k.clone(), format!("{v}"), None)).collect());
        let edition = parsed
            .get("package")
            .and_then(|v| v.as_table())
            .and_then(|table| table.get("edition"))
            .and_then(|v| v.as_str())
            .unwrap_or("2024");
        self.dependencies.extend(dependencies);
        self.edition = Some(edition.to_string());
        Ok(())
    }

    fn extract_inline_attributes(&mut self, attributes: Vec<syn::Attribute>) -> Result<String> {
        let mut other_attributes = Vec::with_capacity(attributes.len());
        let mut new_dependencies: Vec<Dependency> = vec![]; // FIXME expl typing not needed
        for attr in attributes {
            let tokens = attr.parse_args::<TokenStream>().context("Failed to parse attributes")?;
            let tokens_str = tokens.to_string().replace(" ", "");
            let token_range = tokens.clone().into_iter().next()
                .zip(tokens.clone().into_iter().last())
                .map(|(first, last)| TokenRange::new(first, last));
            if attr.path().is_ident("dependency") {
                let (key, value) = tokens_str.split_once('=').context(||
                    error!("Incorrect dependency '{tokens_str}'")
                )?;
                new_dependencies.push(Dependency::new(key.to_string(), value.to_string(), token_range)); // FIXME
            } else if attr.path().is_ident("edition") {
                self.edition = Some(tokens_str);
            } else {
                other_attributes.push(attr.to_token_stream().to_string());
            }
        }
        #[cfg(nightly)]
        for dependency in &new_dependencies {
            warning!(dependency.span(),
                "When using the nightly Rust channel, dependencies should be specified in the \
                [build-dependencies] section of your Cargo.toml file."
            ).emit();
        }
        self.dependencies.extend(new_dependencies);
        Ok(other_attributes.join("\n"))
    }
}

fn create_project_skeleton(project_dir: &Path, cfg: CargoConfig, main: &str) -> Result<bool> {
    let src_dir = project_dir.join("src");
    let existed = src_dir.exists();
    if !existed {
        fs::create_dir_all(&src_dir).context("Failed to create src directory.")?;
    }

    let cargo_toml = project_dir.join("Cargo.toml");
    let cargo_toml_content = cfg.print();
    fs::write(&cargo_toml, cargo_toml_content).context("Failed to write Cargo.toml.")?;

    let main_rs = src_dir.join("main.rs");
    let mut file = File::create(&main_rs).context("Failed to create main.rs")?;
    file.write_all(main.as_bytes()).context("Failed to write main.rs")?;
    Ok(existed)
}

fn get_host_target() -> Result<String> {
    let output = Command::new("rustc")
        .arg("-vV")
        .stdout(std::process::Stdio::piped())
        .output()
        .context("Failed to run rustc")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(stripped) = line.strip_prefix("host:") {
            return Ok(stripped.trim().to_string())
        }
    }
    err!("Could not determine host target from rustc")
}

fn run_cargo_project(project_dir: &PathBuf) -> Result<String> {
    // In case the project uses .cargo/config.toml, we need to explicitly revert target to native.
    let host_target = get_host_target()?;
    let output = Command::new("cargo")
        .arg("run")
        .arg("--target")
        .arg(&host_target)
        .current_dir(project_dir)
        .output()
        .context("Failed to execute cargo run")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // TODO: Parse it and map gen code spans to call site spans.
        eprintln!("{stderr}");
        #[allow(clippy::panic)]
        if let Some(index) = stderr.find("thread 'main' panicked") {
            panic!("{}", &stderr[index..]);
        }
        err!("Compilation of the generated code failed.")
    } else {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

// ====================
// === Output Macro ===
// ====================

/// Find and expand the `output!` macro in the input `TokenStream`. After this lib stabilizes, this
/// should be rewritten to standard macro and imported by the generated code.
fn expand_expand_macro(input: TokenStream) -> TokenStream {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    let mut output = TokenStream::new();
    let mut i = 0;
    while i < tokens.len() {
        if let TokenTree::Ident(ref ident) = tokens[i] {
            if *ident == "expand" && i + 1 < tokens.len() {
                if let TokenTree::Punct(ref excl) = tokens[i + 1] {
                    if excl.as_char() == '!' && i + 2 < tokens.len() {
                        if let TokenTree::Group(ref group) = tokens[i + 2] {
                            output.extend(group.stream());
                            i += 3;
                            continue;
                        }
                    }
                }
            }
        }
        match &tokens[i] {
            TokenTree::Group(group) => {
                let new_stream = expand_expand_macro(group.stream());
                let new_group = TokenTree::Group(proc_macro2::Group::new(group.delimiter(), new_stream));
                output.extend(std::iter::once(new_group));
            }
            _ => {
                output.extend(std::iter::once(tokens[i].clone()));
            }
        }
        i += 1;
    }
    output
}

fn expand_builtin_macro(name: &str, input: TokenStream, f: &impl Fn(TokenStream) -> TokenStream) -> TokenStream {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    let mut output = TokenStream::new();
    let len = tokens.len();
    let mut i = 0;

    while i < len {
        // Check for the pattern: crabtime :: output ! ( group )
        if i + 5 < len {
            if let TokenTree::Ident(ref ident) = tokens[i] {
                if ident == GEN_MOD {
                    if let TokenTree::Punct(ref colon1) = tokens[i + 1] {
                        if colon1.as_char() == ':' {
                            if let TokenTree::Punct(ref colon2) = tokens[i + 2] {
                                if colon2.as_char() == ':' {
                                    if let TokenTree::Ident(ref out_ident) = tokens[i + 3] {
                                        if out_ident == name {
                                            if let TokenTree::Punct(ref excl) = tokens[i + 4] {
                                                if excl.as_char() == '!' {
                                                    if let TokenTree::Group(ref group) = tokens[i + 5] {
                                                        let inner_rewritten = expand_builtin_macro(name, group.stream(), f);
                                                        let new_tokens = f(inner_rewritten);
                                                        output.extend(new_tokens);
                                                        i += 6;
                                                        continue;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Recurse into groups or pass through token.
        match &tokens[i] {
            TokenTree::Group(group) => {
                let new_stream = expand_builtin_macro(name, group.stream(), f);
                // Rebuild group with same delimiter.
                let new_group = TokenTree::Group(proc_macro2::Group::new(group.delimiter(), new_stream));
                output.extend(std::iter::once(new_group));
            }
            token => output.extend(std::iter::once(token.clone())),
        }
        i += 1;
    }
    output
}

fn expand_output_macro(input: TokenStream) -> TokenStream {
    let gen_mod = syn::Ident::new(GEN_MOD, Span::call_site());
    expand_builtin_macro("output", input, &|inner_rewritten| {
        let content_str = print_tokens(&inner_rewritten);
        let lit = syn::LitStr::new(&content_str, Span::call_site());
        quote! {
            #gen_mod::write_ln!(__output_buffer__, #lit);
        }
    })
}

fn expand_quote_macro(input: TokenStream) -> TokenStream {
    expand_builtin_macro("quote", input, &|inner_rewritten| {
        let content_str = print_tokens(&inner_rewritten);
        let lit = syn::LitStr::new(&content_str, Span::call_site());
        quote! { format!(#lit) }
    })
}

// =============
// === Print ===
// =============

#[derive(Debug)]
struct PrintOutput {
    output: String,
    start_token: Option<LineColumn>,
    end_token: Option<LineColumn>,
}

/// Prints the token stream as a string ready to be used by the format macro. It is very careful
/// where spaces are inserted. In particular, spaces are not inserted around `{` and `}` tokens if
/// they were not present in the original token stream. It is fine-tuned to work in different IDEs,
/// such as `RustRover`.
fn print_tokens(tokens: &TokenStream) -> String {
    // Replaces `{` with `{{` and vice versa.
    print_tokens_internal(tokens).output
        .replace("{", "{{")
        .replace("}", "}}")
        .replace("{{{{", "{")
        .replace("}}}}", "}")
}

fn print_tokens_internal(tokens: &TokenStream) -> PrintOutput {
    let token_vec: Vec<TokenTree> = tokens.clone().into_iter().collect();
    let mut output = String::new();
    let mut first_token_start = None;
    let mut prev_token_end: Option<LineColumn> = None;
    let mut prev_token_was_brace = false;
    for (i, token) in token_vec.iter().enumerate() {
        let mut add_space = true;
        let mut token_start = token.span().start();
        let mut token_end = token.span().end();
        let mut is_brace = false;
        let token_str = match token {
            TokenTree::Group(g) => {
                let content = print_tokens_internal(&g.stream());
                let mut content_str = content.output;
                content_str.pop();
                let (open, close) = match g.delimiter() {
                    Delimiter::Brace =>{
                        is_brace = true;
                        ("{", "}")
                    },
                    Delimiter::Parenthesis => ("(", ")"),
                    Delimiter::Bracket => ("[", "]"),
                    _ => ("", ""),
                };

                if let Some(content_first_token_start) = content.start_token {
                    token_start.line = content_first_token_start.line;
                    if content_first_token_start.column > 0 {
                        token_start.column = content_first_token_start.column - 1;
                    }
                }
                if let Some(content_end) = content.end_token {
                    token_end.line = content_end.line;
                    token_end.column = content_end.column + 1;
                }

                format!("{open}{content_str}{close}")
            }
            TokenTree::Ident(ident) => ident.to_string(),
            TokenTree::Literal(lit) => lit.to_string(),
            TokenTree::Punct(punct) => {
                let str = punct.as_char().to_string();
                if str == "'" {
                    add_space = false;
                }
                str
            },
        };
        debug!("{i}: [{token_start:?}-{token_end:?}] [{prev_token_end:?}]: {token}");

        if is_brace || prev_token_was_brace {
            if let Some(prev_token_end) = prev_token_end {
                if prev_token_end.line == token_start.line
                && prev_token_end.column >= token_start.column
                && output.ends_with(" ") {
                    output.pop();
                }
            }
        }
        prev_token_was_brace = is_brace;

        output.push_str(&token_str);
        if add_space {
            output.push(' ');
        }

        first_token_start.get_or_insert(token_start);
        prev_token_end = Some(token_end);
    }
    PrintOutput {
        output,
        start_token: first_token_start,
        end_token: prev_token_end,
    }
}

// ==================
// === Eval Macro ===
// ==================

enum Args {
    TokenStream { ident: syn::Ident },
    Pattern { str: TokenStream }
}

impl Args {
    fn pattern(&self) -> TokenStream {
        match self {
            Self::TokenStream { ident } => quote! { $($#ident:tt)* },
            Self::Pattern { str } => str.clone(),
        }
    }

    fn setup(&self) -> TokenStream {
        if let Self::TokenStream { ident } = self {
            quote! {
                use proc_macro2::TokenStream;
                let #ident: TokenStream = stringify!($($#ident)*).parse().unwrap();
            }
        } else {
            Default::default()
        }
    }
}

fn parse_args(
    args: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>
) -> Option<(Args, TokenStream)> {
    let Some(arg) = args.first() else {
        return Some((Args::Pattern { str: Default::default() }, TokenStream::new()))
    };

    // First try the specialized parsers, then fallback to our generic type handling.
    parse_args_for_pattern(arg)
        .or_else(|| parse_args_for_token_stream(arg))
        .map(|t| (t, TokenStream::new()))
        .or_else(|| {
            let mut is_first = true;
            let mut pat = quote!{};
            let mut code = TokenStream::new();

            for arg in args {
                if !is_first {
                    pat = quote! {#pat, };
                }
                is_first = false;
                if let syn::FnArg::Typed(pat_type) = arg {

                    if let syn::Pat::Ident(name) = &*pat_type.pat {
                        let name_str = name.ident.to_string();
                        let ty = &*pat_type.ty;
                        code = quote! {
                            #code
                            let #name: #ty =
                        };
                        if let Some((param_pat, param_code)) = parse_arg_type(&name_str, ty) {
                            pat = quote! {#pat #param_pat};
                            code = quote! {#code #param_code};
                        }
                        code = quote! {#code;};
                    }
                }
            }
            pat = quote! {#pat $(,)?};
            Some((Args::Pattern { str: pat }, code)) // FIXME middle token stream not computed
        })
}

/// Returns (pattern, code) for a given type. It supports both vector types and non‑vector types.
#[inline(always)]
fn parse_arg_type(pfx: &str, ty: &syn::Type) -> Option<(TokenStream, TokenStream)> {
    if let syn::Type::Path(type_path) = ty {
        let last_segment = type_path.path.segments.last()?;
        if last_segment.ident == "Vec" {
            if let syn::PathArguments::AngleBracketed(angle_bracketed) = &last_segment.arguments {
                let generic_arg = angle_bracketed.args.first()?;
                if let syn::GenericArgument::Type(inner_ty) = generic_arg {
                    if let Some((inner_pat, inner_code)) = parse_inner_type(pfx, inner_ty) {
                        let pat = quote! {[$(#inner_pat),*$(,)?]};
                        let code = quote! { [$(#inner_code),*].into_iter().collect() };
                        return Some((pat, code));
                    }
                }
            }
        } else {
            return parse_inner_type(pfx, ty);
        }
    }
    None
}

#[inline(always)]
fn parse_inner_type(pfx: &str, ty: &syn::Type) -> Option<(TokenStream, TokenStream)> {
    let arg_str = format!("{pfx}_arg");
    let arg_ident = syn::Ident::new(&arg_str, Span::call_site());
    let arg = quote! {$#arg_ident};
    match ty {
        syn::Type::Reference(ty_ref) => {
            if let syn::Type::Path(inner_path) = &*ty_ref.elem {
                if let Some(inner_seg) = inner_path.path.segments.last() {
                    if inner_seg.ident == "str" {
                        return Some((quote!{#arg:expr}, quote!{crabtime::stringify_if_needed!{#arg}}));
                    }
                }
            }
        },
        syn::Type::Path(inner_type_path) => {
            if let Some(inner_seg) = inner_type_path.path.segments.last() {
                let ident_str = inner_seg.ident.to_string();
                if ident_str == "String" {
                    return Some((quote!{#arg:expr}, quote!{crabtime::stringify_if_needed!(#arg).to_string()}));
                } else if matches!(ident_str.as_str(),
                    "usize" | "u8" | "u16" | "u32" | "u64" | "u128" |
                    "isize" | "i8" | "i16" | "i32" | "i64" | "i128"
                ) {
                    return Some((quote!{#arg:literal}, quote!{#arg}));
                }
            }
        },
        _ => {}
    }
    None
}

fn parse_args_for_pattern(arg: &syn::FnArg) -> Option<Args> {
    let syn::FnArg::Typed(pat) = arg else { return None };
    let syn::Pat::Macro(m) = &*pat.pat else { return None };
    Some(Args::Pattern {str: m.mac.tokens.clone() })
}

fn parse_args_for_token_stream(arg: &syn::FnArg) -> Option<Args> {
    let syn::FnArg::Typed(pat) = arg else { return None };
    let syn::Pat::Ident(pat_ident) = &*pat.pat else { return None };
    let tp = &pat.ty;
    let tp_str = quote! { #tp }.to_string();
    if tp_str != "TokenStream" { return None }
    let ident = pat_ident.ident.clone();
    Some(Args::TokenStream { ident })
}

const WRONG_ARGS: &str = "Function should have zero or one argument, one of:
    - `pattern!(<pattern>): _`, where <pattern> is a `macro_rules!` pattern
    - `input: TokenStream`
";

fn prepare_input_code(attributes:&str, body: &str, output_tp: &str, include_token_stream_impl: bool) -> String {
    let body_esc: String = body.chars().flat_map(|c| c.escape_default()).collect();
    let prelude = gen_prelude(include_token_stream_impl);
    format!("
        {attributes}
        {prelude}

        const SOURCE_CODE: &str = \"{body_esc}\";

        fn main() {{
            let mut __output_buffer__ = String::new();
            let result: {output_tp} = {{
                {body}
            }};
            __output_buffer__.push_str(&{GEN_MOD}::code_from_output(result));
            println!(\"{{}}\", {GEN_MOD}::prefix_lines_with_output(&__output_buffer__));
        }}",
    )
}

fn parse_output(output: &str) -> String {
    let mut code = String::new();
    for line in output.split('\n') {
        let line_trimmed = line.trim();
        if let Some(stripped) = line_trimmed.strip_prefix(OUTPUT_PREFIX) {
            code.push_str(stripped);
            code.push('\n');
        } else if let Some(stripped) = line_trimmed.strip_prefix(Level::WARNING_PREFIX) {
            print_warning!("{}", stripped);
        } else if let Some(stripped) = line_trimmed.strip_prefix(Level::ERROR_PREFIX) {
            print_error!("{}", stripped);
        } else if !line_trimmed.is_empty() {
            println!("{line}");
        }
    }
    code
}

#[derive(Clone, Copy, Debug)]
struct MacroOptions {
    pub cache: bool,
    pub content_base_name: bool,
}

impl Default for MacroOptions {
    fn default() -> Self {
        Self {
            cache: true,
            content_base_name: false,
        }
    }
}

impl syn::parse::Parse for MacroOptions {
    fn parse(input: syn::parse::ParseStream) -> Result<Self, syn::Error> {
        let mut options = MacroOptions::default();
        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            let _eq_token: syn::Token![=] = input.parse()?;
            if ident == "cache" {
                let bool_lit: syn::LitBool = input.parse()?;
                options.cache = bool_lit.value;
            } else if ident == "content_base_name" {
                let bool_lit: syn::LitBool = input.parse()?;
                options.content_base_name = bool_lit.value;
            } else {
                return Err(syn::Error::new(ident.span(), "unknown attribute"));
            }
            if input.peek(syn::Token![,]) {
                let _comma: syn::Token![,] = input.parse()?;
            }
        }
        Ok(options)
    }
}

// =====================
// === Eval Function ===
// =====================

#[proc_macro_attribute]
pub fn eval_fn(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    // SAFETY: Used to panic in case of error.
    #[allow(clippy::unwrap_used)]
    eval_fn_impl(attr, item).unwrap_or_compile_error().into()
}


fn eval_fn_impl(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream
) -> Result<TokenStream> {
    let options = syn::parse::<MacroOptions>(attr)?;
    let start_time = get_current_time();
    let timer = std::time::Instant::now();

    let input_fn_ast = syn::parse::<syn::ItemFn>(item)?;
    let name = &input_fn_ast.sig.ident.to_string();
    let body_ast = &input_fn_ast.block.stmts;
    let output_tp = &input_fn_ast.sig.output;
    let input_str = expand_output_macro(expand_quote_macro(quote!{ #(#body_ast)* })).to_string();
    let paths = Paths::new(options, name, &input_str)?;

    let mut cfg = CargoConfig::default();
    if let Some(path) = &paths.cargo_toml_path {
        cfg.fill_from_cargo_toml(path)?;
    }
    let attributes = cfg.extract_inline_attributes(input_fn_ast.attrs)?;
    let include_token_stream_impl = cfg.contains_dependency("proc-macro2");
    let output_tp_str = match output_tp {
        syn::ReturnType::Default => "()".to_string(),
        syn::ReturnType::Type(_, tp) => quote!{#tp}.to_string(),
    };
    let input_code = prepare_input_code(&attributes, &input_str, &output_tp_str, include_token_stream_impl);
    debug!("INPUT CODE: {input_code}");
    let mut output_dir_str = String::new();
    let (output, was_cached) = paths.with_output_dir(options.cache, |output_dir| {
        debug!("OUTPUT_DIR: {:?}", output_dir);
        output_dir_str = output_dir.to_string_lossy().to_string();
        let was_cached = create_project_skeleton(output_dir, cfg, &input_code)?;
        let output = run_cargo_project(output_dir)?;
        Ok((output, was_cached))
    })?;
    let output_code = parse_output(&output);
    let duration = format_duration(timer.elapsed());
    let options_doc = format!("{options:#?}").replace("\n", "\n/// ");
    let macro_code = format!("
        /// # Compilation Stats
        /// Start: {start_time}
        /// Duration: {duration}
        /// Cached: {was_cached}
        /// Output Dir: {output_dir_str}
        /// Macro Options: {options_doc}
        const _: () = ();
        {output_code}
    ");

    debug!("BODY: {macro_code}");
    let out: TokenStream = macro_code.parse()
        .map_err(|err| error!("{err:?}"))
        .context("Failed to parse generated code.")?;
    debug!("OUTPUT: {out} ");
    Ok(out)
}

// ================
// === Function ===
// ================

#[proc_macro_attribute]
pub fn function(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    // SAFETY: Used to panic in case of error.
    #[allow(clippy::unwrap_used)]
    function_impl(attr, item).unwrap_or_compile_error().into()
}

fn function_impl(
    attr_in: proc_macro::TokenStream,
    item: proc_macro::TokenStream
) -> Result<TokenStream> {
    let attr: TokenStream = attr_in.into();
    let input_fn_ast = syn::parse::<syn::ItemFn>(item)?;
    let name = &input_fn_ast.sig.ident;
    let args_ast = &input_fn_ast.sig.inputs;
    let body_ast = &input_fn_ast.block.stmts;
    let output_tp = &input_fn_ast.sig.output;

    let (args, args_code) = parse_args(args_ast).context(|| error!(WRONG_ARGS))?;
    let args_pattern = args.pattern();
    let args_setup = args.setup();
    let body = quote!{ #(#body_ast)* };
    let input_str = expand_expand_macro(quote!{ #(#body_ast)* });

    // Check if the expansion engine is Rust Analyzer. If so, we need to generate
    // a code which looks like a function to enable type hints.
    let program_name = std::env::current_exe()?
        .file_name()
        .map_or_else(|| "unknown".into(), |s| s.to_string_lossy().into_owned());
    let rust_analyzer_hints = if program_name.contains("rust-analyzer") {
        quote! {
            mod __rust_analyzer_hints__ {
                #[test]
                #[ignore]
                fn mytest() {
                    #body
                }
            }
        }
    } else {
        quote! {}
    };

    let mut attrs_vec = input_fn_ast.attrs;
    let export_attr_opt = remove_macro_export_attribute(&mut attrs_vec);

    let attrs = quote!{ #(#attrs_vec)* };
    #[cfg(not(test))]
    let out = quote! {
        #rust_analyzer_hints

        #export_attr_opt
        macro_rules! #name {
            (#args_pattern) => {
                #[crabtime::eval_fn(#attr)]
                fn #name() #output_tp {
                    #attrs
                    #args_setup
                    #args_code
                    #input_str
                }
            };
        }
    };
    // panic!("OUT: {out}");
    Ok(out)
}

fn format_duration(duration: std::time::Duration) -> String {
    let total_seconds = duration.as_secs();
    if total_seconds >= 60 {
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{minutes}m {seconds}s")
    } else {
        let millis = duration.as_millis() % 1000;
        let fractional = millis as f64 / 1000.0;
        format!("{:.2} s", total_seconds as f64 + fractional)
    }
}

fn get_current_time() -> String {
    let now = std::time::SystemTime::now();
    #[allow(clippy::unwrap_used)]
    let duration_since_epoch = now.duration_since(std::time::UNIX_EPOCH).unwrap();
    let total_seconds = duration_since_epoch.as_secs();
    let milliseconds = (duration_since_epoch.as_millis() % 1000) as u32;
    let hours = (total_seconds / 3600) % 24;
    let minutes = (total_seconds / 60) % 60;
    let seconds = total_seconds % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02} ({milliseconds:03})")
}


fn remove_macro_export_attribute(attrs: &mut Vec<syn::Attribute>) -> Option<syn::Attribute> {
    attrs.iter().position(|attr| attr.path().is_ident("macro_export")).map(|pos| attrs.remove(pos))
}

// TODO: get lints from Cargo
// TODO: support workspaces, for edition and dependencies or is it done automatically for edition?
// TODO: removing project can cause another process to fail - after compilation, another process might already acquire lock


