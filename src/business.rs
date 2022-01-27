use crate::log::Log;
use amazon_log::{AmazonBrowser, AmazonBrowserResult};
use diesel::prelude::*;
use diesel::result::Error;
use diesel::MysqlConnection;
use range::Range;
use transaction::prelude::*;
use transaction_diesel_mysql::{with_conn, DieselContext};

type Ctx<'a> = DieselContext<'a, MysqlConnection>;
type BoxTx<'a, T> = Box<dyn Transaction<Ctx = Ctx<'a>, Item = T, Err = Error> + 'a>;

async fn wakeup_browser() -> AmazonBrowserResult<AmazonBrowser> {
    use dotenv::dotenv;
    use std::env;
    dotenv().ok();
    let email = env::var("AMAZON_EMAIL").expect("AMAZON_EMAIL must be set");
    let pass = env::var("AMAZON_PASSWORD").expect("AMAZON_PASSWORD must be set");
    let browser = AmazonBrowser::new(&email, &pass, "air2_release").await?;
    Ok(browser)
}
pub fn most_recently_log<'a>() -> BoxTx<'a, Log> {
    use crate::schema::logs::dsl;
    use crate::schema::logs::table;
    with_conn(move |cn| table.order(dsl::purchased_at.desc()).limit(1).first(cn)).boxed()
}
pub fn next_day(date_str: String) -> String {
    use chrono::{Duration, NaiveDate};
    (NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap() + Duration::days(1))
        .format("%Y-%m-%d")
        .to_string()
}
pub fn yesterday() -> String {
    use chrono::{Duration, Local};
    (Local::today() + Duration::days(-1))
        .naive_local()
        .format("%Y-%m-%d")
        .to_string()
}
pub async fn most_formerly_date() -> AmazonBrowserResult<String> {
    let mut browser = wakeup_browser().await?;
    let result = browser.most_formerly_date().await?;
    browser.quit().await?;
    Ok(result)
}
pub async fn difference_log(diff_range: Range) -> AmazonBrowserResult<Vec<amazon_log::Log>> {
    let mut browser = wakeup_browser().await?;
    let logs = browser.extract(&diff_range).await?;
    browser.quit().await?;

    Ok(logs)
}

pub fn scrape_logs_and_save_to_db_if_needed(cn: &MysqlConnection) -> Result<(), ()> {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        get_difference_logs_from_amazon(cn).await.unwrap();
    });
    Ok(())
}

pub async fn get_difference_logs_from_amazon(cn: &MysqlConnection) -> AmazonBrowserResult<()> {
    // use crate::business::{most_formerly_date, most_recently_log, next_day, yesterday};

    let most_formerly_date = &most_formerly_date().await.unwrap();
    assert_eq!(most_formerly_date, "2018-01-01");

    // DB内の最新情報とブラウザ上の最古の情報からスクレイピング範囲を決定
    use range::Range;
    use transaction::with_ctx;
    let tx = with_ctx(|ctx| -> Result<crate::log::Log, _> { most_recently_log().run(ctx) });
    let diff_range = match transaction_diesel_mysql::run(cn, tx) {
        Ok(log) => Range::new(&yesterday(), &next_day(log.purchased_at)),
        Err(_) => Range::new(&yesterday(), most_formerly_date),
    };
    println!("start: {}\nend  : {}", diff_range.start(), diff_range.end());

    // 決定した範囲の履歴(amazon_log::Log)をスクレイピングして取得
    use amazon_log;
    let amazon_logs: Vec<amazon_log::Log> = difference_log(diff_range).await.unwrap();
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
            crate::log::create(*new_log).run(ctx)?;
        }
        Ok(())
    });
    transaction_diesel_mysql::run(&cn, tx).unwrap();

    Ok(())
}

pub fn get_all_logs_from_db(cn: &MysqlConnection) -> Vec<crate::log::Log> {
    use transaction::with_ctx;
    // DB内の全ログ(log::Log)を取得
    use crate::business::difference_log;
    use crate::log;
    let tx = with_ctx(|ctx| log::all().run(ctx));
    let db_logs = transaction_diesel_mysql::run(cn, tx).unwrap_or(vec![]);

    let max_len = db_logs.iter().map(|log| log.name.len()).max().unwrap_or(0);
    println!("最大のnameバイト数は「{}」です。", max_len);

    let logs_count = db_logs.len();
    println!("取扱件数は{}件です。", logs_count);

    db_logs
}
