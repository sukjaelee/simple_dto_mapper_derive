use simple_dto_mapper_derive::DtoFrom;

mod types {
    pub struct Source {
        pub id: String,
    }
}

#[derive(DtoFrom)]
#[dto(from = types::Source, from = types::Source)]
struct Dto {
    id: String,
}

fn main() {}
