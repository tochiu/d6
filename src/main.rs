extern crate log;

mod raster;
mod transform;
mod world;
mod body;

use world::*;
use raster::*;

use tui_logger::{TuiLoggerWidget, TuiLoggerLevelOutput};
use tui::{backend::CrosstermBackend, layout::{Layout, Direction, Constraint}, widgets::{Borders, Block}, style::{Style, Color}};
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::{io::stdout, time::{Duration, Instant}};

fn main() {
    tui_logger::init_logger(log::LevelFilter::Trace).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Trace);

    enable_raw_mode().expect("Failed to enable terminal raw mode");

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen).expect("Failed to enter alternate terminal screen");

    let mut terminal = tui::Terminal::new(CrosstermBackend::new(stdout))
        .expect("Failed to create interface to terminal backend");

    let mut test_world = test_world::TestWorld::new();
    let mut raster = Raster::default();
    let mut then = Instant::now();

    loop {
        if poll(Duration::from_micros(16666)).unwrap() {
            match read().unwrap() {
                Event::Key(key_event) => {
                    // Esc or Crtl+C interrupt handler
                    if key_event.code == KeyCode::Esc // Esc is an exit if debugger isnt sinking keys
                        || key_event.modifiers.contains(KeyModifiers::CONTROL) // Ctrl+C is a hard exit
                            && (key_event.code == KeyCode::Char('c')
                                || key_event.code == KeyCode::Char('C'))
                    {
                        disable_raw_mode().expect("Failed to disable terminal raw mode");
                        execute!(terminal.backend_mut(), LeaveAlternateScreen)
                            .expect("Failed to leave alternate terminal screen");
                        terminal
                            .show_cursor()
                            .expect("Failed to show terminal cursor");
                        return;
                    }
                }
                _ => (),
            }
        }

        let now = Instant::now();
        test_world.update(now.duration_since(then).as_secs_f64());
        then = now;

        terminal
            .draw(|frame| {
                let [raster_area, logger_area, ..] = Layout::default()
                    .constraints([
                        Constraint::Percentage(50), 
                        Constraint::Percentage(50)
                    ])
                    .direction(Direction::Horizontal)
                    .split(frame.size())[..] else { unreachable!() }; 

                frame.render_widget(
                    RasterWidget::<'_, _, OrthographicCamera>::new(&mut raster, &test_world, 4),
                    raster_area,
                );
                frame.render_widget(logger_widget(Borders::ALL), logger_area);
            })
            .expect("Failed to draw to terminal");
    }
}

pub fn logger_widget(borders: Borders) -> TuiLoggerWidget<'static> {
    TuiLoggerWidget::default()
        .block(
            Block::default()
                .title(" Log ")
                .border_style(Style::default().fg(Color::White))
                .borders(borders),
        )
        .output_separator('|')
        .output_timestamp(Some("%H:%M:%S%.3f".to_string()))
        .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
        .output_target(false)
        .output_file(false)
        .output_line(false)
        .style_error(Style::default().fg(Color::Red))
        .style_debug(Style::default().fg(Color::Cyan))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_trace(Style::default().fg(Color::White))
        .style_info(Style::default().fg(Color::Green))
}