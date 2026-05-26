#[tokio::test]
async fn user_repository_schema_is_available() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("db connection");

    let count = sqlx::query_scalar::<_, i64>("select count(*) from users")
        .fetch_one(&pool)
        .await;

    let count = count.expect("users table should exist");
    assert!(count >= 0);
}
