use actix_files::{Files, NamedFile};
use actix_web::{
    App, Either, HttpRequest, HttpResponse, HttpServer, Responder,
    cookie::{Cookie, SameSite},
    get,
    middleware::from_fn,
    post,
    web::{Data, Form, Html, Redirect},
};
use rand::Rng;
use sqlx::{PgPool, postgres::PgConnectOptions};
use std::path::PathBuf;
use tera::{Context, Tera};
use time::Duration;
mod auth;
use auth::{UserLogin, decode_jwt_from_req, encode_jwt, get_lang_id, jwt_auth_mw};
mod misc;
use misc::{Answer, LangChange, QUESTIONS, Question};
mod db;
use db::{get_top, get_user, login, register, update_user};

#[get("/")]
async fn index() -> impl Responder {
    let path: PathBuf = "static/html/start.html".parse().unwrap();
    NamedFile::open(path).unwrap()
}

#[get("/login")]
async fn login_handler(tmpl: Data<Tera>) -> impl Responder {
    Html::new(tmpl.render("login.tera", &Context::new()).unwrap())
}

#[get("/register")]
async fn register_handler(tmpl: Data<Tera>) -> impl Responder {
    Html::new(tmpl.render("register.tera", &Context::new()).unwrap())
}

#[post("/login-processing")]
async fn login_processing(
    req: HttpRequest,
    pool: Data<PgPool>,
    tmpl: Data<Tera>,
    Form(form): Form<UserLogin>,
) -> Either<Html, HttpResponse> {
    match login(&pool, &form.username, &form.password, get_lang_id(req)).await {
        Ok(jwt) => Either::Right(
            HttpResponse::SeeOther()
                .append_header(("Location", "/profile"))
                .cookie(
                    Cookie::build("token", jwt)
                        .path("/")
                        .http_only(true)
                        .same_site(SameSite::Lax)
                        .max_age(Duration::hours(1))
                        .finish(),
                )
                .finish(),
        ),
        Err(x) => {
            let mut ctx = Context::new();
            ctx.insert("err", &x);
            Either::Left(Html::new(tmpl.render("login.tera", &ctx).unwrap()))
        }
    }
}

#[post("/register-processing")]
async fn regster_processing(
    req: HttpRequest,
    pool: Data<PgPool>,
    tmpl: Data<Tera>,
    Form(form): Form<UserLogin>,
) -> Either<Html, HttpResponse> {
    let lang_id = get_lang_id(req);
    if let Err(x) = register(&pool, &form.username, &form.password, lang_id).await {
        let mut ctx = Context::new();
        ctx.insert("errs", &x);
        Either::Left(Html::new(tmpl.render("register.tera", &ctx).unwrap()))
    } else {
        Either::Right(
            HttpResponse::SeeOther()
                .append_header(("Location", "/profile"))
                .cookie(
                    Cookie::build(
                        "token",
                        login(&pool, &form.username, &form.password, lang_id)
                            .await
                            .unwrap(),
                    )
                    .path("/")
                    .http_only(true)
                    .same_site(SameSite::Lax)
                    .max_age(Duration::hours(1))
                    .finish(),
                )
                .finish(),
        )
    }
}

#[get("/profile")]
async fn profile(req: HttpRequest, pool: Data<PgPool>, tmpl: Data<Tera>) -> impl Responder {
    let claims = decode_jwt_from_req(req.clone()).unwrap();
    let user = get_user(&pool, &claims.usrnm, get_lang_id(req))
        .await
        .unwrap();
    let mut ctx = Context::new();
    ctx.insert("username", user.username.as_str());
    Html::new(tmpl.render("profile.tera", &ctx).unwrap())
}

#[get("/top")]
async fn top(pool: Data<PgPool>, tmpl: Data<Tera>) -> impl Responder {
    let top_users = get_top(&pool).await;
    let mut ctx = Context::new();
    ctx.insert("top_users", &top_users);
    Html::new(tmpl.render("top.tera", &ctx).unwrap())
}

#[post("/logout")]
async fn logout() -> impl Responder {
    HttpResponse::SeeOther()
        .append_header(("Location", "/"))
        .cookie(
            Cookie::build("token", "")
                .max_age(Duration::hours(0))
                .finish(),
        )
        .finish()
}

#[get("/answer")]
async fn get_question(tmpl: Data<Tera>, req: HttpRequest) -> impl Responder {
    let lang_id = get_lang_id(req);
    let mut rng = rand::rng();
    let qstn_id = rng.random_range(0..40);
    let question: &Question = &QUESTIONS[qstn_id];

    let mut ctx = tera::Context::new();
    ctx.insert("topic", question.topic);
    ctx.insert(
        "question",
        &question.question[if question.question.len() > 1 {
            lang_id as usize
        } else {
            0_usize
        }],
    );
    ctx.insert("qstn_id", &qstn_id);
    if let Some(options) = &question.options {
        ctx.insert("options", &options[lang_id as usize]);
    }
    Html::new(tmpl.render("question.html", &ctx).unwrap())
}

#[post("/answer-check")]
async fn check_answer(
    req: HttpRequest,
    Form(form): Form<Answer>,
    pool: Data<PgPool>,
) -> impl Responder {
    if QUESTIONS[form.qstn_id as usize]
        .answer
        .contains(&form.answer.as_str())
    {
        let claims = decode_jwt_from_req(req).unwrap();
        update_user(&pool, claims.usrnm.as_str(), form.topic.as_str()).await;
        Redirect::to("/answer").see_other()
    } else {
        Redirect::to("/login").see_other()
    }
}

#[post("/lang-change")]
async fn language_change(req: HttpRequest, Form(form): Form<LangChange>) -> impl Responder {
    let cookie = Cookie::build(
        "token",
        encode_jwt(
            decode_jwt_from_req(req).unwrap().usrnm.as_str(),
            form.lang_id.parse::<u8>().unwrap(),
        ),
    )
    .path("/")
    .http_only(true)
    .same_site(SameSite::Lax)
    .max_age(time::Duration::hours(1))
    .finish();

    HttpResponse::SeeOther()
        .append_header(("Location", "/profile"))
        .cookie(cookie)
        .finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let tera = Tera::new("/app/static/html/*").unwrap();
    let options = PgConnectOptions::new()
        .host(std::env::var("POSTGRES_HOST").unwrap().as_str())
        .port(5432)
        .username(std::env::var("POSTGRES_USER").unwrap().as_str())
        .password(std::env::var("POSTGRES_PASSWORD").unwrap().as_str())
        .database("app");
    let pool: PgPool = {
        if let Ok(some_pool) = sqlx::Pool::connect_with(options).await {
            some_pool
        } else {
            return Ok(());
        }
    };

    HttpServer::new(move || {
        App::new()
            .wrap(from_fn(jwt_auth_mw))
            .service(index)
            .service(login_handler)
            .service(register_handler)
            .service(login_processing)
            .service(regster_processing)
            .service(profile)
            .service(top)
            .service(logout)
            .service(get_question)
            .service(check_answer)
            .service(language_change)
            .service(Files::new("/static", "./static"))
            .app_data(Data::new(tera.clone()))
            .app_data(Data::new(pool.clone()))
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
