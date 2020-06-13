use std::io;

use termion::{event::Key, raw::IntoRawMode};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, Text},
    Terminal,
};

mod bitbucket;
use bitbucket::BitBucket;

mod event;
use event::{Event, Events};

mod utils;
use utils::StatefulList;

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

        std::env::vars().for_each(|x| println!("{}:{}", x.0, x.1));
        let user = std::env::var("TUI_PR_USER")
            .unwrap_or_else(|x| args.next().expect("first argument must be username"));
        let password = std::env::var("TUI_PR_PASSWORD")
            .unwrap_or_else(|x| args.next().expect("second argument must be password"));
        let host = std::env::var("TUI_PR_HOST")
            .unwrap_or_else(|x| args.next().expect("third argument must be host"));
        let project = std::env::var("TUI_PR_PROJECT")
            .unwrap_or_else(|x| args.next().expect("fourth argument must be project"));

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

    let mut repos = StatefulList::with_items(repos);
    let events = Events::new();

    loop {
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                .split(f.size());

            let style = Style::default();
            let items = repos.items.iter().map(|i| Text::raw(format!("{}", i)));
            let repo_list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Repositories"))
                .style(style)
                .highlight_style(style.fg(Color::LightGreen).modifier(Modifier::BOLD))
                .highlight_symbol(">");

            
            f.render_stateful_widget(repo_list, chunks[0], &mut repos.state);
        })?;

        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Left => repos.unselect(),
                Key::Down => {
                    repos.next();
                }
                Key::Up => {
                    repos.previous();
                }
                _ => {}
            },
            Event::Tick => {}
        };
    }
    Ok(())
}
