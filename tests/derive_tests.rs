//! Integration test demonstrating the `#[derive(DtoFrom)]` macro features.
//!
//! Covers:
//! - Direct 1:1 field mapping
//! - Field rename with `#[dto(rename = "...")]`
//! - Field transformation with `#[dto(transform_fn = path)]`
//! - Skipped fields via `#[dto(skip)]` (default-initialized)
//! - Conversion with `#[dto(into)]`
//! - Collection mapping via `transform_fn`

use simple_dto_mapper_derive::DtoFrom;

mod types {
    // ----- source side -----
    #[derive(Debug, Clone)]
    pub enum SourceStatus {
        Active,
        Inactive,
    }

    #[derive(Debug)]
    pub struct Source {
        pub id: String,
        pub name: String,
        pub age: u32,
        pub note: Option<String>,
        pub status: SourceStatus,
        pub tags: Vec<String>,
    }

    // ----- dto side -----
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum DtoStatus {
        Active,
        Inactive,
    }

    impl From<SourceStatus> for DtoStatus {
        fn from(s: SourceStatus) -> Self {
            match s {
                SourceStatus::Active => DtoStatus::Active,
                SourceStatus::Inactive => DtoStatus::Inactive,
            }
        }
    }

    // ----- helpers for transform_fn -----
    pub fn to_display_name(name: String) -> String {
        // make the display name loud for the test
        name.to_uppercase()
    }

    pub fn tags_to_lengths(v: Vec<String>) -> Vec<usize> {
        v.into_iter().map(|s| s.len()).collect()
    }
}

use types::*;

#[derive(Debug, DtoFrom)]
#[dto(from = types::Source)]
pub struct Dto {
    // direct 1:1 mapping
    pub id: String,

    // rename + transform
    #[dto(rename = "name", transform_fn = types::to_display_name)]
    pub display_name: String,

    // another direct mapping
    pub age: u32,

    // direct mapping of Option
    pub note: Option<String>,

    // skip -> Default::default()
    #[dto(skip)]
    pub placeholder: Option<String>,

    // into -> uses From<SourceStatus> for DtoStatus
    #[dto(into)]
    pub status: types::DtoStatus,

    // collection via transform_fn
    #[dto(rename = "tags", transform_fn = types::tags_to_lengths)]
    pub tag_lengths: Vec<usize>,
}

#[test]
fn test_basic_mapping() {
    let src = Source {
        id: "u1".into(),
        name: "Alice".into(),
        age: 42,
        note: Some("hi".into()),
        status: SourceStatus::Active,
        tags: vec!["ab".into(), "rust".into(), "dto".into()],
    };

    let dto: Dto = src.into();

    assert_eq!(dto.id, "u1");
    assert_eq!(dto.display_name, "ALICE");
    assert_eq!(dto.age, 42);
    assert_eq!(dto.note.as_deref(), Some("hi"));
    assert_eq!(dto.placeholder, None);
    assert_eq!(dto.status, DtoStatus::Active);
    assert_eq!(dto.tag_lengths, vec![2, 4, 3]);
}
