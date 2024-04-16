use clap::Parser;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, sync::mpsc, thread};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::Color,
    text::{Span, Spans},
    widgets::{
        canvas::{Canvas, Line, Map, MapResolution},
        Block, Borders, Paragraph,
    },
    Terminal,
};

mod ip_fetcher;

use ip_fetcher::{FetchState, IpFetcher};

/// Get Details of an ip address
#[derive(Parser)]
#[command(version)]
struct Cli {
    /// ipv4 or ipv6 address or url of target
    target: String,
}

fn render<B: Backend>(
    terminal: &mut Terminal<B>,
    fetched_details: &IpFetcher,
    fetch_state: &FetchState,
) -> Result<(), io::Error> {
    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
            .split(f.size());
        let map = Canvas::default()
            .block(Block::default().title("Ip Location").borders(Borders::ALL))
            .paint(|ctx| {
                ctx.draw(&Map {
                    color: Color::Red,
                    resolution: MapResolution::High,
                });
                if *fetch_state == FetchState::Success {
                    let details = fetched_details.details.as_ref().unwrap();
                    ctx.layer();
                    ctx.draw(&Line {
                        x1: -180.0,
                        y1: details.lat,
                        x2: 180.0,
                        y2: details.lat,
                        color: Color::Green,
                    });
                    ctx.draw(&Line {
                        x1: details.lon,
                        y1: 90.0,
                        x2: details.lon,
                        y2: -90.0,
                        color: Color::Green,
                    });
                }
            })
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0]);
        f.render_widget(map, chunks[0]);
        match *fetch_state {
            FetchState::Pending => {
                let paragraph = Paragraph::new(vec![
                    Spans::from(vec![Span::raw("")]),
                    Spans::from(vec![Span::raw("  Fetching details")]),
                ])
                .block(Block::default().title("Details").borders(Borders::ALL));
                f.render_widget(paragraph, chunks[1]);
            }
            FetchState::Failure => {
                let paragraph = Paragraph::new(vec![
                    Spans::from(vec![Span::raw("")]),
                    Spans::from(vec![Span::raw("  Error while fetching details")]),
                ])
                .block(Block::default().title("Details").borders(Borders::ALL));
                f.render_widget(paragraph, chunks[1]);
            }
            FetchState::Success => {
                let details = fetched_details.details.as_ref().unwrap();
                let paragraph = Paragraph::new(vec![
                    Spans::from(vec![Span::raw("")]),
                    Spans::from(vec![Span::raw(format!("  IP Address: {}", details.query))]),
                    Spans::from(vec![Span::raw("")]),
                    Spans::from(vec![Span::raw(format!("  Lattitude: {}", details.lat))]),
                    Spans::from(vec![Span::raw("")]),
                    Spans::from(vec![Span::raw(format!("  Longittude: {}", details.lon))]),
                    Spans::from(vec![Span::raw("")]),
                    Spans::from(vec![Span::raw(format!("  Country: {}", details.country))]),
                    Spans::from(vec![Span::raw("")]),
                    Spans::from(vec![Span::raw(format!(
                        "  Country Code: {}",
                        details.country_code
                    ))]),
                    Spans::from(vec![Span::raw("")]),
                    Spans::from(vec![Span::raw(format!("  Region: {}", details.region))]),
                    Spans::from(vec![Span::raw("")]),
                    Spans::from(vec![Span::raw(format!(
                        "  Region Name: {}",
                        details.region_name
                    ))]),
                    Spans::from(vec![Span::raw("")]),
                    Spans::from(vec![Span::raw(format!("  City: {}", details.city))]),
                    Spans::from(vec![Span::raw("")]),
                    Spans::from(vec![Span::raw(format!("  Zip: {}", details.zip))]),
                    Spans::from(vec![Span::raw("")]),
                    Spans::from(vec![Span::raw(format!(
                        "  Time zone: {}",
                        details.timezone
                    ))]),
                    Spans::from(vec![Span::raw("")]),
                    Spans::from(vec![Span::raw(format!("  Organisation: {}", details.org))]),
                    Spans::from(vec![Span::raw("")]),
                    Spans::from(vec![Span::raw(format!("  ISP: {}", details.isp))]),
                ])
                .block(Block::default().title("Details").borders(Borders::ALL));
                f.render_widget(paragraph, chunks[1]);
            }
        }
    })?;
    Ok(())
}

fn main() -> Result<(), io::Error> {
    let cli = Cli::parse();

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        if let Ok(ipfetcher) = IpFetcher::fetch(&cli.target) {
            tx.send(ipfetcher).unwrap();
        }
    });

    let mut fetch_state = FetchState::Pending;
    let mut fetched_details = IpFetcher { details: None };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    render(&mut terminal, &fetched_details, &fetch_state)?;

    if let Ok(message) = rx.recv() {
        fetched_details = message;
        fetch_state = FetchState::Success;
    } else {
        fetch_state = FetchState::Failure;
    }

    render(&mut terminal, &fetched_details, &fetch_state)?;

    loop {
        if let Event::Key(_) = event::read()? {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
