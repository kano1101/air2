#[macro_use]
extern crate diesel;
extern crate amazon_log;
extern crate dotenv;
// extern crate mdo;
extern crate range;
extern crate tokio;
extern crate transaction;
extern crate transaction_diesel_mysql;

mod business;
mod category;
mod history;
mod item;
mod log;
mod schema;
mod ui;
mod utils;

// use crate::amazon_log::AmazonBrowserResult;

fn main() -> iced::Result {
    use crate::business::{get_all_logs_from_db, get_difference_logs_from_amazon};
    use crate::utils::establish_connection;
    let cn = establish_connection();

    let mut rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        get_difference_logs_from_amazon(&cn).await;
    });

    let db_logs = get_all_logs_from_db(&cn);

    ui::run()
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
