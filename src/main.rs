use actix_web::{post, Result as AwResult, web};
use actix_web::{get, web::ServiceConfig};
use actix_web::error::ErrorInternalServerError;
use actix_web::web::{Data, Path};
use maud::{html, Markup};
use ndm::Dice;
use serde::Deserialize;
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_runtime::CustomError;
use sqlx::{Executor, FromRow, PgPool};

#[get("/")]
async fn root(conn: Data<PgPool>) -> AwResult<Markup> {
    Ok(html! {
        html {
            (head())
            body {
                h1 { "Todo List" }
                div #"notes" {
                    (get_notes(conn).await?)
                }
            }
        }
    })
}

fn head() -> Markup {
    html! {
        head {
            link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/water.css@2/out/water.css";
            link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@24,400,0,0";
            script src="https://unpkg.com/htmx.org@1.9.5" {}
        }
    }
}

#[get("/notes")]
async fn notes(conn: Data<PgPool>) -> AwResult<Markup> {
    get_notes(conn).await
}

async fn get_notes(conn: Data<PgPool>) -> AwResult<Markup> {
    let all_notes: Vec<Note> = sqlx::query_as("SELECT id, note FROM todos").fetch_all(conn.get_ref())
        .await.map_err(|e| ErrorInternalServerError(e))?;
    Ok(html! {
        table {
            tbody {
                @for note in &all_notes {
                    tr {
                        td { (note.note) }
                        td { a href="" hx-post={"/notes/"(note.id)"/delete"} hx-target="#notes" {
                            span ."material-symbols-outlined" { "delete" }
                        } }
                    }
                }
            }
        }
        form hx-post="/notes" hx-target="#notes" {
            input type="text" name="text" autofocus;
            button type="submit" { "Submit" }
        }
    })
}

#[post("/notes")]
async fn create_note(conn: Data<PgPool>, web::Form(new_note): web::Form<NewNote>) -> AwResult<Markup> {
    sqlx::query("INSERT INTO todos(note) VALUES ($1)")
        .bind(new_note.text)
        .execute(conn.get_ref())
        .await.map_err(|e| ErrorInternalServerError(e))?;
    get_notes(conn).await
}

#[post("/notes/{id}/delete")]
async fn delete_note(conn: Data<PgPool>, id: Path<i32>) -> AwResult<Markup> {
    sqlx::query("DELETE FROM todos WHERE id=$1")
        .bind(id.into_inner())
        .execute(conn.get_ref())
        .await.map_err(|e| ErrorInternalServerError(e))?;
    get_notes(conn).await
}

#[derive(Debug, FromRow)]
struct Note {
    id: i32,
    note: String,
}

#[derive(Debug, Deserialize)]
struct NewNote {
    text: String,
}

#[get("/roll")]
async fn roll_dice(web::Query(dice): web::Query<Option<String>>) -> AwResult<Markup> {
    let dice = dice
        .unwrap_or("1d6".to_string())
        .parse::<Dice>().
        .map_err(CustomError::new)?;
    Ok(html!{
        (head())
        body {
            div {
                @for roll in dice.rolls() {
                    span ."roll" { (roll) }
                }
            }
        }
    })
}

#[shuttle_runtime::main]
async fn actix_web(
    #[shuttle_shared_db::Postgres] pool: PgPool,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    pool.execute(include_str!("./schema.sql"))
        .await
        .map_err(CustomError::new)?;

    let config = move |cfg: &mut ServiceConfig| {
        cfg
            .app_data(Data::new(pool))
            .service(root)
            .service(notes)
            .service(create_note)
            .service(delete_note)
            .service(roll_dice)
        ;
    };

    Ok(config.into())
}
