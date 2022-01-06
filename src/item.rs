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
    pub hash: &'a str,
    pub name: &'a str,
}

use diesel::prelude::*;
use diesel::result::Error;
use diesel::MysqlConnection;
use transaction::prelude::*;
use transaction_diesel_mysql::{with_conn, DieselContext};

type Ctx<'a> = DieselContext<'a, MysqlConnection>;
type BoxTx<'a, T> = Box<dyn Transaction<Ctx = Ctx<'a>, Item = T, Err = Error> + 'a>;

pub fn create<'a>(new: &'a NewItem) -> BoxTx<'a, Item> {
    use crate::schema::items::table;
    with_conn(move |cn| {
        diesel::insert_into(table).values(new).execute(cn)?;
        table
            .order(crate::schema::items::id.desc())
            .limit(1)
            .first(cn)
    })
    .boxed()
}

pub fn find<'a>(id: i32) -> BoxTx<'a, Option<Item>> {
    use crate::schema::items::dsl::items;
    with_conn(move |cn| items.find(id).get_result(cn).optional()).boxed()
}

pub fn update<'a>(edit: Item) -> BoxTx<'a, Option<()>> {
    use crate::schema::items::dsl;
    with_conn(move |cn| {
        let edit = edit.clone(); // TODO: 本当はclone()したくない
        diesel::update(dsl::items.find(edit.id))
            .set((items::hash.eq(edit.hash), items::name.eq(edit.name)))
            .execute(cn)
            .map(|_| ())
            .optional()
    })
    .boxed()
}

pub fn delete<'a>(id: i32) -> BoxTx<'a, Option<()>> {
    use crate::schema::items::dsl::items;
    with_conn(move |cn| {
        diesel::delete(items.find(id))
            .execute(cn)
            .map(|_| ())
            .optional()
    })
    .boxed()
}
