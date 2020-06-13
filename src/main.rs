use std::collections::HashMap;
use std::io;

use termion::{event::Key, raw::IntoRawMode};
use tui::{
    backend::TermionBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{BarChart, Block, Borders, List, Paragraph, Text},
    Terminal,
};

mod bitbucket;
use bitbucket::BitBucket;

mod event;
use event::{Event, Events};

mod utils;
use serde_json::Value;
use utils::StatefulList;

struct App {
    storage: BitBucket,

    repositories: StatefulList<String>,
    reviewers: Vec<(String, u64)>,
    data: Vec<Value>,
}

impl App {
    fn new() -> Self {
        let mut args = std::env::args();
        args.next();

        let user = std::env::var("TUI_PR_USER")
            .unwrap_or_else(|_| args.next().expect("first argument must be username"));
        let password = std::env::var("TUI_PR_PASSWORD")
            .unwrap_or_else(|_| args.next().expect("second argument must be password"));
        let host = std::env::var("TUI_PR_HOST")
            .unwrap_or_else(|_| args.next().expect("third argument must be host"));
        let project = std::env::var("TUI_PR_PROJECT")
            .unwrap_or_else(|_| args.next().expect("fourth argument must be project"));

        let storage = BitBucket::new(&user, &password, &host, &project);

        App {
            storage,
            repositories: StatefulList::new(),
            reviewers: Vec::new(),
            data: Vec::new(),
        }
    }

    async fn download_repositories(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let repos = self
            .storage
            .request_repos()
            .await?
            .iter()
            .filter_map(|r| r["name"].as_str())
            .map(String::from)
            .collect();

        self.repositories = StatefulList::with_items(repos);

        Ok(())
    }

    async fn download_reviwers(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.repositories.selected() {
            Some(i) => match self.repositories.items.get(i) {
                Some(repo_name) => {
                    let mut frequency: HashMap<String, u64> = HashMap::new();
                    self.storage
                        .request_pr_data(repo_name)
                        .await?
                        .iter()
                        .filter_map(|pr| pr["reviewers"].as_array())
                        .flatten()
                        .filter_map(|reviwer| reviwer["user"]["displayName"].as_str())
                        .for_each(|reviewer_name| {
                            let name: Vec<&str> = reviewer_name.split(" ").collect();
                            let name = format!("{} {}", name[0], name[1]);
                            let counter = frequency.entry(name).or_insert(0);
                            *counter += 1;
                        });

                    self.reviewers = frequency.into_iter().map(|x| x).collect();
                    self.reviewers.sort_by(|x, y| y.1.cmp(&x.1))
                }
                None => {}
            },
            None => {}
        };

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    let mut app = App::new();
    app.download_repositories().await?;

    let events = Events::new();
    loop {
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                .split(f.size());

            let style = Style::default();
            let items = app
                .repositories
                .items
                .iter()
                .map(|i| Text::raw(format!("{}", i)));

            let repo_list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Repositories"))
                .style(style)
                .highlight_style(style.fg(Color::LightGreen).modifier(Modifier::BOLD))
                .highlight_symbol(">");

            let reviewers: Vec<(&str, u64)> =
                app.reviewers.iter().map(|x| (&x.0[..], x.1)).collect();
            let barchart = BarChart::default()
                .block(
                    Block::default()
                        .title("Reviewers stats")
                        .borders(Borders::ALL),
                )
                .data(&reviewers)
                .label_style(Style::default())
                .bar_width(15)
                .bar_gap(3)
                .style(Style::default().fg(Color::Green))
                .value_style(Style::default().bg(Color::Green).modifier(Modifier::BOLD));

            f.render_widget(barchart, chunks[1]);
            f.render_stateful_widget(repo_list, chunks[0], &mut app.repositories.state);
        })?;

        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => break,
                Key::Left => app.repositories.unselect(),
                Key::Down => app.repositories.next(),
                Key::Up => app.repositories.previous(),
                Key::Right => app.download_reviwers().await?,
                _ => {}
            },
            Event::Tick => {}
        };
    }
    Ok(())
}
