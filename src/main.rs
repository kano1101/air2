#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate range;
extern crate rstest;
extern crate transaction;
extern crate transaction_diesel_mysql;

mod amazon_browser;
mod item;
mod schema;

fn main() {
    println!("Hello, world!");
}

use diesel::MysqlConnection;

pub fn establish_connection() -> MysqlConnection {
    use diesel::prelude::*;
    use dotenv::dotenv;
    use std::env;

    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

#[cfg(test)]
mod tests {
    use rstest::*;

    #[test]
    fn 初期動作確認() {
        assert_eq!(1, 1);
    }
    #[test]
    fn crudの確認() {
        use super::establish_connection;
        use crate::item;
        use crate::item::{Item, NewItem};
        use crate::transaction::with_ctx;
        use diesel::result::Error;

        let new_name = "keen";
        let update_name = "KeenS";

        let new_item = NewItem {
            hash: "0000",
            name: new_name,
        };

        let conn = establish_connection();

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

            println!("updated item: {:?}", updated_item);
            match item::delete(updated_item.id).run(ctx)? {
                None => {
                    println!("item not found");
                }
                Some(()) => (),
            };
            Ok(())
        });
        transaction_diesel_mysql::run(&conn, tx).unwrap()
    }
}
