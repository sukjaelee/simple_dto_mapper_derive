# simple_dto_mapper_derive

`simple_dto_mapper_derive` provides a custom derive macro `DtoFrom` that generates
`impl From<Source> for Target` to map model structs into DTOs. It focuses on a
small, explicit set of attributes for clarity and reliability.

## Features

- **Source struct**: `#[dto(from = Type)]` (required)
- **Field rename**: `#[dto(rename = "orig_name")]`
- **Custom transform**: `#[dto(transform_fn = path::to::function)]` (owned in → owned out)
- **Skip with default**: `#[dto(skip)]`
- **Infallible convert**: `#[dto(into)]` (uses `From<Src> for Dst`)
- **Clear diagnostics**: unknown/duplicate/conflicting attributes produce span-accurate compile errors (see `tests/ui`)

## Mapping Rules (at a glance)

- **Default (owned move)**  
  Same **name** & same **type** → `target = source.field`  
  “Compatible type” means:

  - Identical type, or
  - `#[dto(into)]` where `From<SourceFieldType> for FieldType` exists, or
  - `#[dto(transform_fn = ...)]` provides an explicit conversion

- **Field attributes**

  - `#[dto(rename = "orig_name")]` — read from another source field name
  - `#[dto(transform_fn = path)]` — call `path(source.orig_name)` before assignment
  - `#[dto(skip)]` — initialize with `Default::default()`
  - `#[dto(into)]` — call `Into::into(source.orig_name)` (requires `From`)

- **Struct attribute (required)**  
  `#[dto(from = Type)]` — specify the **source struct**.

## Usage

```rust
use simple_dto_mapper_derive::DtoFrom;

mod types {
    #[derive(Debug)]
    pub struct User {
        pub id: String,
        pub name: String,
        pub age: u32,
        pub password: String, // intentionally not mapped
        pub status: SourceStatus,
    }

    #[derive(Debug, Clone)]
    pub enum SourceStatus { Active, Inactive }

    #[derive(Debug, Clone)]
    pub enum DtoStatus { Active, Inactive }

    // For `#[dto(into)]`
    impl From<SourceStatus> for DtoStatus {
        fn from(s: SourceStatus) -> Self {
            match s { SourceStatus::Active => Self::Active, SourceStatus::Inactive => Self::Inactive }
        }
    }

    // For `#[dto(transform_fn = ...)]`
    pub fn mask_name(name: String) -> String {
        let ch = name.chars().next().unwrap_or('*');
        format!("{ch}***")
    }
}

#[derive(DtoFrom, Debug)]
#[dto(from = types::User)]
struct UserDto {
    // `rename` + `transform_fn`
    #[dto(rename = "name", transform_fn = types::mask_name)]
    display_name: String,

    // Direct mapping
    age: u32,

    // skip: default initialize
    #[dto(skip)]
    note: Option<String>,

    // `into`: uses `From<SourceStatus> for DtoStatus`
    #[dto(into)]
    status: types::DtoStatus,
}

fn main() {
    let user = types::User {
        id: "u1".into(),
        name: "Alice".into(),
        age: 30,
        password: "secret".into(),
        status: types::SourceStatus::Active,
    };
    let dto: UserDto = user.into();
    println!("{dto:?}");
}
```

## Collections & Option

Collections and Option do not auto-convert inner elements. Use a transform_fn helper or implement From<Vec<T>> for Vec<U> if you prefer #[dto(into)].

```rust
// 1) Vec<String> → Vec<String> (same type, no transform needed)
pub keywords: Vec<String>,

// 2) Vec<SourceTag> → Vec<DtoTag> using a transform function
#[dto(rename = "labels", transform_fn = types::vec_into::<types::SourceTag, types::DtoTag>)]
pub tags: Vec<DtoTag>,

// 3) Option<SourceAuthor> → Option<DtoAuthor>
#[dto(transform_fn = types::opt_into::<types::SourceAuthor, types::DtoAuthor>)]
pub author: Option<DtoAuthor>,
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
simple_dto_mapper_derive = "0.1.0"
```

## Diagnostics & Limitations

- Named-field structs only (tuple/unit structs & enums are not supported)
- Owned-only mapping (`impl From<Source> for Target`)
- `transform_fn` must be `FnOnce(SrcField) -> DstField`
- `into` requires `From<SrcField> for DstField`
- `skip` requires `Default`
- Clear errors for unknown/duplicate/conflicting attributes; see `tests/ui`

## License

Licensed under the MIT license.
