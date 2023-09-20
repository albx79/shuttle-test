use actix_web::Result as AwResult;
use actix_web::{get, web::ServiceConfig};
use maud::{html, Markup};
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_runtime::CustomError;
use sqlx::{Executor, PgPool};

#[get("/")]
async fn hello_world() -> AwResult<Markup> {
    Ok(html!{
        html {
            head {
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/water.css@2/out/water.css";
                script src="https://unpkg.com/htmx.org@1.9.5"{}
            }
            body {
                h1 { "Todo List" }
                ul {
                    li { "Item 1" }
                    li { "Item 2" }
                    li {
                        form {
                            input type="text" name="item";
                            button type="submit" { "Submit" }
                        }
                    }
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
        cfg.service(hello_world);
    };

    Ok(config.into())
}
