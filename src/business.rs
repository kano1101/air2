use crate::history::History;
use amazon_log::{AmazonBrowser, AmazonBrowserResult, Log};
use diesel::prelude::*;
use diesel::result::Error;
use diesel::MysqlConnection;
use range::Range;
use transaction::prelude::*;
use transaction_diesel_mysql::{with_conn, DieselContext};

type Ctx<'a> = DieselContext<'a, MysqlConnection>;
type BoxTx<'a, T> = Box<dyn Transaction<Ctx = Ctx<'a>, Item = T, Err = Error> + 'a>;

fn most_reacently_history<'a>() -> BoxTx<'a, History> {
    use crate::schema::histories::dsl;
    use crate::schema::histories::table;
    with_conn(move |cn| table.order(dsl::purchased_at.desc()).limit(1).first(cn)).boxed()
}
fn yesterday() -> String {
    use chrono::{Duration, Local};
    (Local::today() + Duration::days(-1))
        .naive_local()
        .format("%Y-%m-%d")
        .to_string()
}
fn difference_period_range(history: &History) -> Range {
    Range::new(&history.purchased_at, &yesterday())
}
pub async fn difference_log() -> AmazonBrowserResult<Vec<Log>> {
    use crate::utils::establish_connection;
    use dotenv::dotenv;
    use std::env;

    dotenv().ok();

    let tx = with_ctx(|ctx| -> Result<History, Error> { most_reacently_history().run(ctx) });
    let cn = establish_connection();

    let diff_range = match transaction_diesel_mysql::run(&cn, tx) {
        Ok(history) => difference_period_range(&history),
        Err(NotFound) => Range::new("2018-01-14", &yesterday()),
        // Err(NotFound) => Range::new("2021-11-08", &yesterday()),
        _ => panic!("例外的なエラーです。"),
    };

    let email = env::var("AMAZON_EMAIL").expect("AMAZON_EMAIL must be set");
    let pass = env::var("AMAZON_PASSWORD").expect("AMAZON_PASSWORD must be set");

    let mut browser = AmazonBrowser::new(&email, &pass, "air2_release").await?;
    let logs = browser.extract(&diff_range).await?;
    browser.quit().await?;

    Ok(logs)
}
