use http::StatusCode;
use poem::{
    endpoint::StaticFilesEndpoint,
    web::{Multipart, Path},
    *,
};
use toml::Table;

pub fn route() -> Route {
    Route::new()
        .nest("/assets", StaticFilesEndpoint::new("assets"))
        .at("/23/star", get(light_star))
        .at("/23/present/:color", get(cycle_present_color))
        .at("/23/ornament/:state/:n", get(ornament_iteration))
        .at("/23/lockfile", post(bake_a_cake))
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

#[handler]
async fn bake_a_cake(mut form: Multipart) -> Result<impl IntoResponse> {
    let field = form.next_field().await?.ok_or(StatusCode::BAD_REQUEST)?;

    let lockfile: Table =
        toml::from_str(&field.text().await?).map_err(|_| StatusCode::BAD_REQUEST)?;
    let checksums: Vec<&str> = lockfile
        .get("package")
        .unwrap()
        .as_array()
        .unwrap()
        .into_iter()
        .filter_map(|p| p.get("checksum").and_then(|c| c.as_str()))
        .collect();

    if checksums.is_empty() {
        return Err(StatusCode::BAD_REQUEST.into());
    }

    checksums
        .into_iter()
        .map(|checksum| {
            if checksum.len() < 10
                || checksum.chars().any(|c| {
                    let c = c.to_ascii_lowercase();

                    // ASCII wizardry
                    c < '0' || c > 'f' || (c > '9' && c < 'a')
                })
            {
                return Err(StatusCode::UNPROCESSABLE_ENTITY.into());
            }

            let color = &checksum[0..6];
            let top = i32::from_str_radix(&checksum[6..8], 16).unwrap();
            let left = i32::from_str_radix(&checksum[8..10], 16).unwrap();

            Ok(format!(
                r#"<div style="background-color:#{color};top:{top}px;left:{left}px;"></div>"#
            ))
        })
        .collect::<Result<Vec<String>>>()
        .map(|v| v.join("\n"))
}
