//! Collections mapping example
//!
//! This example demonstrates how to handle collection/optional field conversions
//! **without extra macro features** — using only the current attributes:
//! `rename`, `transform_fn`, `skip`, and `into`.

use chrono::{DateTime, NaiveDateTime, Utc};
use simple_dto_mapper_derive::DtoFrom;

mod types {
    use super::*;

    // --------- Source types ---------
    #[derive(Debug)]
    pub struct SourceTag(pub String);

    #[derive(Debug)]
    pub struct SourceAuthor {
        pub name: String,
    }

    #[derive(Debug)]
    pub struct Article {
        pub id: String,
        pub title: String,
        pub keywords: Vec<String>,        // -> Vec<String> (same type)
        pub labels: Vec<SourceTag>,       // -> Vec<DtoTag> (via transform_fn)
        pub author: Option<SourceAuthor>, // -> Option<DtoAuthor> (via transform_fn)
        pub published_at: Option<NaiveDateTime>, // -> Option<DateTime<Utc>> (via transform_fn)
    }

    // --------- DTO-side types ---------
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct DtoTag(pub String);

    impl From<SourceTag> for DtoTag {
        fn from(s: SourceTag) -> Self {
            DtoTag(s.0)
        }
    }

    #[derive(Debug, Clone)]
    pub struct DtoAuthor {
        pub name: String,
    }

    impl From<SourceAuthor> for DtoAuthor {
        fn from(a: SourceAuthor) -> Self {
            DtoAuthor { name: a.name }
        }
    }

    // --------- Helper transforms (owned) ---------
    /// Vec<T> → Vec<U> using Into
    pub fn vec_into<T, U>(v: Vec<T>) -> Vec<U>
    where
        U: From<T>,
    {
        v.into_iter().map(Into::into).collect()
    }

    /// Option<T> → Option<U> using Into
    pub fn opt_into<T, U>(o: Option<T>) -> Option<U>
    where
        U: From<T>,
    {
        o.map(Into::into)
    }

    /// Option<NaiveDateTime> → Option<DateTime<Utc>>
    pub fn opt_naive_to_utc(o: Option<NaiveDateTime>) -> Option<DateTime<Utc>> {
        o.map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
    }
}

use types::*;

#[derive(Debug, DtoFrom)]
#[dto(from = types::Article)]
pub struct ArticleDto {
    // 1) Direct mapping (same type)
    pub id: String,

    // 2) Rename + direct mapping
    #[dto(rename = "title")]
    pub headline: String,

    // 3) Vec<String> → Vec<String> (same type, no transform needed)
    pub keywords: Vec<String>,

    // 4) Vec<SourceTag> → Vec<DtoTag> using a transform function
    #[dto(rename = "labels", transform_fn = types::vec_into::<types::SourceTag, types::DtoTag>)]
    pub tags: Vec<DtoTag>,

    // 5) Option<SourceAuthor> → Option<DtoAuthor>
    #[dto(transform_fn = types::opt_into::<types::SourceAuthor, types::DtoAuthor>)]
    pub author: Option<DtoAuthor>,

    // 6) Option<NaiveDateTime> → Option<DateTime<Utc>>
    #[dto(rename = "published_at", transform_fn = types::opt_naive_to_utc)]
    pub published_at: Option<DateTime<Utc>>,

    // 7) Example of skip on a collection (defaults to empty)
    #[dto(skip)]
    pub notes: Vec<String>,
}

fn main() {
    let src = Article {
        id: "a-001".to_string(),
        title: "Hello Collections".to_string(),
        keywords: vec!["rust".into(), "derive".into()],
        labels: vec![SourceTag("tag-1".into()), SourceTag("tag-2".into())],
        author: Some(SourceAuthor {
            name: "Alice".into(),
        }),
        published_at: Some(Utc::now().naive_utc()),
    };

    let dto: ArticleDto = src.into();
    println!("DTO: {:?}", dto);
}
