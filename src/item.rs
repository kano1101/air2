use crate::schema::*;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash, Queryable)]
pub struct Item {
    pub id: i32,
    pub hash: String,
    pub name: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash, Insertable)]
#[table_name = "items"]
pub struct NewItem<'a> {
    pub id: i32,
    pub hash: &'a str,
    pub name: &'a str,
}
