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
    use amazon_log::{AmazonBrowser, AmazonBrowserResult, Log};
    use tokio;
    #[tokio::test]
    async fn amazon_logチェック() -> AmazonBrowserResult<()> {
        use dotenv::dotenv;
        use range::Range;
        use std::env;
        dotenv().ok();
        let email = env::var("AMAZON_EMAIL").expect("AMAZON_EMAIL must be set");
        let pass = env::var("AMAZON_PASSWORD").expect("AMAZON_PASSWORD must be set");
        let mut b = AmazonBrowser::new(&email, &pass, "air2").await?;
        let r = Range::new("2021-11-08", "2021-10-21");
        let logs = b.extract(&r).await?;
        assert_eq!(logs.len(), 2);
        Ok(())
    }
}
