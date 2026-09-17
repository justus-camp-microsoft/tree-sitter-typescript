//! This crate provides TypeScript and TSX language support for the [tree-sitter][] parsing library.
//!
//! Typically, you will use the [LANGUAGE_TYPESCRIPT] constant to add this language to a
//! tree-sitter [Parser][], and then use the parser to parse some code:
//!
//! ```
//! use tree_sitter::Parser;
//!
//! let code = r#"
//! function double(x: number): number {
//!     return x * 2;
//! }
//! "#;
//! let mut parser = Parser::new();
//! let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
//! parser
//!     .set_language(&language.into())
//!     .expect("Error loading TypeScript parser");
//! let tree = parser.parse(code, None).unwrap();
//! assert!(!tree.root_node().has_error());
//! ```
//!
//! [Parser]: https://docs.rs/tree-sitter/*/tree_sitter/struct.Parser.html
//! [tree-sitter]: https://tree-sitter.github.io/

use tree_sitter_language::LanguageFn;

/// Build-time identity of the bundled grammar and generated parser sources.
pub const SOURCE_FINGERPRINT: &str = env!("EROSION_TYPESCRIPT_FINGERPRINT");

extern "C" {
    fn tree_sitter_typescript() -> *const ();
    fn tree_sitter_tsx() -> *const ();
}

/// The tree-sitter [`LanguageFn`] for TypeScript.
///
/// [LanguageFn]: https://docs.rs/tree-sitter-language/*/tree_sitter_language/struct.LanguageFn.html
pub const LANGUAGE_TYPESCRIPT: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_typescript) };

/// The tree-sitter [`LanguageFn`] for TSX.
///
/// [LanguageFn]: https://docs.rs/tree-sitter-language/*/tree_sitter_language/struct.LanguageFn.html
pub const LANGUAGE_TSX: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_tsx) };

/// The content of the [`node-types.json`][] file for TypeScript.
///
/// [`node-types.json`]: https://tree-sitter.github.io/tree-sitter/using-parsers#static-node-types
pub const TYPESCRIPT_NODE_TYPES: &str = include_str!("../../typescript/src/node-types.json");

/// The content of the [`node-types.json`][] file for TSX.
///
/// [`node-types.json`]: https://tree-sitter.github.io/tree-sitter/using-parsers#static-node-types
pub const TSX_NODE_TYPES: &str = include_str!("../../tsx/src/node-types.json");

/// The syntax highlighting query for TypeScript.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../../queries/highlights.scm");

/// The local-variable syntax highlighting query for TypeScript.
pub const LOCALS_QUERY: &str = include_str!("../../queries/locals.scm");

/// The symbol tagging query for TypeScript.
pub const TAGS_QUERY: &str = include_str!("../../queries/tags.scm");

#[cfg(test)]
mod tests {
    #[test]
    fn test_can_load_typescript_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&super::LANGUAGE_TYPESCRIPT.into())
            .expect("Error loading TypeScript parser");
    }

    #[test]
    fn test_can_load_tsx_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&super::LANGUAGE_TSX.into())
            .expect("Error loading TSX parser");
    }

    fn parse(source: &str, language: tree_sitter_language::LanguageFn) -> tree_sitter::Tree {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language.into()).unwrap();
        let tree = parser.parse(source, None).unwrap();
        assert!(
            !tree.root_node().has_error(),
            "{source}\n{}",
            tree.root_node().to_sexp()
        );
        tree
    }

    #[test]
    fn compatibility_syntax_in_both_dialects() {
        let cases = [
            "export type * from './types'; export type * as api from './types';",
            "interface Producer<out T> { readonly value: T; }",
            "interface Consumer<in T> { consume(value: T): void; }",
            "interface Cell<in out T> { get(): T; set(value: T): void; }",
            "class C<const in T> {}",
            "interface A<out> { value: out; } interface B<T, out> {}",
            "class Q<out> extends Array<out> {}",
            "function f<out>(x: out): out { return x; }",
            "type P<out = string> = out; interface Covariant<out out> {}",
            "type Callback = (any) => any;",
            "interface Events { handler: (readonly?: boolean) => void; }",
            "type Callback = (unknown, never) => any;",
            "type A = (any); type B = readonly number[];",
            "class C { constructor(readonly value: string) {} }",
            "type N = -1; type Boxed = Box<-1,>; f<-1>(); f<-1,>(); const f2 = f<-1>;",
        ];
        for language in [super::LANGUAGE_TYPESCRIPT, super::LANGUAGE_TSX] {
            for source in cases {
                parse(source, language);
            }
        }
    }

    #[test]
    fn comparisons_are_expressions_not_type_arguments() {
        for language in [super::LANGUAGE_TYPESCRIPT, super::LANGUAGE_TSX] {
            for rhs in ["-1", "+1", "1"] {
                for suffix in [");", ",);", ", other);"] {
                    let source = format!("items.filter((item: Item) => item.value < {rhs}{suffix}");
                    let tree = parse(&source, language);
                    let sexp = tree.root_node().to_sexp();
                    assert!(sexp.contains("body: (binary_expression"), "{sexp}");
                    assert!(!sexp.contains("type_arguments"), "{sexp}");
                }
            }
        }
    }

    #[test]
    fn malformed_neighbors_still_fail() {
        for language in [super::LANGUAGE_TYPESCRIPT, super::LANGUAGE_TSX] {
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&language.into()).unwrap();
            for source in [
                "export type * from ;",
                "interface Broken<out T> { value: }",
                "items.filter((item: Item) => item.value < ,);",
                "function {",
            ] {
                assert!(
                    parser.parse(source, None).unwrap().root_node().has_error(),
                    "{source}"
                );
            }
        }
    }
}
