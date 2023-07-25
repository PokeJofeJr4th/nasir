#![warn(clippy::nursery, clippy::pedantic)]

use core::time;
use std::{
    io::{stdin, stdout, Write},
    sync::{Arc, Mutex},
};

use cacher::{get_from_cache_blocking, ByteCacher, Cacher};
use clap::Parser;
use crossterm::{
    event::{self, KeyCode},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode, SetTitle},
};

mod cacher;
mod img;
mod parser;
mod types;
mod utils;

use crate::{parser::parse_html, utils::get_link_destination};
use img::{approximate_image, get_image};
use types::prelude::*;

#[derive(Parser)]
struct Args {
    /// the page to visit
    url: Option<String>,
    /// print extra debug information
    #[clap(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();
    browse(
        &args.url.unwrap_or_else(|| {
            print!("Enter URL\r\n:");
            stdout().flush().unwrap();
            let mut url = String::new();
            stdin().read_line(&mut url).unwrap();
            url
        }),
        args.verbose,
    );
}

// fn fetch(url: &str) -> Result<String, reqwest::Error> {
//     let body = http::get(url)?.text()?;
//     Ok(body)
// }

fn fetch_html(
    url: &str,
    set_title: &mut SetTitle<RStr>,
    cacher: &Arc<Mutex<ByteCacher>>,
    verbose: bool,
) -> Vec<TerminalLine> {
    let bytes = match get_from_cache_blocking(cacher, url) {
        Ok(response) => response,
        Err(err) => return vec![TerminalLine::from(format!("Network Error: {err}"))],
    };
    // if we can get an image, return it
    if let Ok(img) = get_image(&bytes) {
        return approximate_image(
            &img,
            {
                let size = terminal::size().unwrap();
                (size.0.into(), size.1.into())
            },
            verbose,
        );
    }
    let body: String = match String::from_utf8(bytes) {
        Ok(body) => body,
        Err(err) => return vec![TerminalLine::from(format!("Network Error: {err}"))],
    };
    if verbose {
        print!("response body: {body}\r\n");
    }
    match parse_html(&body) {
        Ok(html) => html.display(set_title, cacher, url, verbose),
        Err(err) => vec![TerminalLine::from(format!("HTML Parsing Error: {err}"))],
    }
}

fn browse(url: &str, verbose: bool) {
    let terminal_handler = TermHandler::new();
    let cacher: Arc<Mutex<ByteCacher>> = Arc::new(Mutex::new(Cacher::new()));
    let mut breadcrumbs = vec![String::from(url)];
    let mut htmelements = Vec::new();
    load_link(url.into(), &mut htmelements, &cacher, verbose);
    if verbose {
        print!("{htmelements:#?}\r\n");
    }
    let mut focused = 0;
    loop {
        if htmelements.len() <= focused {
            focused = htmelements.len() - 1;
        }
        let lines = render_lines(&htmelements, focused, verbose);
        // clear the screen
        print!("\x1B[2J\x1B[1;1H");
        // print out the current window
        for l in lines {
            print!("{l}\r\n");
        }
        while matches!(event::poll(time::Duration::from_secs(0)), Ok(false)) {}
        while matches!(event::poll(time::Duration::from_secs(0)), Ok(true)) {
            if let Ok(event::Event::Key(event::KeyEvent {
                code,
                kind: event::KeyEventKind::Press,
                ..
            })) = event::read()
            {
                match code {
                    KeyCode::Esc => {
                        let current = breadcrumbs.pop().unwrap();
                        if let Some(last) = breadcrumbs.last() {
                            load_link(
                                get_link_destination(&current, &RStr::from(last.as_ref())),
                                &mut htmelements,
                                &cacher,
                                verbose,
                            );
                            focused = 0;
                        } else {
                            break;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => focused = focused.saturating_sub(1),
                    KeyCode::PageUp => focused = focused.saturating_sub(10),
                    KeyCode::Down | KeyCode::Char('j') => focused += 1,
                    KeyCode::PageDown => focused = focused.saturating_add(10),
                    KeyCode::Enter => {
                        if let InteractionType::Link(link) = htmelements[focused].interaction() {
                            let current = breadcrumbs.last().unwrap();
                            let link = load_link(
                                get_link_destination(current, link),
                                &mut htmelements,
                                &cacher,
                                verbose,
                            );
                            breadcrumbs.push(String::from(&*link));
                            focused = 0;
                        }
                    }
                    KeyCode::Char('r') => {
                        load_link(
                            RStr::from(breadcrumbs.last().unwrap().as_ref()),
                            &mut htmelements,
                            &cacher,
                            verbose,
                        );
                    }
                    KeyCode::Char('y') => {
                        let content = htmelements[focused].display(false);
                        cli_clipboard::set_contents(content).unwrap();
                    }
                    KeyCode::Char(':') => {
                        disable_raw_mode().unwrap();
                        print!(":");
                        stdout().flush().unwrap();
                        let mut response = String::new();
                        std::io::stdin().read_line(&mut response).unwrap();
                        enable_raw_mode().unwrap();
                        response = response.trim().to_owned();
                        load_link(
                            RStr::from(response.as_ref()),
                            &mut htmelements,
                            &cacher,
                            verbose,
                        );
                        breadcrumbs.push(response);
                    }
                    _ => {}
                }
            }
        }
    }
    // make sure term handler lives till here
    drop(terminal_handler);
}

/// get the link destination and fetch the content on that page
fn load_link(
    link: RStr,
    htmelements: &mut Vec<TerminalLine>,
    cacher: &Arc<Mutex<ByteCacher>>,
    verbose: bool,
) -> RStr {
    let mut set_title = SetTitle(link.clone());
    *htmelements = fetch_html(&link, &mut set_title, cacher, verbose);
    if verbose {
        print!("{htmelements:#?}\r\n");
    }
    execute!(stdout(), set_title).unwrap();
    link
}

/// print out the lines out of a parsed html
fn render_lines(lines: &[TerminalLine], focused: usize, verbose: bool) -> Vec<String> {
    let mut effective_focus = focused;
    let window_height = terminal::size().unwrap().1 as usize / 2 - 1;
    let max = lines.len();
    // can't focus past the end of the page
    if effective_focus > max {
        effective_focus = max;
    }
    // window has to end before the end of the page
    if effective_focus + window_height > max {
        // saturating sub because it'll wrap otherwise
        effective_focus = max.saturating_sub(window_height);
    }
    // window has to start after the start of the page
    if effective_focus < window_height {
        effective_focus = window_height;
    }
    let start = effective_focus - window_height;
    let end = effective_focus + window_height;
    if verbose {
        print!("showing window from {start} to {end}; effective focus: {effective_focus}; window height: {window_height}; max: {max}\r\n");
    }
    lines
        .iter()
        .enumerate()
        .take(end)
        .skip(start)
        .map(|(i, line)| line.display(i == focused))
        .collect()
}
