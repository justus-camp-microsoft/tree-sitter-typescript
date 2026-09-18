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
    fn contextual_type_names_in_exports() {
        for language in [super::LANGUAGE_TYPESCRIPT, super::LANGUAGE_TSX] {
            for source in [
                "export { type };",
                "export { type, value } from './source';",
                "export { type as renamed, value as type } from './source';",
                "export { type as type } from './source';",
                "export { type Item, type type, type type as Renamed } from './source';",
                "export type { type, type as Renamed } from './source';",
                "export { 'type' as type, type as 'type' } from './source';",
                "export * as type from './source'; export type * as type from './source';",
            ] {
                parse(source, language);
            }
            let tree = parse("export { type, type Item };", language);
            let export = tree.root_node().named_child(0).unwrap();
            let clause = export.named_child(0).unwrap();
            let value = clause.named_child(0).unwrap();
            assert_eq!(
                value.child_by_field_name("name").unwrap().kind(),
                "identifier"
            );
            assert_eq!(value.child_count(), 1);
            let type_only = clause.named_child(1).unwrap();
            assert_eq!(type_only.child(0).unwrap().kind(), "type");
            assert_eq!(type_only.child_count(), 2);
        }
    }

    #[test]
    fn contextual_labels_in_tuple_type_arguments() {
        for language in [super::LANGUAGE_TYPESCRIPT, super::LANGUAGE_TSX] {
            for label in [
                "type",
                "value",
                "readonly",
                "any",
                "unknown",
                "out",
                "as",
                "satisfies",
                "never",
                "number",
                "boolean",
                "string",
                "symbol",
                "object",
                "unique",
                "void",
                "typeof",
                "keyof",
                "infer",
                "this",
                "true",
                "false",
                "null",
                "undefined",
                "const",
                "abstract",
                "import",
                "function",
                "class",
                "new",
                "await",
                "yield",
                "super",
                "delete",
                "return",
                "switch",
                "case",
            ] {
                for member in [
                    format!("{label}: string"),
                    format!("{label}?: string"),
                    format!("...{label}: string[]"),
                ] {
                    for source in [
                        format!("type Args = [{member}];"),
                        format!("const f = sandbox.stub<[{member}], void>();"),
                        format!("const f = sandbox\n .stub<[{member}], void>()\n .callsFake(() => {{}});"),
                    ] {
                        let tree = parse(&source, language);
                        assert!(tree.root_node().to_sexp().contains("tuple_type"), "{source}");
                    }
                }
            }
            parse(
                "const f = sandbox.stub<[type: string, content: unknown, localOpMetadata: unknown, squash: boolean], void>().callsFake((type: string, content: unknown, localOpMetadata: unknown, squash: boolean) => {});",
                language,
            );
            let tree = parse(
                "type Args = [type: string, readonly?: number, ...unknown: boolean[]];",
                language,
            );
            let tuple = tree
                .root_node()
                .named_child(0)
                .unwrap()
                .child_by_field_name("value")
                .unwrap();
            let required = tuple.named_child(0).unwrap();
            let optional = tuple.named_child(1).unwrap();
            let rest = tuple.named_child(2).unwrap();
            assert_eq!(required.kind(), "required_parameter");
            assert_eq!(
                required.child_by_field_name("name").unwrap().kind(),
                "identifier"
            );
            assert_eq!(optional.kind(), "optional_parameter");
            assert_eq!(
                optional.child_by_field_name("name").unwrap().kind(),
                "identifier"
            );
            assert_eq!(
                rest.child_by_field_name("name").unwrap().kind(),
                "rest_pattern"
            );
            assert_eq!(rest.named_child_count(), 2);
            let rest_name = rest.child_by_field_name("name").unwrap();
            assert_eq!(rest_name.named_child_count(), 1);
            assert_eq!(rest_name.named_child(0).unwrap().kind(), "identifier");
            let ordinary = parse("type Args = [number, boolean?, ...string[]];", language);
            let sexp = ordinary.root_node().to_sexp();
            assert!(sexp.contains("(optional_type (predefined_type))"), "{sexp}");
            assert!(
                sexp.contains("(rest_type (array_type (predefined_type)))"),
                "{sexp}"
            );
            assert!(!sexp.contains("required_parameter"), "{sexp}");
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
                "export { type Item as } from './source';",
                "export { type as } from './source';",
                "sandbox.stub<[type: ], void>();",
                "sandbox.stub<[type?: ], void>();",
                "sandbox.stub<[...type: ], void>();",
                "type Args = [...{ name }: string[]];",
            ] {
                assert!(
                    parser.parse(source, None).unwrap().root_node().has_error(),
                    "{source}"
                );
            }
        }
    }
}
