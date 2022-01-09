use crate::schema::*;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash, Queryable)]
pub struct History {
    pub id: i32,
    pub item_id: i32,
    pub price: i32,
    pub purchased_at: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash, Insertable)]
#[table_name = "histories"]
pub struct NewHistory<'a> {
    pub item_id: i32,
    pub price: i32,
    pub purchased_at: &'a str,
}

use diesel::prelude::*;
use diesel::result::Error;
use diesel::MysqlConnection;
use transaction::prelude::*;
use transaction_diesel_mysql::{with_conn, DieselContext};

type Ctx<'a> = DieselContext<'a, MysqlConnection>;
type BoxTx<'a, T> = Box<dyn Transaction<Ctx = Ctx<'a>, Item = T, Err = Error> + 'a>;

pub fn create<'a>(new: &'a NewHistory) -> BoxTx<'a, History> {
    use crate::schema::histories::table;
    with_conn(move |cn| {
        diesel::insert_into(table).values(new).execute(cn)?;
        table
            .order(crate::schema::histories::id.desc())
            .limit(1)
            .first(cn)
    })
    .boxed()
}

pub fn find<'a>(id: i32) -> BoxTx<'a, Option<History>> {
    use crate::schema::histories::dsl::histories;
    with_conn(move |cn| histories.find(id).get_result(cn).optional()).boxed()
}

pub fn update<'a>(edit: History) -> BoxTx<'a, Option<()>> {
    use crate::schema::histories::dsl;
    with_conn(move |cn| {
        let edit = edit.clone(); // TODO: 本当はclone()したくない
        diesel::update(dsl::histories.find(edit.id))
            .set((
                histories::item_id.eq(edit.item_id),
                histories::price.eq(edit.price),
                histories::purchased_at.eq(edit.purchased_at),
            ))
            .execute(cn)
            .map(|_| ())
            .optional()
    })
    .boxed()
}

pub fn delete<'a>(id: i32) -> BoxTx<'a, Option<()>> {
    use crate::schema::histories::dsl::histories;
    with_conn(move |cn| {
        diesel::delete(histories.find(id))
            .execute(cn)
            .map(|_| ())
            .optional()
    })
    .boxed()
}

#[cfg(test)]
mod tests {
    #[test]
    fn historyのcrudの確認() {
        use crate::history;
        use crate::history::{History, NewHistory};
        use crate::transaction::with_ctx;
        use crate::utils::establish_connection;
        // use chrono::{Duration, Local};
        use diesel::result::Error;

        let conn = establish_connection();

        let item_id;

        {
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

            use crate::item::NewItem;
            let new_item = NewItem {
                category_id: category_id,
                hash: "1000",
                name: "Aqun",
            };
            let item_tx = with_ctx(|ctx| -> Result<i32, Error> {
                use crate::item;
                let item = item::create(&new_item).run(ctx)?;
                Ok(item.id)
            });
            item_id = transaction_diesel_mysql::run(&conn, item_tx).unwrap()
        }

        let new_price = 42;
        let update_price = 35;

        let new_history = NewHistory {
            item_id: item_id,
            price: new_price,
            purchased_at: "2021-10-01",
        };

        let tx = with_ctx(|ctx| -> Result<(), Error> {
            let history = history::create(&new_history).run(ctx)?;
            assert_ne!(history.id, 0);
            assert_eq!(history.price, new_price);

            let edit_history = History {
                price: update_price,
                ..history
            };
            let res = history::update(edit_history).run(ctx)?;
            match res {
                None => {
                    println!("history not found");
                    return Ok(());
                }
                Some(()) => (),
            };
            let updated_history = match history::find(history.id).run(ctx)? {
                None => {
                    println!("history not found");
                    return Ok(());
                }
                Some(u) => u,
            };
            assert_eq!(updated_history.price, update_price);

            use crate::category;
            use crate::item;
            let delete_history = updated_history;
            let delete_item = item::find(delete_history.item_id).run(ctx)?.unwrap();
            let delete_category = category::find(delete_item.category_id).run(ctx)?.unwrap();
            history::delete(delete_history.id).run(ctx)?;
            item::delete(delete_item.id).run(ctx)?;
            category::delete(delete_category.id).run(ctx)?;

            Ok(())
        });
        transaction_diesel_mysql::run(&conn, tx).unwrap()
    }
}
