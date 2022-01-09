#[macro_use]
extern crate diesel;
extern crate amazon_log;
extern crate dotenv;
extern crate range;
extern crate tokio;
extern crate transaction;
extern crate transaction_diesel_mysql;

mod business;
mod category;
mod history;
mod item;
mod schema;
mod utils;

#[tokio::main]
async fn main() {
    use crate::business::difference_log;
    let logs = difference_log().await.unwrap();
    logs.iter().for_each(|log| println!("{:?}", log));
    println!("{}個の履歴が見つかりました。", logs.len());

    let max_name_log = logs.iter().max_by_key(|log| log.name.len()).unwrap();
    let max_len = max_name_log.name.len();
    println!("最大のnameバイト数は「{}」です。", max_len);
}

#[cfg(test)]
mod tests {
    use amazon_log::{AmazonBrowser, AmazonBrowserResult};
    use tokio;
    #[ignore]
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
        b.quit().await?;
        assert_eq!(logs.len(), 2);
        Ok(())
    }
}
