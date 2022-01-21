use crate::schema::*;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash, Queryable)]
pub struct Category {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash, Insertable)]
#[table_name = "categories"]
pub struct NewCategory<'a> {
    pub name: &'a str,
}

use diesel::prelude::*;
use diesel::result::Error;
use diesel::MysqlConnection;
use transaction::prelude::*;
use transaction_diesel_mysql::{with_conn, DieselContext};

type Ctx<'a> = DieselContext<'a, MysqlConnection>;
type BoxTx<'a, T> = Box<dyn Transaction<Ctx = Ctx<'a>, Item = T, Err = Error> + 'a>;

// pub fn access<'a, R, F: 'a>(f: F) -> BoxTx<'a, R>
// where
//     F: Fn(&'a MysqlConnection) -> Result<R, Error>,
// {
//     with_conn(f).boxed()
// }

pub fn all<'a>() -> BoxTx<'a, Vec<Category>> {
    use crate::schema::categories::dsl::categories;
    with_conn(move |cn| categories.load::<Category>(cn)).boxed()
}

pub fn create<'a>(new: &'a NewCategory) -> BoxTx<'a, Category> {
    use crate::schema::categories::{id, table};
    with_conn(move |cn| {
        diesel::insert_into(table).values(new).execute(cn)?;
        table.order(id.desc()).limit(1).first(cn)
    })
    .boxed()
}

pub fn find<'a>(id: i32) -> BoxTx<'a, Category> {
    use crate::schema::categories::dsl::categories;
    with_conn(move |cn| categories.find(id).get_result(cn)).boxed()
}

pub fn filter<'a>(name: &'a str) -> BoxTx<'a, Category> {
    use crate::schema::categories::{name, table};
    with_conn(move |cn| table.filter(name.eq(name)).first(cn)).boxed()
}

pub fn update<'a>(edit: Category) -> BoxTx<'a, ()> {
    use crate::schema::categories::dsl;
    with_conn(move |cn| {
        let edit = edit.clone(); // TODO: 本当はclone()したくない
        diesel::update(dsl::categories.find(edit.id))
            .set((categories::name.eq(edit.name),))
            .execute(cn)
            .map(|_| ())
    })
    .boxed()
}

pub fn delete<'a>(id: i32) -> BoxTx<'a, ()> {
    use crate::schema::categories::dsl::categories;
    with_conn(move |cn| diesel::delete(categories.find(id)).execute(cn).map(|_| ())).boxed()
}

#[cfg(test)]
mod tests {
    #[test]
    fn categoryのcrudの確認() {
        use crate::category;
        use crate::category::{Category, NewCategory};
        use crate::utils::establish_connection;
        use transaction::Transaction;

        let conn = establish_connection();

        let new_name = "keen";
        let update_name = "KeenS";

        let new_category = NewCategory { name: new_name };

        let tx = category::create(&new_category).and_then(move |category| {
            assert_eq!(category.name, new_name);
            let edit_category = Category {
                name: update_name.to_string(),
                ..category
            };
            category::update(edit_category).and_then(move |()| {
                category::find(category.id).and_then(move |updated_category| {
                    assert_eq!(updated_category.name, update_name);
                    category::delete(updated_category.id)
                })
            })
        });

        transaction_diesel_mysql::run(&conn, tx).unwrap()
    }
}
