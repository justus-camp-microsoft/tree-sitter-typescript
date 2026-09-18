# Erosion compatibility branch

`erosion-compat` carries `tree-sitter-typescript 0.23.2-erosion.2`, based on
upstream 0.23.2 commit `f975a621f4e7f532fe322e13c4f79495e0a7b2e7`.
The upstream [MIT license](LICENSE) and attribution are retained.

## Grammar changes

- `in`, `out`, and `in out` generic variance annotations, preserving contextual
  type parameters named `out`.
- `export type *` and `export type * as Name` re-exports.
- Named exports of an identifier called `type`, including aliases and
  type-only specifiers, without conflating the name with the modifier.
- Required, optional, and rest tuple labels using contextual/reserved keywords,
  including in generic calls. Labels retain identifier/rest-pattern AST shapes;
  unlabeled optional and rest types keep their existing shapes.
- Contextual parameter identifiers (`any`, `readonly`, `unknown`, `never`).
- Signed-number comparisons in call arguments such as
  `items.filter((item: Item) => item.value < -1,)`.

The feature changes correspond to upstream variance PRs
[#361](https://github.com/tree-sitter/tree-sitter-typescript/pull/361) /
[#364](https://github.com/tree-sitter/tree-sitter-typescript/pull/364) and export PRs
[#358](https://github.com/tree-sitter/tree-sitter-typescript/pull/358) /
[#360](https://github.com/tree-sitter/tree-sitter-typescript/pull/360).
These are focused adaptations, not wholesale merges of those PR branches.
The expression/type ambiguity fixes replace eager static precedence with explicit
GLR conflicts. They do not rewrite source or suppress parsing errors.

## Rust consumption

Consumers can override the crates.io dependency with `[patch.crates-io]` pointing
at this fork and a full commit SHA. Pin `=0.23.2-erosion.2` as the dependency
version and commit the consumer's Cargo.lock. Do not follow a moving branch for
reproducible measurements.

Normal Rust builds compile the checked-in C parsers and require no Node,
Tree-sitter CLI, or measured-project dependencies. The generated parsers retain
ABI 14 and are tested against Tree-sitter 0.25.2.

The Rust build script exports `SOURCE_FINGERPRINT`, a digest of grammar, generated
parser/scanner/header sources, bindings, and build inputs. Consumers can include
it in their analysis-cache identity without embedding a second copy of generated
C source. Keep the build script's input list complete when adding source files.

## Maintaining the grammar

```console
npm ci --ignore-scripts --registry=https://registry.npmjs.org
npm rebuild tree-sitter-cli
npm run generate
npm run test:corpus
cargo fmt -- --check
cargo test
cargo clippy --all-targets -- -D warnings
```

The lockfile pins generator `tree-sitter-cli 0.24.4` and inherited grammar
`tree-sitter-javascript 0.23.1`. Regenerate TypeScript and TSX together and commit
the generated C, grammar JSON, node types, and headers. Repeating generation with
unchanged inputs must produce byte-identical outputs. Node bindings and their
existing tests are separate from the Rust-consumer workflow above.

Rust tests exercise the local fixes in both dialects, expression AST shapes,
and malformed neighboring inputs. The upstream grammar corpus is retained.
Prefer an upstream release once it incorporates equivalent fixes and passes
these regressions and representative consumer measurements.
