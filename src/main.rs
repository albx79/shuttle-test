use askama::Template;
use axum::{routing::get, Router};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::post;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use edgedb_tokio::Client as EdgeClient;
use serde::Deserialize;

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    println!("EDGEDB {:?};", std::env::var("EDGEDB_SECRET_KEY"));
    let client: EdgeClient = edgedb_tokio::create_client().await
        .map_err(|e| shuttle_runtime::Error::Database(e.to_string()))?;
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "with_axum_htmx_askama=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("initializing router...");
    let fate_router = Router::new()
        .route("/characters/:id", get(render_character))
        .with_state(client.clone());

    let router = Router::new()
        .nest("/fate", fate_router)
        .with_state(client);

    Ok(router.into())
}

#[derive(Template)]
#[template(path = "character-sheet.html")]
struct CharacterSheet {
    character: Character,
    editable: bool,
}

struct Character {
    name: String,
    aspects: Vec<String>,
    skills: Vec<Skill>,
    stunts: Vec<String>,
}

struct Skill {
    name: String,
    rating: u8,
}

#[derive(Deserialize, Default)]
struct RenderCharacter {
    #[serde(default)]
    editable: bool,
}

async fn render_character(Path(id): Path<String>, Query(params): Query<RenderCharacter>, State(_client): State<EdgeClient>) -> impl IntoResponse {
    let char = CharacterSheet {
        character: Character {
            name: "John Doe".into(),
            aspects: [
                "Brave adventurer",
                "Afraid of the dark",
                "Clever",
                "Resourceful",
            ].iter().map(|s| s.to_string()).collect(),
            skills: vec![
                Skill{ name: "Contacts".to_string(), rating: 4 },
                Skill{ name: "Deceive".to_string(), rating: 3 },
                Skill{ name: "Provoke".to_string(), rating: 3 },
                Skill{ name: "Rapport".to_string(), rating: 2 },
                Skill{ name: "Contacts".to_string(), rating: 2 },
                Skill{ name: "Shoot".to_string(), rating: 2 },
            ],
            stunts: vec![
                "Acrobatic Maneuver: Once per session, gain +2 to Athletics for a daring physical feat.".to_string(),
                "Master of Disguise: Gain +2 to Deceive when attempting to disguise yourself.".to_string(),
                "Keen Observer: Once per scene, reroll any failed Notice check.".to_string(),
            ],
        },
        editable: params.editable,
    };
    HtmlTemplate(char)
}

/// Allows us to convert Askama HTML templates into valid HTML for axum to serve in the response.
impl<T> IntoResponse for HtmlTemplate<T>
    where
        T: Template,
{
    fn into_response(self) -> Response {
        // Attempt to render the template with askama
        match self.0.render() {
            // If we're able to successfully parse and aggregate the template, serve it
            Ok(html) => Html(html).into_response(),
            // If we're not, return an error or some bit of fallback HTML
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

/// A wrapper type that we'll use to encapsulate HTML parsed by askama into valid HTML for axum to serve.
struct HtmlTemplate<T>(T);

