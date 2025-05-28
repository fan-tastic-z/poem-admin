use modql::SIden;
use sea_query::{Iden, IntoIden, TableRef};

pub trait Dao {
    const TABLE: &'static str;
    fn table_ref() -> TableRef {
        TableRef::Table(SIden(Self::TABLE).into_iden())
    }
}

#[derive(Iden)]
pub enum CommonIden {
    Id,
}
