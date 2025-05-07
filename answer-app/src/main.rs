use actix_web::{
    App, Error, HttpResponse, HttpRequest, HttpServer, Responder, Result, Either,
    error, get, post,
    http::{StatusCode, header::ContentType},
    web::{Html, Data, Form, Redirect},
    cookie::{Cookie, SameSite},
};
use actix_files::{Files, NamedFile};
use tera::Tera;
use std::path::PathBuf;
use deadpool_postgres::{Client, Pool};
use tokio_postgres::NoTls;
mod auth;
use auth::*;
mod misc;
use misc::*;
mod db;
use db::*;

#[post("/user-login")]
async fn user_login(Form(form): Form<UserLogin>) -> impl Responder {
    let cookie = Cookie::build("token", encode_jwt(&form.username))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::hours(1))
        .finish();

    HttpResponse::Found()
        .append_header(("Location", "/profile"))
        .cookie(cookie)
        .finish()
}

#[post("/user-register")]
async fn user_register(Form(form): Form<UserLogin>) -> impl Responder {
    let cookie = Cookie::build("token", encode_jwt(&form.username))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::hours(1))
        .finish();

    HttpResponse::Found()
        .append_header(("Location", "/profile"))
        .cookie(cookie)
        .finish()
}

#[get("/")]
async fn index() -> impl Responder {
    let path: PathBuf = "./static/html/start.html".parse().unwrap();
    return NamedFile::open(path).unwrap();
}

#[get("/login")]
async fn login(req: HttpRequest) -> Either<Redirect, NamedFile> {
    return match req.cookie("token") {
        Some(_) => Either::Left(Redirect::to("/profile").permanent()),
        None => {
            let path: PathBuf = "./static/html/login.html".parse().unwrap();
            Either::Right(NamedFile::open(path).unwrap())
        },
    }
}

#[get("/register")]
async fn register() -> impl Responder {
    let path: PathBuf = "./static/html/register.html".parse().unwrap();
    return NamedFile::open(path).unwrap();
}

#[get("/profile")]
async fn profile(tmpl: Data<Tera>, req: HttpRequest) -> Either<Html, Redirect> {
    return match req.cookie("token") {
        Some(x) => {
            let mut ctx = tera::Context::new();
            Either::Left(Html::new(tmpl.render("profile.html", &ctx).unwrap()))
        },
        None => Either::Right(Redirect::to("/login").permanent()),
    }
}

#[get("/answer")]
async fn next_question(tmpl: Data<Tera>, req: HttpRequest) -> Either<Html, Redirect> {
    return match req.cookie("token") {
        Some(x) => {
            let mut ctx = tera::Context::new();
            ctx.insert("answers", &["pivo", "snus", "vkid"]);
            Either::Left(Html::new(tmpl.render("question.html", &ctx).unwrap()))
        },
        None => Either::Right(Redirect::to("/login").permanent()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    HttpServer::new(|| {
        let tera = Tera::new("/app/static/html/*").unwrap();
        let pool = config.pg.create_pool(None, NoTls).unwrap();

        App::new()
            .service(index)
            .service(next_question)
            .service(login)
            .service(register)
            .service(user_register)
            .service(user_login)
            .service(profile)
            .service(Files::new("/static", "./static"))
            .app_data(Data::new(tera))
            .app_data(Data::new(pool.clone()))
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
