#[macro_use]
extern crate diesel;
extern crate amazon_log;
extern crate dotenv;
extern crate range;
extern crate rstest;
extern crate transaction;
extern crate transaction_diesel_mysql;

mod history;
mod item;
mod schema;
mod utils;

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn 初期動作確認() {
        assert_eq!(1, 1);
    }
}
