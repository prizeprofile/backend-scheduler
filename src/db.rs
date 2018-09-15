use mysql;
use std::env;

pub fn get_latest_id_for_region(region_id: u64) -> Result<u64, &'static str> {
    // TODO: Error handling.
    let user: String = env::var("DB_USER").expect("Config is missing some fields [DB_USER]");
    let host: String = env::var("DB_HOST").expect("Config is missing some fields [DB_HOST]");
    let password: String = env::var("DB_PASSWORD").expect("Config is missing some fields [DB_PASSWORD]");
    let credentials: String = format!("mysql://{}:{}@{}:3306/prizeprofile", user, password, host);

    let pool = mysql::Pool::new(credentials).expect("Couldn't connect to the database");

    let query: String = format!("
        SELECT tweet_id FROM competitions
        WHERE region_id = {}
        ORDER BY tweet_id DESC LIMIT 1", region_id);

    let id: u64 =
        pool.prep_exec(query, ())
        .map(|result| {
            let mut tweet_id: u64 = 0;

            for xd in result.map(|x| x.unwrap()) {
                tweet_id = mysql::from_row(xd);
            }

            tweet_id
        }).unwrap_or(1);

    Ok(id)
}
