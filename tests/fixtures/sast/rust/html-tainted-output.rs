fn handler() -> axum::response::Html<String> {
    let input = std::env::var("SNIPE_INPUT").unwrap();
    axum::response::Html(format!("<p>{input}</p>"))
}
