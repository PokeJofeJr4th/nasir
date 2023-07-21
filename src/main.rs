#![warn(clippy::nursery, clippy::pedantic)]

use core::time;

use clap::Parser;
use crossterm::{
    event::{self, KeyCode},
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use reqwest::blocking as http;

mod parser;
mod types;

use crate::parser::parse_html;
use types::prelude::*;

#[derive(Parser)]
struct Args {
    /// the page to visit
    url: String,
    /// print extra debug information
    #[clap(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();
    let fetch = fetch_html(&args.url, args.verbose);
    if args.verbose {
        println!("{fetch:#?}");
    }
    render_html(&fetch);
}

// fn fetch(url: &str) -> Result<String, reqwest::Error> {
//     let body = http::get(url)?.text()?;
//     Ok(body)
// }

fn fetch_html(url: &str, verbose: bool) -> DocElement {
    let response = match http::get(url) {
        Ok(response) => response,
        Err(err) => return DocElement::Text(format!("Network Error: {err}").into()),
    };
    let body: String = match response.text() {
        Ok(body) => body,
        Err(err) => return DocElement::Text(format!("Network Error: {err}").into()),
    };
    if verbose {
        println!("{body}");
    }
    match parse_html(&body) {
        Ok(html) => html,
        Err(err) => DocElement::Text(format!("HTML Parsing Error: {err}").into()),
    }
}

fn render_html(html: &DocElement) {
    enable_raw_mode().unwrap();
    let mut htmelements = html.display();
    let mut focused = 0;
    loop {
        if htmelements.len() <= focused {
            focused = htmelements.len() - 1;
        }
        while matches!(event::poll(time::Duration::from_secs(0)), Ok(false)) {}
        if let Ok(event::Event::Key(event::KeyEvent {
            code,
            kind: event::KeyEventKind::Press,
            ..
        })) = event::read()
        {
            match code {
                KeyCode::Esc => break,
                KeyCode::Up | KeyCode::PageUp => {
                    focused = focused.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::PageDown => focused += 1,
                KeyCode::Enter => {
                    if let InteractionType::Link(link) = htmelements[focused].interaction() {
                        todo!("navigate to {link}")
                    }
                }
                _ => {}
            }
        }
        let lines = render_lines(&htmelements, focused);
        // clear the screen
        print!("\x1B[2J\x1B[1;1H");
        for l in lines {
            println!("{l}");
        }
    }
    disable_raw_mode().unwrap();
}

/// print out the lines out of a parsed html
fn render_lines(lines: &[TerminalLine], focused: usize) -> Vec<RStr> {
    let mut effective_focus = focused;
    let min = 0;
    let window_height = terminal::size().unwrap().1 as usize / 2;
    let max = lines.len();
    // can't focus past the end of the page
    if effective_focus > max {
        effective_focus = max;
    }
    // window has to end before the end of the page
    if effective_focus + window_height > max {
        effective_focus = max - window_height;
    }
    // window has to start after the start of the page
    if effective_focus < min + window_height {
        effective_focus = window_height;
    }
    let start = effective_focus - window_height;
    let end = (effective_focus + window_height).min(max);
    lines
        .iter()
        .enumerate()
        .take(end)
        .skip(start)
        .map(|(i, line)| line.display(i == focused))
        .collect()
}
