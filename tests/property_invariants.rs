//! Property-based tests to check invariants for `into` + `transform_fn` mappings.
//! Invariants covered: length preservation, order preservation, and null-safety.

use simple_dto_mapper_derive::DtoFrom;
use proptest::prelude::*;

mod types {
    // ----- source-side types -----
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum SourceRank {
        Low,
        High,
        Custom(i32),
    }

    #[derive(Debug, Clone)]
    pub struct Source {
        pub labels: Vec<String>,    // direct mapping (same type)
        pub ranks: Vec<SourceRank>, // element mapping via Into (through transform_fn helper)
        pub maybe: Option<String>,  // option mapping via transform_fn
    }

    // ----- dto-side types -----
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum DtoRank {
        Low,
        High,
        Custom(i32),
    }

    impl From<SourceRank> for DtoRank {
        fn from(s: SourceRank) -> Self {
            match s {
                SourceRank::Low => DtoRank::Low,
                SourceRank::High => DtoRank::High,
                SourceRank::Custom(n) => DtoRank::Custom(n),
            }
        }
    }

    // ----- helpers used by `transform_fn` -----
    /// Vec<T> -> Vec<U> using Into
    pub fn vec_into<T, U>(v: Vec<T>) -> Vec<U>
    where
        U: From<T>,
    {
        v.into_iter().map(Into::into).collect()
    }

    /// Option<String> -> Option<usize> (length), preserving None
    pub fn opt_len(o: Option<String>) -> Option<usize> {
        o.map(|s| s.len())
    }
}

use types::*;

#[derive(Debug, DtoFrom)]
#[dto(from = types::Source)]
pub struct Dto {
    // direct (same type)
    pub labels: Vec<String>,

    // Vec<SourceRank> -> Vec<DtoRank> via transform helper + Into
    #[dto(rename = "ranks", transform_fn = types::vec_into::<types::SourceRank, types::DtoRank>)]
    pub ranks: Vec<DtoRank>,

    // Option<String> -> Option<usize>
    #[dto(rename = "maybe", transform_fn = types::opt_len)]
    pub maybe_len: Option<usize>,
}

// Strategy for ranks: build from a vector of i32 and then wrap as Custom variants.
fn ranks_from_ints(data: Vec<i32>) -> Vec<SourceRank> {
    data.into_iter().map(SourceRank::Custom).collect()
}

proptest! {
    #[test]
    fn invariants_hold(
        labels in proptest::collection::vec(any::<String>(), 0..64),
        ranks_data in proptest::collection::vec(any::<i32>(), 0..64),
        maybe in proptest::option::of(any::<String>()),
    ) {
        let src = Source {
            labels: labels.clone(),
            ranks: ranks_from_ints(ranks_data.clone()),
            maybe: maybe.clone(),
        };

        let dto: Dto = src.into();

        // 1) length preservation
        prop_assert_eq!(dto.labels.len(), labels.len());
        prop_assert_eq!(dto.ranks.len(), ranks_data.len());

        // 2) order preservation for ranks
        let expected: Vec<DtoRank> = ranks_data.into_iter().map(DtoRank::Custom).collect();
        prop_assert_eq!(dto.ranks, expected);

        // 3) null-safety for Option mapping
        match (maybe, dto.maybe_len) {
            (None, None) => {}
            (Some(s), Some(n)) => prop_assert_eq!(s.len(), n),
            other => panic!("Option shape changed unexpectedly: {:?}", other),
        }
    }
}
