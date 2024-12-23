use poem::{endpoint::StaticFilesEndpoint, *};

pub fn route() -> Route {
    Route::new()
        .nest("/assets", StaticFilesEndpoint::new("assets"))
        .at("/23/star", get(light_star))
}

#[handler]
async fn light_star() -> impl IntoResponse {
    r#"<div id="star" class="lit"></div>"#
}
