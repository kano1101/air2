use crate::schema::*;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash, Queryable)]
pub struct Item {
    pub id: i32,
    pub category_id: i32,
    pub hash: String,
    pub name: String,
}

// 本当はCopyトレイトを使いたくない
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Default, Hash, Insertable)]
#[table_name = "items"]
pub struct NewItem<'a> {
    pub category_id: i32,
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

pub fn test_init<'a>(category_id: i32) -> BoxTx<'a, Item> {
    use crate::schema::items::{id, table};
    with_conn(move |cn| {
        let len = table.load::<Item>(cn).expect("item::test_init").len();
        if len == 0 {
            let initial_item = NewItem {
                category_id: category_id,
                hash: "1000",
                name: "Initial Item",
            };
            diesel::insert_into(items::table)
                .values(&initial_item)
                .execute(cn)?;
        }
        items::table.order(id.desc()).limit(1).first(cn)
    })
    .boxed()
}

pub fn all<'a>() -> BoxTx<'a, Vec<Item>> {
    use crate::schema::items::dsl::items;
    with_conn(move |cn| items.load::<Item>(cn)).boxed()
}

pub fn create<'a>(new: NewItem<'a>) -> BoxTx<'a, Item> {
    use crate::schema::items::table;
    with_conn(move |cn| {
        let new = new.clone(); // TODO: 本当はclone()したくない
        diesel::insert_into(table).values(&new).execute(cn)?;
        table
            .order(crate::schema::items::id.desc())
            .limit(1)
            .first(cn)
    })
    .boxed()
}

pub fn find<'a>(id: i32) -> BoxTx<'a, Item> {
    use crate::schema::items::dsl::items;
    with_conn(move |cn| items.find(id).get_result(cn)).boxed()
}

pub fn update<'a>(edit: Item) -> BoxTx<'a, ()> {
    use crate::schema::items::dsl;
    with_conn(move |cn| {
        let edit = edit.clone(); // TODO: 本当はclone()したくない
        diesel::update(dsl::items.find(edit.id))
            .set((
                items::category_id.eq(edit.category_id),
                items::hash.eq(edit.hash),
                items::name.eq(edit.name),
            ))
            .execute(cn)
            .map(|_| ())
    })
    .boxed()
}

pub fn delete<'a>(id: i32) -> BoxTx<'a, ()> {
    use crate::schema::items::dsl::items;
    with_conn(move |cn| diesel::delete(items.find(id)).execute(cn).map(|_| ())).boxed()
}

#[cfg(test)]
mod tests {
    #[test]
    fn itemのcrudの確認() {
        use crate::item;
        use crate::item::{Item, NewItem};
        use crate::utils::establish_connection;
        use transaction::with_ctx;

        let cn = establish_connection();

        let tx = with_ctx(|ctx| {
            let category = crate::category::test_init().run(ctx);
            let category_id = category.expect("None Categoryが見つかりません。").id;

            let new_name = "keen";
            let update_name = "KeenS";

            let new_item = NewItem {
                category_id: category_id,
                hash: "0000",
                name: new_name,
            };

            let item = item::create(new_item).run(ctx)?;
            assert_eq!(item.name, new_name);
            let edit_item = Item {
                name: update_name.to_string(),
                ..item
            };
            item::update(edit_item).run(ctx)?;
            let updated_item = item::find(item.id).run(ctx)?;
            assert_eq!(updated_item.name, update_name);

            let delete_item = updated_item;
            let delete_category = crate::category::find(delete_item.category_id).run(ctx)?;
            crate::item::delete(delete_item.id).run(ctx);
            crate::category::delete(delete_category.id).run(ctx)
        });
        transaction_diesel_mysql::run(&cn, tx).unwrap()
    }
}
