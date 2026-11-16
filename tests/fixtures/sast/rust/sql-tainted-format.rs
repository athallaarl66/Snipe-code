use rusqlite::Connection;

fn query(connection: &Connection) {
    let input = std::env::var("SNIPE_INPUT").unwrap();
    let sql = format!("SELECT value FROM records WHERE name = '{input}'");
    let _ = connection.execute(&sql, []);
}
