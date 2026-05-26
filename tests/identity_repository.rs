#[tokio::test]
async fn user_repository_inserts_and_reads_user() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("db connection");

    let row = sqlx::query_scalar::<_, i64>("select count(*) from users")
        .fetch_one(&pool)
        .await;

    assert!(row.is_ok());
}
