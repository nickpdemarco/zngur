use std::panic::catch_unwind;

use expect_test::{Expect, expect};
use zngur_def::{Import, RustPathAndGenerics, RustType};

use crate::ParsedZngFile;

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
               ╭─[main.zng:2:6]
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
            Error: found 'welcome_traits' expected 'layout', '#', 'wellknown_traits', 'constructor', 'fn', or '}'
               ╭─[main.zng:4:5]
               │
             4 │     welcome_traits(Copy);
               │     ───────┬──────  
               │            ╰──────── found 'welcome_traits' expected 'layout', '#', 'wellknown_traits', 'constructor', 'fn', or '}'
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
               ╭─[main.zng:4:5]
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
               ╭─[main.zng:3:5]
               │
             3 │     #layout(size = 1, align = 2);
               │     ─────────────┬─────────────  
               │                  ╰─────────────── Duplicate layout policy found
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
    let ty = parsed.types.iter().next().expect("no type parsed");
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
    let ty = parsed.types.iter().next().expect("no type parsed");
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
import "/absolute/path.zng";
type Example {
    #layout(size = 1, align = 1);
}
    "#,
    );
    assert_eq!(parsed.imports.len(), 2);
    match &parsed.imports[0] {
        Import::FilePath(path) => {
            assert_eq!(*path, std::path::PathBuf::from("./relative/path.zng"));
        }
        Import::DependencyPath { .. } => panic!("Expected FilePath import"),
    }
    match &parsed.imports[1] {
        Import::FilePath(path) => {
            assert_eq!(*path, std::path::PathBuf::from("/absolute/path.zng"));
        }
        Import::DependencyPath { .. } => panic!("Expected FilePath import"),
    }
}

#[test]
fn test_import_path_types() {
    // Test file path import
    let spec = ParsedZngFile::parse_str(
        r#"
        import "test/file.zng";
    "#,
    );

    assert_eq!(spec.imports.len(), 1);
    match &spec.imports[0] {
        Import::FilePath(path) => {
            assert_eq!(path.to_string_lossy(), "test/file.zng");
        }
        Import::DependencyPath { .. } => panic!("Expected FilePath import"),
    }

    // Test dependency-based import
    let spec = ParsedZngFile::parse_str(
        r#"
        import "@my-crate/test/file.zng";
    "#,
    );

    assert_eq!(spec.imports.len(), 1);
    match &spec.imports[0] {
        Import::DependencyPath {
            crate_name,
            relative_path,
        } => {
            assert_eq!(crate_name, "my-crate");
            assert_eq!(relative_path.to_string_lossy(), "test/file.zng");
        }
        Import::FilePath(_) => panic!("Expected DependencyPath import"),
    }
}
