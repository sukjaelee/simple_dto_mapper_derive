//! Compile-fail diagnostics test suite.
//!
//! Uses `trybuild` to run all `.rs` files in `tests/ui` and assert that they
//! fail to compile with the expected `.stderr` output. This ensures that the
//! `#[derive(DtoFrom)]` macro emits clear, accurate error messages for:
//! - Unknown or unsupported attributes
//! - Duplicate or conflicting attributes
//! - Empty or invalid `rename` values
//! - Struct-level misuse
//! - Other violations of the mapping rules

#[test]
fn compile_fail_diagnostics() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
