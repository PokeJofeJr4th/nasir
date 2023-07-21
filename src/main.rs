#![warn(clippy::nursery, clippy::pedantic)]

use core::time;
use std::io::stdout;

use clap::Parser;
use crossterm::{
    event::{self, KeyCode},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode, SetTitle},
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
    browse(&args.url, args.verbose);
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

fn browse(url: &str, verbose: bool) {
    let mut set_title = SetTitle("".into());
    enable_raw_mode().unwrap();
    let mut breadcrumbs = vec![String::from(url)];
    let fetched = fetch_html(url, verbose);
    if verbose {
        println!("{fetched:#?}");
    }
    let mut htmelements = fetched.display(&mut set_title);
    execute!(stdout(), set_title).unwrap();
    let mut focused = 0;
    loop {
        if htmelements.len() <= focused {
            focused = htmelements.len() - 1;
        }
        let lines = render_lines(&htmelements, focused);
        // clear the screen
        println!("\x1B[2J\x1B[1;1H");
        // print out the current window
        for l in lines {
            println!("{l}");
        }
        while matches!(event::poll(time::Duration::from_secs(0)), Ok(false)) {}
        if let Ok(event::Event::Key(event::KeyEvent {
            code,
            kind: event::KeyEventKind::Press,
            ..
        })) = event::read()
        {
            match code {
                KeyCode::Esc => {
                    breadcrumbs.pop();
                    if let Some(last) = breadcrumbs.last() {
                        follow_link(RStr::from(last.as_ref()), &mut htmelements, verbose);
                        focused = 0;
                    } else {
                        break;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    focused = focused.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => focused += 1,
                KeyCode::Enter => {
                    if let InteractionType::Link(link) = htmelements[focused].interaction() {
                        breadcrumbs.push(String::from(&**link));
                        follow_link(link.clone(), &mut htmelements, verbose);
                        focused = 0;
                    }
                }
                _ => {}
            }
        }
    }
    disable_raw_mode().unwrap();
}

fn follow_link(link: RStr, htmelements: &mut Vec<TerminalLine>, verbose: bool) {
    let fetched = fetch_html(&link, verbose);
    let mut set_title = SetTitle(link);
    *htmelements = fetched.display(&mut set_title);
    execute!(stdout(), set_title).unwrap();
}

/// print out the lines out of a parsed html
fn render_lines(lines: &[TerminalLine], focused: usize) -> Vec<String> {
    let mut effective_focus = focused;
    let min = 0;
    let window_height = terminal::size().unwrap().1 as usize / 2 - 1;
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
