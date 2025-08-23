use askama::Template;
use crate::models::LinkResponse;

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate {
    pub links: Vec<LinkResponse>,
}

#[derive(Template)]
#[template(path = "add.html")]
pub struct AddTemplate<'a> {
    pub source: &'a str,
    pub error: Option<&'a str>,
}

#[derive(Template)]
#[template(path = "edit.html")]
pub struct EditTemplate<'a> {
    pub link: &'a LinkResponse,
    pub error: Option<&'a str>,
}

#[derive(Template)]
#[template(path = "search.html")]
pub struct SearchTemplate<'a> {
    pub query: &'a str,
    pub links: Vec<LinkResponse>,
    pub page: i32,
}
