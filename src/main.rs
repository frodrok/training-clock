use std::io::stdout;
use std::{thread, time::Duration, time::Instant};

use std::io;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{BarChart, Block, BorderType, Borders, Gauge},
    Frame, Terminal,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};

use soloud::*;
use tui::widgets::Paragraph;

#[macro_use]
extern crate crossterm;

struct AppState {
    running_timer: bool,
    play_sound: Wav,
    timer_value: String,
    start_time: Instant,
    wait_time: Duration,
    input_mode: InputMode,
}

enum InputMode {
    Normal,
    Editing,
}

fn main() -> Result<(), io::Error> {
    let mut stdout = stdout();

    enable_raw_mode()?;

    //    beep(440);
    //    print!("\x07");

    //clearing the screen, going to top left corner and printing welcoming message
    /* execute!(
        stdout,
        Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        Print(
            r#"ctrl + q to exit, ctrl + h to print "Hello world", alt + t to print "crossterm is cool""#
        )
    )?; */

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let sl = Soloud::default().unwrap();
    let mut wav = audio::Wav::default();
    wav.load_mem(include_bytes!("../go.wav")).unwrap();

    let wait_time: Duration = Duration::from_secs(60);

    let mut app_state = AppState {
        running_timer: false,
        play_sound: wav,
        timer_value: "60".to_string(),
        start_time: Instant::now(),
        wait_time: wait_time,
        input_mode: InputMode::Normal,
    };

    loop {
        if !app_state.running_timer {
            let _ = terminal.draw(|f| sleeping_ui(f, &app_state));
        } else {
            'timer: loop {
                //            execute!(stdout, cursor::MoveTo(0, 0)).unwrap();
                let elapsed = app_state.start_time.elapsed().as_secs();
                let time_string = elapsed.to_string();
                let progress: f64 = elapsed as f64 / app_state.wait_time.as_secs() as f64;
                //let progress: f64 = (elapsed / app_state.wait_time.as_secs()) as f64;
                //          execute!(stdout, Clear(ClearType::All), Print(time_string)).unwrap();

                //println!("elapsed {} / wait_time {} = progress {}", elapsed, app_state.wait_time.as_secs(), progress);

                let _ = terminal.draw(|f| {
                    timer_ui(
                        f,
                        time_string.clone(),
                        app_state.running_timer.clone(),
                        progress,
                    )
                });

                if elapsed >= app_state.wait_time.as_secs() {
                    //   execute!(stdout, Clear(ClearType::All), Print("\x07")).unwrap();

                    //sl.play(&wav);
                    sl.play(&app_state.play_sound);

                    while sl.voice_count() > 0 {
                        thread::sleep(std::time::Duration::from_millis(100));
                    }

                    app_state.running_timer = false;

                    let _ = terminal.draw(|t| {
                        timer_ui(
                            t,
                            time_string.clone(),
                            app_state.running_timer.clone(),
                            progress,
                        )
                    });

                    break 'timer;
                }

                thread::sleep(Duration::from_millis(250));
            }
        }

        if let Event::Key(key) = event::read()? {
            match app_state.input_mode {
                InputMode::Normal => match key {
                    KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::CONTROL,
                    } => {
                        break;
                    }
                    KeyEvent {
                        code: KeyCode::Char('e'),
                        ..
                    } => app_state.input_mode = InputMode::Editing,
                    KeyEvent {
                        code: KeyCode::Char(' '),
                        ..
                    } => {
                        app_state.start_time = Instant::now();
                        app_state.running_timer = true;
                    }

                    _ => {}
                },
                InputMode::Editing => match key {
                    KeyEvent {
                        code: KeyCode::Esc, ..
                    } => app_state.input_mode = InputMode::Normal,
                    KeyEvent {
                        code: KeyCode::Char(c),
                        ..
                    } => {
                        app_state.timer_value.push(c);
                    }
                    KeyEvent {
                        code: KeyCode::Backspace,
                        ..
                    } => {
                        app_state.timer_value.pop();
                    }
                    KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    } => {
                        let timer_seconds: u32 = match app_state.timer_value.parse::<u32>() {
                            Ok(v) => v,
                            Err(_e) => 60,
                        };

                        app_state.wait_time = Duration::from_secs(timer_seconds as u64);
                        app_state.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
            }
        }

        thread::sleep(Duration::from_millis(100));
    }

    disable_raw_mode().unwrap();

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .unwrap();

    terminal.show_cursor().unwrap();

    Ok(())
}

fn sleeping_ui<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let size = f.size();

    let block = Block::default()
        .title("TIMER")
        .borders(Borders::ALL)
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);

    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let top_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[0]);

    let block = Block::default()
        .title(vec![
            //                        Span::styled("With", Style::default().fg(Color::Yellow)),
            Span::from("CTRL-Q to quit, SPACE to start timer, E to edit timer then Enter to confirm or Escape to quit editing"),
        ])
        .style(Style::default().bg(Color::Green));

    // Show quotes around to display spaces
    let timer_text = "\'".to_owned() + &app_state.timer_value + "\'";
    //let input = Paragraph::new(app_state.timer_value.as_ref())
    let input = Paragraph::new(timer_text)
        .style(Style::default())
        .block(Block::default().borders(Borders::ALL).title("Seconds"));

    f.render_widget(block, top_chunks[0]);
    f.render_widget(input, top_chunks[1]);
}

fn progress_to_bar_chart_data(progress: f64) -> &'static [(&'static str, u64)] {
    if progress < 0.25 {
        &[("B0", 10), ("B1", 10), ("B2", 10), ("B3", 10)]
    } else if progress < 0.5 {
        &[
            ("B0", 10),
            ("B1", 10),
            ("B2", 10),
            ("B3", 10),
            ("B0", 40),
            ("B1", 40),
            ("B2", 40),
            ("B3", 40),
        ]
    } else if progress < 0.75 {
        &[
            ("B0", 10),
            ("B1", 10),
            ("B2", 10),
            ("B3", 10),
            ("B0", 40),
            ("B1", 40),
            ("B2", 40),
            ("B3", 40),
            ("B0", 60),
            ("B1", 60),
            ("B2", 60),
            ("B3", 60),
        ]
    } else if progress < 100.0 {
        &[
            ("B0", 10),
            ("B1", 10),
            ("B2", 10),
            ("B3", 10),
            ("B0", 40),
            ("B1", 40),
            ("B2", 40),
            ("B3", 40),
            ("B0", 60),
            ("B1", 60),
            ("B2", 60),
            ("B3", 60),
            ("B0", 90),
            ("B1", 90),
            ("B2", 90),
            ("B3", 90),
        ]
    } else {
        &[
            ("B0", 10),
            ("B1", 10),
            ("B2", 10),
            ("B3", 10),
            ("B0", 40),
            ("B1", 40),
            ("B2", 40),
            ("B3", 40),
            ("B0", 60),
            ("B1", 60),
            ("B2", 60),
            ("B3", 60),
            ("B0", 90),
            ("B1", 90),
            ("B2", 90),
            ("B3", 90),
            ("B0", 100),
            ("B1", 100),
            ("B2", 100),
            ("B3", 100),
        ]
    }
}

fn timer_ui<B: Backend>(
    f: &mut Frame<B>,
    time_string: String,
    waiting_for_start: bool,
    progress: f64,
) {
    let size = f.size();

    let block = Block::default()
        .title("TIMER")
        .borders(Borders::ALL)
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);

    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let top_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[0]);

    let start_string = (waiting_for_start).to_string();

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Progress"))
        .gauge_style(
            Style::default()
                .fg(Color::Green)
                .bg(Color::Black)
                .add_modifier(Modifier::ITALIC),
        )
        .ratio(progress);

    let bar_chart_data = progress_to_bar_chart_data(progress);

    let bar_chart = BarChart::default()
        .block(Block::default().title("Progress").borders(Borders::ALL))
        .bar_width(3)
        .bar_gap(1)
        .bar_style(Style::default().fg(Color::Yellow).bg(Color::Red))
        .value_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .label_style(Style::default().fg(Color::White))
        //.data(&[("B0", 0), ("B1", 2), ("B2", 4), ("B3", 3)])
        .data(bar_chart_data)
        .max(100);

    let block = Block::default()
        .title(vec![
            //                        Span::styled("With", Style::default().fg(Color::Yellow)),
            Span::from("Debug info: ".to_string()),
            Span::from(time_string),
            Span::from(" ".to_string()),
            Span::from(start_string),
            Span::from(" ".to_string()),
            Span::from(progress.to_string()),
        ])
        .style(Style::default().bg(Color::Green));

    f.render_widget(block, top_chunks[0]);
    f.render_widget(gauge, top_chunks[1]);
    f.render_widget(bar_chart, chunks[1]);
}
