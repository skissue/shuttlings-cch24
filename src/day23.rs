use poem::{endpoint::StaticFilesEndpoint, *};

pub fn route() -> Route {
    Route::new().nest("/assets", StaticFilesEndpoint::new("assets"))
}
