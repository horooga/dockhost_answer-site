use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
use yaml_rust2::{YamlLoader, yaml::Yaml};

pub static TEXT: Lazy<HashMap<&'static str, [&'static str; 2]>> = Lazy::new(|| {
    HashMap::from([
        (
            "username_short",
            [
                "Username should be longer than 5 symbols",
                "Юзернейм должен быть длинее 5-и символов",
            ],
        ),
        (
            "password_short",
            [
                "Password should be longer than 8 symbols",
                "Пароль должен быть длинее 8-и символов",
            ],
        ),
        (
            "password_long",
            [
                "Password should be shorter than 30 symbols",
                "Пароль должен быть короче 30-и символов",
            ],
        ),
        (
            "username_long",
            [
                "Username should be shorter than 15 symbols",
                "Юзернейм должен быть короче 15-и символов",
            ],
        ),
        (
            "username_cont",
            [
                "Username should consist only from alphanumeric symbols",
                "Юзернейм должен состоять лишь из букв и чисел",
            ],
        ),
        (
            "password_cont",
            [
                "Password should consist only from alphanumeric symbols + !@#$%^&*()_+=-?><",
                "Пароль должен состоять лишь из букв и чисел + !@#$%^&*()_+=-?><",
            ],
        ),
        (
            "user_registered",
            [
                "Username is already registered",
                "Юзернейм уже зарегистрирован",
            ],
        ),
        (
            "user_not_registered",
            ["Username is not registered", "Юзернейм не зарегистрирован"],
        ),
        (
            "sorry",
            ["Sorry, try again later", "Извините, попробуйте позже"],
        ),
        (
            "login_wrong",
            ["Wrong username of password", "Неверный логин или пароль"],
        ),
    ])
});
static FILE: Lazy<String> = Lazy::new(|| std::fs::read_to_string("/app/questions.yaml").unwrap());
static YAML: Lazy<Yaml> =
    Lazy::new(|| YamlLoader::load_from_str(FILE.as_str()).unwrap()[0].clone());
pub static QUESTIONS: Lazy<Vec<Question>> = Lazy::new(|| {
    YAML.as_vec()
        .unwrap()
        .into_iter()
        .map(|q| Question {
            topic: q["topic"].as_str().unwrap(),
            question: q["question"]
                .as_vec()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap())
                .collect(),
            options: if let Some(options) = q
                .as_hash()
                .unwrap()
                .get(&Yaml::String("options".to_owned()))
            {
                Some(
                    options
                        .as_vec()
                        .unwrap()
                        .iter()
                        .map(|i| {
                            i.as_vec()
                                .unwrap()
                                .iter()
                                .map(|s| s.as_str().unwrap())
                                .collect()
                        })
                        .collect(),
                )
            } else {
                None
            },
            answer: q.as_hash().unwrap()[&Yaml::String("answer".to_owned())]
                .as_vec()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap())
                .collect(),
        })
        .collect()
});

#[derive(Hash)]
pub struct Question {
    pub topic: &'static str,
    pub question: Vec<&'static str>,
    pub options: Option<Vec<Vec<&'static str>>>,
    pub answer: Vec<&'static str>,
}

#[derive(Deserialize)]
pub struct LangChange {
    pub lang_id: String,
}

#[derive(Deserialize)]
pub struct Answer {
    pub topic: String,
    pub qstn_id: u8,
    pub answer: String,
}

pub async fn validate(
    username: &String,
    password: &String,
    language_id: u8,
) -> Result<(), Vec<String>> {
    let mut errs: Vec<String> = vec![];
    if username.len() < 5 {
        errs.push(TEXT["username_short"][language_id as usize].to_string());
    }
    if password.len() < 8 {
        errs.push(TEXT["password_short"][language_id as usize].to_string());
    }
    if password.len() > 30 {
        errs.push(TEXT["password_long"][language_id as usize].to_string());
    }
    if username.len() > 15 {
        errs.push(TEXT["username_short"][language_id as usize].to_string());
    }
    if !username.chars().all(char::is_alphanumeric) {
        errs.push(TEXT["username_cont"][language_id as usize].to_string());
    }
    if !password
        .chars()
        .all(|x| char::is_alphanumeric(x) || "!@#$%^&*()_+=-?><".contains(x))
    {
        errs.push(TEXT["password_cont"][language_id as usize].to_string());
    }

    if errs.is_empty() { Ok(()) } else { Err(errs) }
}
