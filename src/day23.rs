use poem::{endpoint::StaticFilesEndpoint, web::Path, *};

pub fn route() -> Route {
    Route::new()
        .nest("/assets", StaticFilesEndpoint::new("assets"))
        .at("/23/star", get(light_star))
        .at("/23/present/:color", get(cycle_present_color))
}

#[handler]
async fn light_star() -> impl IntoResponse {
    r#"<div id="star" class="lit"></div>"#
}

#[handler]
async fn cycle_present_color(color: Path<String>) -> impl IntoResponse {
    let next = match (*color).as_str() {
        "red" => "blue",
        "blue" => "purple",
        "purple" => "red",
        _ => unreachable!()
    };
    format!(
        r#"
          <div class="present {}" hx-get="/23/present/{}" hx-swap="outerHTML">
              <div class="ribbon"></div>
              <div class="ribbon"></div>
              <div class="ribbon"></div>
              <div class="ribbon"></div>
          </div>
        "#,
        *color, next
    )
}
