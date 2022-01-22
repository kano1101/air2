use crate::schema::*;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash, Queryable)]
pub struct History {
    pub id: i32,
    pub item_id: i32,
    pub price: i32,
    pub purchased_at: String,
}

// 本当はCopyトレイトを使いたくない
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Default, Hash, Insertable)]
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

pub fn all<'a>() -> BoxTx<'a, Vec<History>> {
    use crate::schema::histories::dsl::histories;
    with_conn(move |cn| histories.load::<History>(cn)).boxed()
}

pub fn create<'a>(new: NewHistory<'a>) -> BoxTx<'a, History> {
    use crate::schema::histories::table;
    with_conn(move |cn| {
        let new = new.clone(); // TODO: 本当はclone()したくない
        diesel::insert_into(table).values(new).execute(cn)?;
        table
            .order(crate::schema::histories::id.desc())
            .limit(1)
            .first(cn)
    })
    .boxed()
}

pub fn find<'a>(id: i32) -> BoxTx<'a, History> {
    use crate::schema::histories::dsl::histories;
    with_conn(move |cn| histories.find(id).get_result(cn)).boxed()
}

pub fn update<'a>(edit: History) -> BoxTx<'a, ()> {
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
    })
    .boxed()
}

pub fn delete<'a>(id: i32) -> BoxTx<'a, ()> {
    use crate::schema::histories::dsl::histories;
    with_conn(move |cn| diesel::delete(histories.find(id)).execute(cn).map(|_| ())).boxed()
}

#[cfg(test)]
mod tests {
    #[test]
    fn historyのcrudの確認() {
        use crate::history;
        use crate::history::{History, NewHistory};
        use crate::transaction::with_ctx;
        use crate::utils::establish_connection;

        let cn = establish_connection();

        let tx = with_ctx(|ctx| {
            let category = crate::category::test_init().run(ctx);
            let category_id = category.expect("None Categoryが見つかりません。").id;

            let item = crate::item::test_init(category_id).run(ctx);
            let item_id = item.expect("Itemの読み込みでエラーが発生しました。").id;

            let new_price = 42;
            let update_price = 35;

            let new_history = NewHistory {
                item_id: item_id,
                price: new_price,
                purchased_at: "2021-10-01",
            };

            let history = history::create(new_history).run(ctx)?;
            assert_eq!(history.price, new_price);

            let edit_history = History {
                price: update_price,
                ..history
            };
            history::update(edit_history).run(ctx)?;

            let updated_history = history::find(history.id).run(ctx)?;
            assert_eq!(updated_history.price, update_price);

            let delete_history = updated_history;
            let delete_item = crate::item::find(delete_history.item_id).run(ctx)?;
            let delete_category = crate::category::find(delete_item.category_id).run(ctx)?;
            crate::history::delete(delete_history.id).run(ctx)?;
            crate::item::delete(delete_item.id).run(ctx)?;
            crate::category::delete(delete_category.id).run(ctx)
        });
        transaction_diesel_mysql::run(&cn, tx).unwrap()
    }
}
