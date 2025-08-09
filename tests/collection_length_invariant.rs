//!
//! Property tests for collection invariants (length/order/null-safety)
//!

use simple_dto_mapper_derive::DtoFrom;
use proptest::prelude::*;

mod types {
    // ---- source-side types ----
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SourceTag(pub String);

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SourceAuthor {
        pub name: String,
    }

    #[derive(Debug, Clone)]
    pub struct Article {
        pub labels: Vec<SourceTag>,
        pub keywords: Vec<String>,
        pub author: Option<SourceAuthor>,
    }

    // ---- dto-side types ----
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct DtoTag(pub String);

    impl From<SourceTag> for DtoTag {
        fn from(t: SourceTag) -> Self {
            DtoTag(t.0)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct DtoAuthor {
        pub name: String,
    }

    impl From<SourceAuthor> for DtoAuthor {
        fn from(a: SourceAuthor) -> Self {
            DtoAuthor { name: a.name }
        }
    }

    // ---- helpers used by `transform_fn` ----
    pub fn vec_into<T, U>(v: Vec<T>) -> Vec<U>
    where
        U: From<T>,
    {
        v.into_iter().map(Into::into).collect()
    }

    pub fn opt_into<T, U>(o: Option<T>) -> Option<U>
    where
        U: From<T>,
    {
        o.map(Into::into)
    }
}

use types::*;

#[derive(Debug, DtoFrom)]
#[dto(from = types::Article)]
pub struct ArticleDto {
    // Vec<SourceTag> → Vec<DtoTag> via transform_fn
    #[dto(rename = "labels", transform_fn = types::vec_into::<types::SourceTag, types::DtoTag>)]
    pub tags: Vec<DtoTag>,

    // Vec<String> → Vec<String> (same type)
    pub keywords: Vec<String>,

    // Option<SourceAuthor> → Option<DtoAuthor>
    #[dto(transform_fn = types::opt_into::<types::SourceAuthor, types::DtoAuthor>)]
    pub author: Option<DtoAuthor>,
}

proptest! {
    #[test]
    fn length_and_order_preserved(
        labels in proptest::collection::vec(any::<String>(), 0..64),
        keywords in proptest::collection::vec(any::<String>(), 0..64),
        author_name in proptest::option::of(any::<String>()),
    ) {
        // Build source from strategies
        let src = Article {
            labels: labels.clone().into_iter().map(SourceTag).collect(),
            keywords: keywords.clone(),
            author: author_name.clone().map(|n| SourceAuthor { name: n }),
        };

        // Map to DTO
        let dto: ArticleDto = src.clone().into();

        // 1) Length equality for collections
        prop_assert_eq!(dto.tags.len(), labels.len());
        prop_assert_eq!(dto.keywords.len(), keywords.len());

        // 2) Order preserved for tags (element-wise equality after conversion)
        let original: Vec<String> = labels.clone();
        let mapped: Vec<String> = dto.tags.iter().map(|DtoTag(s)| s.clone()).collect();
        prop_assert_eq!(mapped, original);

        // 3) Option null-safety: None stays None; Some stays Some with same payload
        match (author_name, dto.author) {
            (None, None) => {}
            (Some(n), Some(DtoAuthor { name })) => prop_assert_eq!(n, name),
            other => panic!("Option shape changed unexpectedly: {:?}", other),
        }
    }
}
