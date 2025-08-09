use simple_dto_mapper_derive::DtoFrom;

mod types {
    pub struct Source {
        pub id: String,
    }
}

#[derive(DtoFrom)]
#[dto(from = types::Source)]
struct Dto {
    // Typo: rename → renmae
    #[dto(renmae = "id")]
    id: String,
}

fn main() {}
