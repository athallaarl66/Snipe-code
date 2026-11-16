use rusqlite::Connection;

fn query(connection: &Connection) {
    let input = std::env::var("SNIPE_INPUT").unwrap();
    let _ = connection.execute(
        "SELECT value FROM records WHERE name = ?1",
        [&input],
    );
}
