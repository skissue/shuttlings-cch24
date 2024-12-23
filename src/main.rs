mod day0;
mod day12;
mod day16;
mod day19;
mod day2;
mod day23;
mod day5;
mod day9;

use poem::{EndpointExt as _, Route};
use shuttle_poem::ShuttlePoem;

#[shuttle_runtime::main]
async fn poem(
    #[shuttle_shared_db::Postgres(local_uri = "postgres://localhost:5432/")] pool: sqlx::PgPool,
) -> ShuttlePoem<impl poem::Endpoint> {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let app = Route::new()
        // .nest_no_strip("/", day0::route())
        .nest("/2", day2::route())
        .nest("/5", day5::route())
        .nest("/9", day9::route())
        .nest("/12", day12::route())
        .nest("/16", day16::route())
        .nest("/19", day19::route())
        .nest_no_strip("/", day23::route())
        .data(pool);

    Ok(app.into())
}
