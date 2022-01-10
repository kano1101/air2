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
    use transaction::with_ctx;

    let most_formerly_date = &most_formerly_date().await?;
    assert_eq!(most_formerly_date, "2018-01-01");

    use crate::utils::establish_connection;
    let cn = establish_connection();

    // DB内の最新情報とブラウザ上の最古の情報からスクレイピング範囲を決定
    use range::Range;
    let tx = with_ctx(|ctx| -> Result<log::Log, _> { most_recently_log().run(ctx) });
    let diff_range = match transaction_diesel_mysql::run(&cn, tx) {
        Ok(log) => Range::new(&yesterday(), &next_day(log.purchased_at)),
        Err(_) => Range::new(&yesterday(), most_formerly_date),
    };
    println!("start: {}\nend  : {}", diff_range.start(), diff_range.end());

    // ブラウザから、決定した範囲の履歴(amazon_log::Log)をスクレイピングして取得
    use amazon_log;
    let amazon_logs: Vec<amazon_log::Log> = difference_log(diff_range).await?;
    amazon_logs.iter().for_each(|log| println!("{:?}", log));
    println!(
        "{}個の追加が必要な履歴が見つかりました。",
        amazon_logs.len()
    );

    // amazon_log::Logをlog::Logに変換(厳密にはDBに保存したらlog::Logになるので今はlog::NewLog)
    use crate::log::NewLog;
    let new_logs: Vec<NewLog> = amazon_logs
        .iter()
        .map(|log| NewLog {
            hash: &log.hash,
            name: &log.name,
            price: log.price,
            purchased_at: &log.purchased_at,
        })
        .collect();

    use diesel::result::Error;
    let tx = with_ctx(|ctx| -> Result<(), Error> {
        for new_log in new_logs.iter() {
            // TODO: 言語仕様が不明なためメソッドチェーンは使わず普通のfor文で妥協
            log::create(&new_log).run(ctx)?;
        }
        Ok(())
    });
    transaction_diesel_mysql::run(&cn, tx).unwrap();

    // DB内の全ログ(log::Log)を取得
    use crate::business::difference_log;
    use crate::log;
    let tx = with_ctx(|ctx| log::all().run(ctx));
    let db_logs = transaction_diesel_mysql::run(&cn, tx).unwrap_or(vec![]);

    let max_len = db_logs.iter().map(|log| log.name.len()).max().unwrap_or(0);
    println!("最大のnameバイト数は「{}」です。", max_len);

    let logs_count = db_logs.len();
    println!("取扱件数は{}件です。", logs_count);

    // iced

    // db_logs.iter().for_each(|db_log| {
    //     println!("{:?}", db_log);
    // });

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
