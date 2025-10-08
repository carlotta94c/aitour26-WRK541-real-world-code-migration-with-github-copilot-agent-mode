use std::collections::HashMap;

use actix_web::{
    App, HttpResponse, HttpServer, Responder, get,
    http::{StatusCode, header::LOCATION},
    web,
};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, ToSchema)]
struct Temperature {
    high: f64,
    low: f64,
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug, ToSchema)]
struct ErrorResponse {
    detail: String,
}

type WeatherData = HashMap<String, HashMap<String, HashMap<String, Temperature>>>;

#[derive(Clone)]
struct AppState {
    weather: WeatherData,
}

const WEATHER_JSON: &str = include_str!("../../python-app/webapp/weather.json");

fn load_weather_data() -> WeatherData {
    serde_json::from_str(WEATHER_JSON).expect("Failed to parse weather data")
}

#[get("/")]
async fn root() -> impl Responder {
    HttpResponse::build(StatusCode::MOVED_PERMANENTLY)
        .append_header((LOCATION, "/docs/"))
        .finish()
}

#[get("/docs")]
async fn docs_redirect() -> impl Responder {
    HttpResponse::build(StatusCode::MOVED_PERMANENTLY)
        .append_header((LOCATION, "/docs/"))
        .finish()
}

#[utoipa::path(
    get,
    path = "/countries",
    responses(
        (status = 200, description = "List available countries", body = [String])
    ),
    tag = "Weather"
)]
#[get("/countries")]
async fn countries(state: web::Data<AppState>) -> impl Responder {
    let mut countries: Vec<String> = state.weather.keys().cloned().collect();
    countries.sort();
    HttpResponse::Ok().json(countries)
}

#[utoipa::path(
    get,
    path = "/countries/{country}",
    params(("country" = String, Path, description = "Country whose cities are requested")),
    responses(
        (status = 200, description = "List cities within the country", body = [String]),
        (status = 404, description = "Country not found", body = ErrorResponse)
    ),
    tag = "Weather"
)]
#[get("/countries/{country}")]
async fn country_cities(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let country = path.into_inner();

    let Some(cities) = state.weather.get(&country) else {
        return HttpResponse::NotFound().json(ErrorResponse {
            detail: format!("Country '{}' not found", country),
        });
    };

    let mut city_names: Vec<String> = cities.keys().cloned().collect();
    city_names.sort();

    HttpResponse::Ok().json(city_names)
}

#[utoipa::path(
    get,
    path = "/countries/{country}/{city}/{month}",
    params(
        ("country" = String, Path, description = "Country containing the city"),
        ("city" = String, Path, description = "City to query"),
        ("month" = String, Path, description = "Month with capitalized name, e.g. 'June'")
    ),
    responses(
        (status = 200, description = "Monthly average temperature", body = Temperature),
        (status = 404, description = "Country, city, or month not found", body = ErrorResponse)
    ),
    tag = "Weather"
)]
#[get("/countries/{country}/{city}/{month}")]
async fn monthly_average(
    state: web::Data<AppState>,
    path: web::Path<(String, String, String)>,
) -> impl Responder {
    let (country, city, month) = path.into_inner();

    let Some(cities) = state.weather.get(&country) else {
        return HttpResponse::NotFound().json(ErrorResponse {
            detail: format!("Country '{}' not found", country),
        });
    };

    let Some(months) = cities.get(&city) else {
        return HttpResponse::NotFound().json(ErrorResponse {
            detail: format!("City '{}' not found in country '{}'", city, country),
        });
    };

    let Some(temperature) = months.get(&month) else {
        return HttpResponse::NotFound().json(ErrorResponse {
            detail: format!(
                "Month '{}' not found for city '{}' in country '{}'",
                month, city, country
            ),
        });
    };

    HttpResponse::Ok().json(temperature)
}

#[derive(OpenApi)]
#[openapi(
    paths(countries, country_cities, monthly_average),
    components(schemas(Temperature, ErrorResponse)),
    tags((name = "Weather", description = "Weather data endpoints"))
)]
struct ApiDoc;

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{
        App,
        http::{StatusCode, header::LOCATION},
        test,
    };

    fn init_app() -> actix_web::App<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Config = (),
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
            InitError = (),
        >,
    > {
        let app_state = web::Data::new(AppState {
            weather: load_weather_data(),
        });

        App::new()
            .app_data(app_state.clone())
            .service(root)
            .service(docs_redirect)
            .service(countries)
            .service(country_cities)
            .service(monthly_average)
            .service(SwaggerUi::new("/docs/{_:.*}").url("/api-doc/openapi.json", ApiDoc::openapi()))
    }

    #[actix_web::test]
    async fn root_redirects_to_docs() {
        let app = test::init_service(init_app()).await;

        let resp = test::call_service(&app, test::TestRequest::with_uri("/").to_request()).await;
        assert_eq!(resp.status(), StatusCode::MOVED_PERMANENTLY);
        let location = resp
            .headers()
            .get(LOCATION)
            .expect("missing LOCATION header");
        assert_eq!(location, "/docs/");
    }

    #[actix_web::test]
    async fn docs_redirects_to_trailing_slash() {
        let app = test::init_service(init_app()).await;

        let resp =
            test::call_service(&app, test::TestRequest::with_uri("/docs").to_request()).await;
        assert_eq!(resp.status(), StatusCode::MOVED_PERMANENTLY);
        let location = resp
            .headers()
            .get(LOCATION)
            .expect("missing LOCATION header");
        assert_eq!(location, "/docs/");
    }

    #[actix_web::test]
    async fn docs_serves_swagger_ui() {
        let app = test::init_service(init_app()).await;

        let resp =
            test::call_service(&app, test::TestRequest::with_uri("/docs/").to_request()).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let body = String::from_utf8(body.to_vec()).expect("docs response not utf8");
        assert!(body.contains("Swagger UI"));
    }

    #[actix_web::test]
    async fn countries_returns_sorted_list() {
        let app = test::init_service(init_app()).await;

        let body: Vec<String> = test::call_and_read_body_json(
            &app,
            test::TestRequest::with_uri("/countries").to_request(),
        )
        .await;

        assert_eq!(
            body,
            vec![
                "England", "France", "Germany", "Italy", "Peru", "Portugal", "Spain"
            ]
        );
    }

    #[actix_web::test]
    async fn country_cities_success() {
        let app = test::init_service(init_app()).await;

        let body: Vec<String> = test::call_and_read_body_json(
            &app,
            test::TestRequest::with_uri("/countries/Spain").to_request(),
        )
        .await;

        assert_eq!(body, vec!["Seville".to_string()]);
    }

    #[actix_web::test]
    async fn country_cities_not_found() {
        let app = test::init_service(init_app()).await;

        let resp = test::call_service(
            &app,
            test::TestRequest::with_uri("/countries/Unknownland").to_request(),
        )
        .await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let body: ErrorResponse = test::read_body_json(resp).await;
        assert_eq!(
            body,
            ErrorResponse {
                detail: "Country 'Unknownland' not found".to_string()
            }
        );
    }

    #[actix_web::test]
    async fn monthly_average_success() {
        let app = test::init_service(init_app()).await;

        let body: Temperature = test::call_and_read_body_json(
            &app,
            test::TestRequest::with_uri("/countries/England/London/January").to_request(),
        )
        .await;

        assert_eq!(
            body,
            Temperature {
                high: 45.0,
                low: 36.0,
            }
        );
    }

    #[actix_web::test]
    async fn monthly_average_missing_month() {
        let app = test::init_service(init_app()).await;

        let resp = test::call_service(
            &app,
            test::TestRequest::with_uri("/countries/England/London/NotAMonth").to_request(),
        )
        .await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let body: ErrorResponse = test::read_body_json(resp).await;
        assert_eq!(
            body,
            ErrorResponse {
                detail: "Month 'NotAMonth' not found for city 'London' in country 'England'"
                    .to_string()
            }
        );
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        weather: load_weather_data(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(root)
            .service(docs_redirect)
            .service(countries)
            .service(country_cities)
            .service(monthly_average)
            .service(SwaggerUi::new("/docs/{_:.*}").url("/api-doc/openapi.json", ApiDoc::openapi()))
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
