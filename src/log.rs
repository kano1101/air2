use crate::schema::*;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash, Queryable)]
pub struct Log {
    pub id: i32,
    pub hash: String,
    pub name: String,
    pub price: i32,
    pub purchased_at: String,
}

// 本当はCopyトレイトを使いたくない
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Default, Hash, Insertable)]
#[table_name = "logs"]
pub struct NewLog<'a> {
    pub hash: &'a str,
    pub name: &'a str,
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

pub fn all<'a>() -> BoxTx<'a, Vec<Log>> {
    use crate::schema::logs::dsl::logs;
    with_conn(move |cn| logs.load::<Log>(cn)).boxed()
}

pub fn create<'a>(new: NewLog<'a>) -> BoxTx<'a, Log> {
    use crate::schema::logs::table;
    with_conn(move |cn| {
        let new = new.clone(); // TODO: 本当はclone()したくない
        diesel::insert_into(table).values(&new).execute(cn)?;
        table
            .order(crate::schema::logs::id.desc())
            .limit(1)
            .first(cn)
    })
    .boxed()
}

pub fn find<'a>(id: i32) -> BoxTx<'a, Log> {
    use crate::schema::logs::dsl::logs;
    with_conn(move |cn| logs.find(id).get_result(cn)).boxed()
}

pub fn update<'a>(edit: Log) -> BoxTx<'a, ()> {
    use crate::schema::logs::dsl;
    with_conn(move |cn| {
        let edit = edit.clone(); // TODO: 本当はclone()したくない
        diesel::update(dsl::logs.find(edit.id))
            .set((
                logs::hash.eq(edit.hash),
                logs::name.eq(edit.name),
                logs::price.eq(edit.price),
                logs::purchased_at.eq(edit.purchased_at),
            ))
            .execute(cn)
            .map(|_| ())
    })
    .boxed()
}

pub fn delete<'a>(id: i32) -> BoxTx<'a, ()> {
    use crate::schema::logs::dsl::logs;
    with_conn(move |cn| diesel::delete(logs.find(id)).execute(cn).map(|_| ())).boxed()
}

#[cfg(test)]
mod tests {
    #[test]
    fn logのcrudの確認() {
        use crate::log;
        use crate::log::{Log, NewLog};
        use crate::utils::establish_connection;
        use transaction::with_ctx;

        let cn = establish_connection();

        let tx = with_ctx(|ctx| {
            let new_price = 42;
            let update_price = 35;

            let new_log = NewLog {
                hash: "hash",
                name: "name",
                price: new_price,
                purchased_at: "2021-10-01",
            };

            let log = log::create(new_log).run(ctx)?;
            assert_eq!(log.price, new_price);

            let edit_log = Log {
                price: update_price,
                ..log
            };
            log::update(edit_log).run(ctx)?;

            let updated_log = log::find(log.id).run(ctx)?;
            assert_eq!(updated_log.price, update_price);

            let delete_log = updated_log;
            log::delete(delete_log.id).run(ctx)
        });
        transaction_diesel_mysql::run(&cn, tx).unwrap()
    }
}
