use crate::schema::*;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash, Queryable)]
pub struct Category {
    pub id: i32,
    pub name: String,
}

// 本当はCopyトレイトを使いたくない
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Default, Hash, Insertable)]
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

pub fn test_init<'a>() -> BoxTx<'a, Category> {
    use crate::schema::categories::{id, name, table};
    with_conn(move |cn| {
        let maybe_category: Result<Category, Error> =
            table.filter(name.eq("None Category")).first(cn);
        let category = maybe_category.or_else(|_| {
            let initial_category = NewCategory {
                name: "None Category",
            };

            diesel::insert_into(table)
                .values(&initial_category)
                .execute(cn)?;
            table.order(id.desc()).limit(1).first(cn)
        });
        category
    })
    .boxed()
}

pub fn all<'a>() -> BoxTx<'a, Vec<Category>> {
    use crate::schema::categories::dsl::categories;
    with_conn(move |cn| categories.load::<Category>(cn)).boxed()
}

pub fn create<'a>(new: NewCategory<'a>) -> BoxTx<'a, Category> {
    use crate::schema::categories::{id, table};
    with_conn(move |cn| {
        let new = new.clone(); // TODO: 本当はclone()したくない
        diesel::insert_into(table).values(&new).execute(cn)?;
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
        use transaction::with_ctx;

        let cn = establish_connection();

        let new_name = "keen";
        let update_name = "KeenS";

        let new_category = NewCategory { name: new_name };

        let tx = with_ctx(|ctx| {
            let category = category::create(new_category).run(ctx)?;
            assert_eq!(category.name, new_name);
            let edit_category = Category {
                name: update_name.to_string(),
                ..category
            };
            category::update(edit_category).run(ctx)?;
            let updated_category = category::find(category.id).run(ctx)?;
            assert_eq!(updated_category.name, update_name);
            let delete_category = updated_category;
            category::delete(delete_category.id).run(ctx)
        });
        transaction_diesel_mysql::run(&cn, tx).unwrap()
    }
}
