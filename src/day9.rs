use poem::{
    handler,
    http::{HeaderMap, StatusCode},
    post,
    web::Data,
    EndpointExt as _, IntoEndpoint, Response, Route,
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

pub fn route() -> impl IntoEndpoint {
    Route::new()
        .at("/milk", post(leaky_milk))
        .at("/refill", post(fill_milk_bucket))
        .data(MilkBucket(Arc::new(Mutex::new(
            leaky_bucket::RateLimiter::builder()
                .initial(5)
                .max(5)
                .interval(Duration::from_secs(1))
                .build(),
        ))))
}

#[derive(Clone)]
struct MilkBucket(Arc<Mutex<leaky_bucket::RateLimiter>>);

#[derive(Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
enum MilkConversion {
    Liters { liters: f32 },
    Gallons { gallons: f32 },
    IHateTheBritish { litres: f32 },
    IHateTheBritishMore { pints: f32 },
}

#[handler]
async fn leaky_milk(headers: &HeaderMap, bucket: Data<&MilkBucket>, body: String) -> Response {
    if !bucket.0 .0.lock().await.try_acquire(1) {
        return Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .body("No milk available\n");
    }

    if headers
        .get("Content-Type")
        .map_or(false, |v| v == "application/json")
    {
        let Ok(conversion_request): Result<MilkConversion, _> = serde_json::from_str(&body) else {
            return StatusCode::BAD_REQUEST.into();
        };

        let response = match conversion_request {
            MilkConversion::Liters { liters } => {
                let gallons = liters * 0.2641720524;
                MilkConversion::Gallons { gallons }
            }
            MilkConversion::Gallons { gallons } => {
                let liters = gallons * 3.785411784;
                MilkConversion::Liters { liters }
            }
            MilkConversion::IHateTheBritish { litres } => {
                let pints = litres * 1.75975;
                MilkConversion::IHateTheBritishMore { pints }
            }
            MilkConversion::IHateTheBritishMore { pints } => {
                let litres = pints * 0.56826;
                MilkConversion::IHateTheBritish { litres }
            }
        };
        return serde_json::to_string(&response).unwrap().into();
    }

    "Milk withdrawn\n".into()
}

#[handler]
async fn fill_milk_bucket(bucket: Data<&MilkBucket>) {
    *bucket.0 .0.lock().await = leaky_bucket::RateLimiter::builder()
        .initial(5)
        .max(5)
        .interval(Duration::from_secs(1))
        .build();
}
