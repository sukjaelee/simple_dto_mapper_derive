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
    // Build a source model. Note: `password` is present here but intentionally not included in the DTO.

    // Example 1: Using `Into` — converts a `User` into `UserDto` via the derived `impl From<User> for UserDto`.
    // This moves the `user` instance into the conversion.
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
    println!("Mapped DTO (into): {:?}", dto);

    // Example 2: Using `UserDto::from` — alternative explicit call to the generated `From<User>` implementation.
    // This approach makes it clear which type we are converting into.
    let other_user = User {
        id: "u456".to_string(),
        name: "Bob".to_string(),
        age: 25,
        password: "topsecret".to_string(),
        note: None,
        status: SourceStatus::Inactive,
        rank: types::Rank::Junior,
        level: 0,
        created_at: None,
    };

    let dto_from = UserDto::from(other_user);
    println!("Mapped DTO (from): {:?}", dto_from);
}
```

## Collections & Option

Collections and Option do not auto-convert inner elements. Use a transform_fn helper or implement From&lt;Vec&lt;T&gt;&gt; for Vec&lt;U&gt; if you prefer #[dto(into)].

```rust
// same type, no transform needed
pub keywords: Vec<String>,

// Vec<SourceTag> → Vec<DtoTag> using a transform function
#[dto(rename = "labels", transform_fn = types::vec_into::<types::SourceTag, types::DtoTag>)]
pub tags: Vec<DtoTag>,

// Option<SourceAuthor> → Option<DtoAuthor>
#[dto(transform_fn = types::opt_into::<types::SourceAuthor, types::DtoAuthor>)]
pub author: Option<DtoAuthor>,
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
simple_dto_mapper_derive = "0.1.1"
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
