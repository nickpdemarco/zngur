use std::panic::catch_unwind;

use expect_test::{Expect, expect};
use zngur_def::{RustPathAndGenerics, RustType};

use crate::{ImportResolver, ParsedZngFile};

fn check_success(zng: &str) {
    let _ = ParsedZngFile::parse_str(zng);
}

pub struct ErrorText(pub String);

fn check_fail(zng: &str, error: Expect) {
    let r = catch_unwind(|| {
        let _ = ParsedZngFile::parse_str(zng);
    });
    match r {
        Ok(_) => panic!("Parsing succeeded but we expected fail"),
        Err(e) => match e.downcast::<ErrorText>() {
            Ok(t) => error.assert_eq(&t.0),
            Err(e) => std::panic::resume_unwind(e),
        },
    }
}

fn check_import_fail(zng: &str, error: Expect, resolver: &MockFilesystem) {
    let r = catch_unwind(|| {
        let _ = ParsedZngFile::parse_str_with_resolver(zng, resolver);
    });

    match r {
        Ok(_) => panic!("Parsing succeeded but we expected fail"),
        Err(e) => match e.downcast::<ErrorText>() {
            Ok(t) => error.assert_eq(&t.0),
            Err(e) => std::panic::resume_unwind(e),
        },
    }
}

#[test]
fn parse_unit() {
    check_fail(
        r#"
type () {
    #layout(size = 0, align = 1);
    wellknown_traits(Copy);
}
    "#,
        expect![[r#"
            Error: Unit type is declared implicitly. Remove this entirely.
               ╭─[<string literal>:2:6]
               │
             2 │ type () {
               │      ─┬
               │       ╰── Unit type is declared implicitly. Remove this entirely.
            ───╯
        "#]],
    );
}

#[test]
fn parse_tuple() {
    check_success(
        r#"
type (i8, u8) {
    #layout(size = 0, align = 1);
}
    "#,
    );
}

#[test]
fn typo_in_wellknown_trait() {
    check_fail(
        r#"
type () {
    #layout(size = 0, align = 1);
    welcome_traits(Copy);
}
    "#,
        expect![[r#"
            Error: found 'welcome_traits' expected '#', 'wellknown_traits', 'constructor', 'field', 'fn', or '}'
               ╭─[<string literal>:4:5]
               │
             4 │     welcome_traits(Copy);
               │     ───────┬──────
               │            ╰──────── found 'welcome_traits' expected '#', 'wellknown_traits', 'constructor', 'field', 'fn', or '}'
            ───╯
        "#]],
    );
}

#[test]
fn multiple_layout_policies() {
    check_fail(
        r#"
type ::std::string::String {
    #layout(size = 24, align = 8);
    #heap_allocated;
}
    "#,
        expect![[r#"
            Error: Duplicate layout policy found
               ╭─[<string literal>:4:5]
               │
             4 │     #heap_allocated;
               │     ───────┬───────
               │            ╰───────── Duplicate layout policy found
            ───╯
        "#]],
    );
}

#[test]
fn cpp_ref_should_not_need_layout_info() {
    check_fail(
        r#"
type crate::Way {
    #layout(size = 1, align = 2);

    #cpp_ref "::osmium::Way";
}
    "#,
        expect![[r#"
            Error: Duplicate layout policy found
               ╭─[<string literal>:3:5]
               │
             3 │     #layout(size = 1, align = 2);
               │     ──────────────┬─────────────
               │                   ╰─────────────── Duplicate layout policy found
            ───╯
        "#]],
    );
    check_success(
        r#"
type crate::Way {
    #cpp_ref "::osmium::Way";
}
    "#,
    );
}

#[test]
fn alias_expands_correctly() {
    let parsed = ParsedZngFile::parse_str(
        r#"
use ::std::string::String as MyString;
type MyString {
    #layout(size = 24, align = 8);
}
    "#,
    );
    let ty = parsed.types.first().expect("no type parsed");
    let RustType::Adt(RustPathAndGenerics { path: p, .. }) = &ty.ty else {
        panic!("no match?");
    };
    assert_eq!(p.as_slice(), ["std", "string", "String"]);
}

#[test]
fn alias_expands_nearest_scope_first() {
    let parsed = ParsedZngFile::parse_str(
        r#"
use ::std::string::String as MyString;
mod crate {
    use MyLocalString as MyString;
    type MyString {
        #layout(size = 24, align = 8);
    }
}
    "#,
    );
    let ty = parsed.types.first().expect("no type parsed");
    let RustType::Adt(RustPathAndGenerics { path: p, .. }) = &ty.ty else {
        panic!("no match?");
    };
    assert_eq!(p.as_slice(), ["crate", "MyLocalString"]);
}

#[test]
fn import_parser_test() {
    // Test that import statements can be parsed successfully
    let parsed = ParsedZngFile::parse_str(
        r#"
import "./relative/path.zng";
type Example {
    #layout(size = 1, align = 1);
}
    "#,
    );
    assert_eq!(parsed.imports.len(), 1);
    assert_eq!(
        parsed.imports[0].0,
        std::path::PathBuf::from("./relative/path.zng")
    );
}

struct MockFilesystem {
    files: std::collections::HashMap<std::path::PathBuf, String>,
}

impl MockFilesystem {
    fn new(
        files: impl IntoIterator<Item = (impl Into<std::path::PathBuf>, impl Into<String>)>,
    ) -> Self {
        Self {
            files: files
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        }
    }
}

impl ImportResolver for MockFilesystem {
    fn resolve_import(
        &self,
        cwd: &std::path::Path,
        relpath: &std::path::Path,
    ) -> Result<String, String> {
        let path = cwd.join(relpath);
        self.files
            .get(&path)
            .cloned()
            .ok_or_else(|| format!("File not found: {}", path.display()))
    }
}

#[test]
fn import_has_conflict() {
    // Test that an import which introduces a conflict produces a reasonable error message.
    let resolver = MockFilesystem::new(vec![(
        "a.zng",
        r#"
      type A {
        #layout(size = 1, align = 1);
      }
    "#,
    )]);

    check_import_fail(
        r#"
    import "a.zng";
    type A {
      #layout(size = 2, align = 2);
    }
"#,
        expect![[r#"
            Error: Duplicate layout policy found
               ╭─[a.zng:2:12]
               │
             2 │       type A {
               │            ┬
               │            ╰── Duplicate layout policy found
            ───╯
        "#]],
        &resolver,
    );
}

#[test]
fn import_not_found() {
    let resolver = MockFilesystem::new(vec![] as Vec<(&str, &str)>);
    check_import_fail(
        r#"
    import "a.zng";
    "#,
        expect![[r#"
      Error: Import path not found: a.zng
    "#]],
        &resolver,
    );
}
