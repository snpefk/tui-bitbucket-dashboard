use std::io;

use termion::raw::IntoRawMode;
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, Text},
    Terminal,
};

mod bitbucket;
use bitbucket::BitBucket;

struct App {
    user: String,
    password: String,
    host: String,
    project: String,
}

impl Default for App {
    fn default() -> Self {
        let mut args = std::env::args();
        args.next();
        let user = args.next().expect("first argument must be username");
        let password = args.next().expect("second argument must be password");
        let host = args.next().expect("third argument must be host");
        let project = args.next().expect("fourth argument must be project");

        App {
            user,
            password,
            host,
            project,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::default();
    let bitbucket = BitBucket::new(&app.user, &app.password, &app.host, &app.project);
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    let repos = bitbucket
        .request_repos()
        .await?
        .iter()
        .filter_map(|r| r["name"].as_str())
        .map(String::from)
        .collect::<Vec<String>>();

    loop {
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                .split(f.size());

            let style = Style::default();

            let items = repos.iter().map(|i| Text::raw(format!("{}", i)));
            let items = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Repositories"))
                .style(style)
                .highlight_style(style.fg(Color::LightGreen).modifier(Modifier::BOLD))
                .highlight_symbol(">");

            f.render_widget(items, chunks[0]);
        })?;
    }
}
