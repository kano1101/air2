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
