use crate::command_line_interface::load_test_visualizer::LoadtestVisualizer;
use tui::layout::Rect;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset},
    Frame,
};

pub(crate) fn draw_plots<B: Backend>(frame: &mut Frame<B>, visualizer: &LoadtestVisualizer) {
    let panes = create_three_plots(frame);
    frame.render_widget(
        display_request_rate(visualizer),
        *panes.get(0).expect("One Pane has to exist."),
    );
}

pub(crate) fn create_three_plots<B: Backend>(frame: &mut Frame<B>) -> Vec<Rect> {
    let size = frame.size();
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(1, 3)].as_ref())
        .split(size)
}

pub(crate) fn generate_lable_time_axis(visualizer: &LoadtestVisualizer) -> Vec<Span> {
    vec![
        Span::styled(
            format!("{}", visualizer.window[0]),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(
            "{}",
            (visualizer.window[0] + visualizer.window[1]) / 2.0
        )),
        Span::styled(
            format!("{}", visualizer.window[1]),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ]
}

fn display_request_rate(visualizer: &LoadtestVisualizer) -> tui::widgets::Chart {
    let datasets = vec![Dataset::default()
        .name("request_per_second")
        .marker(symbols::Marker::Dot)
        .style(Style::default().fg(Color::Cyan))
        .data(&visualizer.requests_per_second)];

    Chart::new(datasets)
        .block(
            Block::default()
                .title(Span::styled(
                    "Request per Second",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .title("Time")
                .style(Style::default().fg(Color::Gray))
                .labels(generate_lable_time_axis(visualizer))
                .bounds(visualizer.window),
        )
        .y_axis(
            Axis::default()
                .title("Requests")
                .style(Style::default().fg(Color::Gray))
                .labels(vec![
                    Span::styled("-20", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
                ])
                .bounds([0.0, 20.0]),
        )
}
