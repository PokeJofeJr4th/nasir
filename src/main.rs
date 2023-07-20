#![warn(clippy::nursery, clippy::pedantic)]

use reqwest::blocking as http;

mod parser;
mod types;

use types::DocElement;

use crate::parser::parse_html;

fn main() {
    let fetch = fetch_html("https://www.wikipedia.org");
    // println!("{fetch:#?}");
    render_html(&fetch);
}

// fn fetch(url: &str) -> Result<String, reqwest::Error> {
//     let body = http::get(url)?.text()?;
//     Ok(body)
// }

fn fetch_html(url: &str) -> DocElement {
    let response = match http::get(url) {
        Ok(response) => response,
        Err(err) => return DocElement::Text(format!("Network Error: {err}").into()),
    };
    let body: String = match response.text() {
        Ok(body) => body,
        Err(err) => return DocElement::Text(format!("Network Error: {err}").into()),
    };
    // println!("{body}");
    match parse_html(&body) {
        Ok(html) => html,
        Err(err) => DocElement::Text(format!("HTML Parsing Error: {err}").into()),
    }
}

fn render_html(html: &DocElement) {
    println!("{}", html.display());
}
