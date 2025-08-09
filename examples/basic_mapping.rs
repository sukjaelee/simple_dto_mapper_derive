//! Basic mapping example
//!
//! This file shows the four attributes supported by the macro, each exactly once:
//! - `rename`  — map from a differently named source field
//! - `transform_fn` — apply a custom owned transform
//! - `skip`    — initialize with `Default::default()`
//! - `into`    — convert via `From`/`Into` (infallible)

use chrono::{DateTime, Utc};
use simple_dto_mapper_derive::DtoFrom;

mod types {
    use chrono::{DateTime, Utc};

    // Source type definition (model) — some fields are intentionally not mapped into the DTO
    /// Source type definition (model) — some fields are intentionally not mapped into the DTO
    pub struct User {
        pub id: String,
        pub name: String,
        pub age: u32,
        pub password: String, // intentionally not mapped into UserDto
        pub note: Option<String>,
        pub status: SourceStatus,
        pub rank: Rank,
        pub level: i32,
        pub created_at: Option<DateTime<Utc>>,
    }

    #[derive(Debug, Clone)]
    pub enum SourceStatus {
        Active,
        Inactive,
    }

    #[derive(Debug, Clone)]
    pub enum DtoStatus {
        Active,
        Inactive,
    }

    // Lossless enum mapping via Into/From
    impl From<SourceStatus> for DtoStatus {
        fn from(s: SourceStatus) -> Self {
            match s {
                SourceStatus::Active => DtoStatus::Active,
                SourceStatus::Inactive => DtoStatus::Inactive,
            }
        }
    }

    /// Transformer function to mask names (handles empty strings safely)
    #[allow(dead_code)]
    pub fn mask_name(name: String) -> String {
        let first = name.chars().next().unwrap_or('*');
        format!("{}***", first)
    }

    #[derive(Debug, Clone)]
    pub enum Rank {
        Junior,
        Senior,
    }

    // Example of a fallible conversion available to the domain (not used by the macro now)
    impl core::convert::TryFrom<i32> for Rank {
        type Error = &'static str;
        fn try_from(v: i32) -> Result<Self, Self::Error> {
            match v {
                0 => Ok(Rank::Junior),
                1 => Ok(Rank::Senior),
                _ => Err("unknown rank"),
            }
        }
    }

    impl Default for Rank {
        fn default() -> Self {
            Rank::Junior
        }
    }
}

use types::{SourceStatus, User};

#[derive(Debug, DtoFrom)]
#[dto(from = types::User)]
pub struct UserDto {
    /// Direct 1:1 mapping (same name & type)
    pub id: String,

    /// `rename` + `transform_fn`: read from `name` and then apply `types::mask_name`
    #[dto(rename = "name", transform_fn = types::mask_name)]
    pub display_name: String,

    /// Another direct mapping example (primitive type)
    pub age: u32,

    /// Direct mapping of an `Option` (no transform required)
    pub note: Option<String>,

    /// `skip`: do not read from source; initialize with `Default::default()`
    #[dto(skip)]
    pub note2: Option<String>,

    /// `into`: uses `From<SourceStatus> for DtoStatus` (infallible conversion)
    #[dto(into)]
    pub status: types::DtoStatus,

    /// Direct mapping of `Rank` (same enum type on both sides)
    pub rank: types::Rank,

    /// Direct mapping of `Option<DateTime<Utc>>`
    pub created_at: Option<DateTime<Utc>>,
}

fn main() {
    // Build a source model. Note: `password` is present here but intentionally not included in the DTO.
    let user = User {
        id: "u123".to_string(),
        name: "Alice".to_string(),
        age: 30,
        password: "supersecret".to_string(),
        note: Some("note".to_string()),
        status: SourceStatus::Active,
        rank: types::Rank::Senior,
        level: 1,
        created_at: Some(Utc::now()),
    };

    let dto: UserDto = user.into();
    println!("Mapped DTO (owned): {:?}", dto);
}
