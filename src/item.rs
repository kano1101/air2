use crate::schema::*;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash, Queryable)]
pub struct Item {
    pub id: i32,
    pub category_id: i32,
    pub hash: String,
    pub name: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash, Insertable)]
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

pub fn all<'a>() -> BoxTx<'a, Vec<Item>> {
    use crate::schema::items::dsl::items;
    with_conn(move |cn| items.load::<Item>(cn)).boxed()
}

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
            .set((
                items::category_id.eq(edit.category_id),
                items::hash.eq(edit.hash),
                items::name.eq(edit.name),
            ))
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

#[cfg(test)]
mod tests {
    #[test]
    fn itemのcrudの確認() {
        use crate::item;
        use crate::item::{Item, NewItem};
        use crate::transaction::with_ctx;
        use crate::utils::establish_connection;
        use diesel::result::Error;

        let conn = establish_connection();

        let category_id;

        {
            use crate::category::NewCategory;
            let new_category = NewCategory { name: "CATEGORY" };
            let category_tx = with_ctx(|ctx| -> Result<i32, Error> {
                use crate::category;
                let category = category::create(&new_category).run(ctx)?;
                Ok(category.id)
            });
            category_id = transaction_diesel_mysql::run(&conn, category_tx).unwrap()
        }

        let new_name = "keen";
        let update_name = "KeenS";

        let new_item = NewItem {
            category_id: category_id,
            hash: "0000",
            name: new_name,
        };

        let tx = with_ctx(|ctx| -> Result<(), Error> {
            let item = item::create(&new_item).run(ctx)?;
            assert_ne!(item.id, 0);
            assert_eq!(item.name, new_name);

            let edit_item = Item {
                name: update_name.to_string(),
                ..item
            };
            let res = item::update(edit_item).run(ctx)?;
            match res {
                None => {
                    println!("item not found");
                    return Ok(());
                }
                Some(()) => (),
            };
            let updated_item = match item::find(item.id).run(ctx)? {
                None => {
                    println!("item not found");
                    return Ok(());
                }
                Some(u) => u,
            };
            assert_eq!(updated_item.name, update_name);

            use crate::category;
            let delete_item = updated_item;
            let delete_category = category::find(delete_item.category_id).run(ctx)?.unwrap();
            item::delete(delete_item.id).run(ctx)?;
            category::delete(delete_category.id).run(ctx)?;

            Ok(())
        });
        transaction_diesel_mysql::run(&conn, tx).unwrap()
    }
}
