use http::StatusCode;
use poem::{endpoint::StaticFilesEndpoint, web::Path, *};

pub fn route() -> Route {
    Route::new()
        .nest("/assets", StaticFilesEndpoint::new("assets"))
        .at("/23/star", get(light_star))
        .at("/23/present/:color", get(cycle_present_color))
        .at("/23/ornament/:state/:n", get(ornament_iteration))
}

#[handler]
async fn light_star() -> impl IntoResponse {
    r#"<div id="star" class="lit"></div>"#
}

#[handler]
async fn cycle_present_color(color: Path<String>) -> Result<impl IntoResponse> {
    let next = match (*color).as_str() {
        "red" => "blue",
        "blue" => "purple",
        "purple" => "red",
        _ => return Err(StatusCode::IM_A_TEAPOT.into()),
    };
    Ok(format!(
        r#"
          <div class="present {}" hx-get="/23/present/{}" hx-swap="outerHTML">
              <div class="ribbon"></div>
              <div class="ribbon"></div>
              <div class="ribbon"></div>
              <div class="ribbon"></div>
          </div>
        "#,
        *color, next
    ))
}

#[handler]
async fn ornament_iteration(Path((state, n)): Path<(String, String)>) -> Result<impl IntoResponse> {
    let next_state = match state.as_str() {
        "on" => "off",
        "off" => "on",
        _ => return Err(StatusCode::IM_A_TEAPOT.into()),
    };
    let mut buf = String::new();
    let escaped = html_escape::encode_double_quoted_attribute_to_string(n, &mut buf);

    Ok(format!(
        r#"<div class="ornament{}" id="ornament{}" hx-trigger="load delay:2s once" hx-get="/23/ornament/{}/{}" hx-swap="outerHTML"></div>"#,
        if state == "on" { " on" } else { "" },
        escaped,
        next_state,
        escaped,
    ))
}
