extern crate range;
extern crate rstest;
extern crate transaction_diesel_mysql;

mod amazon_browser;
mod item;

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use rstest::*;

    #[test]
    fn 初期動作確認() {
        assert_eq!(1, 1);
    }
}
