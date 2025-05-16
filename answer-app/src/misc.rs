use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;
use yaml_rust2::{YamlLoader, yaml::Yaml};

#[derive(Deserialize)]
pub struct Answer {
    pub question: String,
    pub answer: String,
}

lazy_static! {
    pub static ref TEXT: HashMap<&'static str, [&'static str; 2]> = HashMap::from([
        (
            "username_short",
            [
                "Username should be longer than 5 symbols",
                "Юзернейм должен быть длинее 5-и символов"
            ]
        ),
        (
            "password_short",
            [
                "Password should be longer than 8 symbols",
                "Паролб должен быть длинее 8-и символов"
            ]
        ),
        (
            "password_long",
            [
                "Password should be shorter than 30 symbols",
                "Пароль должен быть короче 30-и символов"
            ]
        ),
        (
            "username_long",
            [
                "Username should be shorter than 15 symbols",
                "Юзернейм должен быть короче 15-и символов"
            ]
        ),
        (
            "username_cont",
            [
                "Username should consist only from alphanumeric symbols",
                "Юзернейм должен состоять лишь из букв и чисел"
            ]
        ),
        (
            "password_cont",
            [
                "Password should consist only from alphanumeric symbols + !@#$%^&*()_+=-?><",
                "Пароль должен состоять лишь из букв и чисел + !@#$%^&*()_+=-?><"
            ]
        ),
        (
            "user_registered",
            [
                "Username is already registered",
                "Юзернейм уже зарегистрирован"
            ]
        ),
        (
            "sorry",
            ["Sorry, try again later", "Извините, попробуйте позже"]
        ),
    ]);
    static ref file: String = std::fs::read_to_string("/app/questions.yaml").unwrap();
    pub static ref QUESTIONS: Yaml = YamlLoader::load_from_str(file.as_str()).unwrap()[0].clone();
    static ref questions_vec: Vec<(&'static str, &'static str)> = QUESTIONS
        .as_hash()
        .unwrap()
        .values()
        .into_iter()
        .map(|i| i.as_vec().unwrap())
        .flatten()
        .map(|v| (
            v["question"].as_str().unwrap(),
            v["answer"].as_str().unwrap()
        ))
        .collect();
    pub static ref ANSWERS: HashMap<&'static str, &'static str> =
        questions_vec.clone().into_iter().collect();
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
