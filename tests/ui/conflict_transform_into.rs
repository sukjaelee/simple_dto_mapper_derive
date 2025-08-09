use simple_dto_mapper_derive::DtoFrom;

mod types {
    pub struct Source {
        pub name: String,
    }
    pub fn upper(s: String) -> String {
        s.to_uppercase()
    }
}

#[derive(DtoFrom)]
#[dto(from = types::Source)]
struct Dto {
    #[dto(rename = "name", transform_fn = types::upper, into)]
    display: String,
}

fn main() {}
