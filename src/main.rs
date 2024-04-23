use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fakeit::data;
use ratatui::{prelude::*, widgets::{block::Title, Axis, Block, BorderType, Borders, Chart, Dataset, GraphType, LegendPosition, Paragraph}, };
use ratatui::widgets::*;
use std::{collections::VecDeque, vec};
use std::{error::Error, io};
use tokio::*;
use tokio::time::interval;
use std::result;
use std::net::ToSocketAddrs;
use std::time::{Instant, Duration};
use std::sync::Mutex;
use sysinfo::*;
// mod system_functions;

pub static PING_DATA: Mutex<VecDeque<f64>> = Mutex::new(VecDeque::new()); // Ping times
pub static DATA_SENT: Mutex<VecDeque<i64>> = Mutex::new(VecDeque::new()); // Total bytes sent
pub static DATA_RECIEVED: Mutex<VecDeque<i64>> = Mutex::new(VecDeque::new()); // Total bytes received

const MAX_PING_DATA_POINTS: usize = 150;
const MAX_UPLOAD_DOWNLOAD_DATA_POINTS: usize = 50;

// TODO: Add a function to get the total bytes received across all interfaces
//          - Step 1: get total bytes sent and recieved at the beginning
//          - Step 2: get total bytes sent and recieved at each interval
//          - Step 3: subtract and divide by the interval time to get the average bytes sent and recieved

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut window = [0.0, 0.0];
    let query = String::from("google.com");

    let mut interval1 = time::interval(Duration::from_millis(100));
    tokio::spawn(async move { // PING DATA
        loop {
            interval1.tick().await;
            let ping_time = ping_website(&query).await;
            PING_DATA.lock().unwrap().push_total(ping_time.unwrap().as_millis() as f64);
        }
    });

    let mut interval2 = time::interval(Duration::from_millis(1000));
    tokio::spawn(async move { // TOTAL BYTES SENT AND RECEIVED
        let mut last_upload_download = get_total_bytes().await;
        loop {
            interval2.tick().await;
            let bytes = get_total_bytes().await;
            let sub = (bytes.0 - last_upload_download.0, bytes.1 - last_upload_download.1);
            last_upload_download = bytes;
            DATA_SENT.lock().unwrap().push_total(sub.0 as i64);
            DATA_RECIEVED.lock().unwrap().push_total(sub.1 as i64);
        }
    });

    run().await?;
    
    crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

async fn get_total_bytes() -> (u64, u64) { // (sent, received)
    let mut system = System::new_all();
    system.refresh_all();
    let mut total_outcome :u64 = 0;
    let mut total_income :u64 = 0;
    let mut starting_outcome :u64 = 0;
    let mut starting_income :u64 = 0;
    let mut network_data = Networks::new_with_refreshed_list();
    network_data.iter().for_each(|(name, data)| {
        total_outcome += data.total_received();
        total_income += data.total_transmitted();
    });
    (total_outcome, total_income)
}

pub trait PushTotal<T> {
    fn push_total(&mut self, new_value: T);
}

impl<T> PushTotal<T> for VecDeque<T>
where
    T: Copy + Default,
{
    fn push_total(&mut self, new_value: T) {
        self.push_back(new_value);
        if self.len() > MAX_PING_DATA_POINTS {
            self.pop_front();
        }
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    loop {
        t.draw(|f| {
                let area = f.size();
                let vertical = Layout::vertical([Constraint::Percentage(40), Constraint::Percentage(60)]);
                let horizontal = Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]);
                let [top_rect, bottom_rect] = vertical.areas(area);
                let [bottom_left, bottom_right] = horizontal.areas(bottom_rect);
                render_ping_chart(f, top_rect);
                render_upload_chart(f, bottom_left);
                render_download_chart(f, bottom_right);
        })?;

    }
    Ok(())
}

fn render_upload_chart(f: &mut Frame, area: Rect) {
    let mut data : Vec<(f64, f64)> = Vec::new();
    let clone = DATA_SENT.lock().unwrap().clone();
    let mut last_value : String = String::from("0");
    for i in 0..clone.len() {
        data.push((i as f64, clone[i] as f64));
        last_value = clone[i].to_string(); // make this more efficient later
    }
    last_value = format!("Upload: {} B/s", last_value);
    let datasets = vec![Dataset::default()
        .marker(symbols::Marker::Dot)
        .style(Style::default().fg(Color::Green))
        .graph_type(GraphType::Line)
        .data(&data)];
    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(
                    Title::default()
                        .content(last_value.blue().bold())
                        .alignment(Alignment::Center),
                )
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .bounds([0.0, MAX_UPLOAD_DOWNLOAD_DATA_POINTS as f64])
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, 15000.0])
                .labels(vec!["0".bold(), "-".into(), "-".into(),  "15000".bold()]),
        )
        .legend_position(Some(LegendPosition::TopLeft))
        .hidden_legend_constraints((Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)));
    f.render_widget(chart, area);
}

fn render_download_chart(f: &mut Frame, area: Rect) {
    let mut data : Vec<(f64, f64)> = Vec::new();
    let clone = DATA_RECIEVED.lock().unwrap().clone();
    let mut last_value : String = String::from("0");
    for i in 0..clone.len() {
        data.push((i as f64, clone[i] as f64));
        last_value = clone[i].to_string(); // make this more efficient later
    }
    last_value = format!("Download: {} B/s", last_value);
    let datasets = vec![Dataset::default()
        .marker(symbols::Marker::Dot)
        .style(Style::default().fg(Color::Red))
        .graph_type(GraphType::Line)
        .data(&data)];
    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(
                    Title::default()
                        .content(last_value.blue().bold())
                        .alignment(Alignment::Center),
                )
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .bounds([0.0, MAX_UPLOAD_DOWNLOAD_DATA_POINTS as f64])
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, 15000.0])
                .labels(vec!["0".bold(), "-".into(), "-".into(),  "15000".bold()]),
        )
        .legend_position(Some(LegendPosition::TopLeft))
        .hidden_legend_constraints((Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)));
    f.render_widget(chart, area);
}

async fn ping_website(site: &str) -> io::Result<Duration> {
    let addr = format!("{}:80", site);
    let addr = match addr.to_socket_addrs() {
        Ok(mut addrs) => match addrs.next() {
            Some(addr) => addr,
            None => return Err(io::Error::new(io::ErrorKind::NotFound, "No addresses found")),
        },
        Err(_) => return Err(io::Error::new(io::ErrorKind::InvalidInput, "There was an error. Please unsure you're connected to the internet.")),
    };
    let start = Instant::now();
    tokio::net::TcpStream::connect(&addr).await?;
    Ok(start.elapsed())
}

fn render_ping_chart(f: &mut Frame, area: Rect) {

    let mut data : Vec<(f64, f64)> = Vec::new();
    let clone = PING_DATA.lock().unwrap().clone();
    let mut last_value : String = String::from("0");
    for i in 0..clone.len() {
        data.push((i as f64, clone[i]));
        last_value = clone[i].to_string(); // make this more efficient later
    }
    last_value = format!("Ping: {}ms", last_value);
    let datasets = vec![Dataset::default()
        .marker(symbols::Marker::Dot)
        .style(Style::default().fg(Color::LightBlue))
        .graph_type(GraphType::Line)
        
        .data(&data)];
    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(
                    Title::default()
                        .content(last_value.blue().bold())
                        .alignment(Alignment::Center),
                )
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .bounds([0.0, MAX_PING_DATA_POINTS as f64])
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, 200.0])
                .labels(vec!["0".bold(), "-".into(), "100".bold()]),
        )
        .legend_position(Some(LegendPosition::TopLeft))
        .hidden_legend_constraints((Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)));
    f.render_widget(chart, area);
}