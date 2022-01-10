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
mod log;
mod schema;
mod utils;

use crate::amazon_log::AmazonBrowserResult;

#[tokio::main]
async fn main() -> AmazonBrowserResult<()> {
    use crate::business::{most_formerly_date, most_recently_log, next_day, yesterday};
    use crate::log::Log;
    use crate::utils::establish_connection;
    use range::Range;
    use transaction::with_ctx;

    let most_formerly_date = &most_formerly_date().await?;
    assert_eq!(most_formerly_date, "2018-01-01");

    let cn = establish_connection();
    let tx = with_ctx(|ctx| -> Result<Log, _> { most_recently_log().run(ctx) });

    let diff_range = match transaction_diesel_mysql::run(&cn, tx) {
        Ok(log) => Range::new(&yesterday(), &next_day(log.purchased_at)),
        Err(_) => Range::new(&yesterday(), most_formerly_date),
    };

    println!("start: {}, end: {}", diff_range.start(), diff_range.end());

    use crate::business::difference_log;
    let logs = difference_log(diff_range).await.unwrap();
    logs.iter().for_each(|log| println!("{:?}", log));
    println!("{}個の履歴が見つかりました。", logs.len());

    let max_len = logs.iter().map(|log| log.name.len()).max().unwrap_or(0);
    println!("最大のnameバイト数は「{}」です。", max_len);

    use crate::log::NewLog;
    let new_logs = logs
        .iter()
        .map(|log| NewLog {
            hash: &log.hash,
            name: &log.name,
            price: log.price,
            purchased_at: &log.purchased_at,
        })
        .collect::<Vec<NewLog>>();

    use crate::log;
    use diesel::result::Error;
    let tx = with_ctx(|ctx| -> Result<(), Error> {
        for new_log in new_logs.iter() {
            // TODO: 言語仕様が不明なためメソッドチェーンは使わず普通のfor文で妥協
            log::create(&new_log).run(ctx)?;
        }
        Ok(())
    });
    transaction_diesel_mysql::run(&cn, tx).unwrap();
    Ok(())
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
