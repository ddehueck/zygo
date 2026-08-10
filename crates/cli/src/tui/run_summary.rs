use std::time::{Duration, SystemTime};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Padding, Paragraph, Row, StatefulWidget, Table,
        TableState, Widget,
    },
};

use crate::commands::{JobRunSummary, WorkflowRunSummary};

use super::format_duration;

const OVERVIEW_HEIGHT: u16 = 6;
const JOB_TABLE_ROW_START: u16 = OVERVIEW_HEIGHT + 2;

pub struct WorkflowRunView<'a> {
    summary: &'a WorkflowRunSummary,
    target: &'a str,
    input_uri: &'a str,
    can_quit: bool,
}

impl<'a> WorkflowRunView<'a> {
    pub fn new(
        summary: &'a WorkflowRunSummary,
        target: &'a str,
        input_uri: &'a str,
        can_quit: bool,
    ) -> Self {
        Self {
            summary,
            target,
            input_uri,
            can_quit,
        }
    }
}

impl StatefulWidget for WorkflowRunView<'_> {
    type State = TableState;

    fn render(self, area: Rect, buffer: &mut Buffer, state: &mut Self::State) {
        let [overview_area, jobs_area] =
            Layout::vertical([Constraint::Length(OVERVIEW_HEIGHT), Constraint::Fill(1)])
                .areas(area);

        let label_style = Style::default().fg(Color::DarkGray);
        let value_style = Style::default().add_modifier(Modifier::BOLD);
        let overview = vec![
            Line::from(vec![
                Span::styled("Workflow    ", label_style),
                Span::styled(self.summary.workflow_id.clone(), value_style),
            ]),
            Line::from(vec![
                Span::styled("Status      ", label_style),
                Span::styled(
                    self.summary.workflow_status.clone(),
                    status_style(&self.summary.workflow_status),
                ),
            ]),
            Line::from(vec![
                Span::styled("Target      ", label_style),
                Span::raw(self.target.to_owned()),
            ]),
            Line::from(vec![
                Span::styled("Input file  ", label_style),
                Span::raw(self.input_uri.to_owned()),
            ]),
        ];
        let block = Block::default()
            .title(Line::from(Span::styled(
                " Overview ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray))
            .padding(Padding::horizontal(1));
        Paragraph::new(overview)
            .block(block)
            .render(overview_area, buffer);

        let header = Row::new(["Status", "Job", "Duration", "Logs"])
            .style(Style::default().add_modifier(Modifier::BOLD));
        let rows = self.summary.job_runs.iter().map(job_run_row);
        let widths = [
            Constraint::Length(10),
            Constraint::Fill(1),
            Constraint::Length(14),
            Constraint::Length(16),
        ];
        let footer = if self.can_quit {
            " ↑/↓ select • Enter open logs • q quit "
        } else {
            " ↑/↓ select • Enter open logs "
        };
        let jobs_block = Block::default()
            .title(Line::from(Span::styled(
                format!(" Job Runs  ({}) ", self.summary.job_runs.len()),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )))
            .title_bottom(Line::from(Span::styled(
                footer,
                Style::default().fg(Color::DarkGray),
            )))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray))
            .padding(Padding::horizontal(1));
        let table = Table::new(rows, widths)
            .header(header)
            .column_spacing(2)
            .row_highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("› ")
            .block(jobs_block);

        StatefulWidget::render(table, jobs_area, buffer, state);
    }
}

pub struct JobLogView<'a> {
    job_id: &'a str,
    job_run_id: &'a str,
    contents: &'a str,
    is_running: bool,
}

impl<'a> JobLogView<'a> {
    pub fn new(job_id: &'a str, job_run_id: &'a str, contents: &'a str, is_running: bool) -> Self {
        Self {
            job_id,
            job_run_id,
            contents,
            is_running,
        }
    }
}

impl Widget for JobLogView<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let mode = if self.is_running {
            "Watching logs"
        } else {
            "Logs"
        };
        let title = format!(
            " {mode} — {} ({}) ",
            self.job_id,
            last_chars(self.job_run_id, 5)
        );
        let block = Block::default()
            .title(Line::from(Span::styled(
                title,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )))
            .title_bottom(Line::from(Span::styled(
                " Esc back ",
                Style::default().fg(Color::DarkGray),
            )))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray))
            .padding(Padding::horizontal(1));
        let visible_lines = usize::from(area.height.saturating_sub(2));
        let line_count = self.contents.lines().count();
        let scroll = line_count
            .saturating_sub(visible_lines)
            .min(usize::from(u16::MAX)) as u16;

        Paragraph::new(self.contents)
            .scroll((scroll, 0))
            .block(block)
            .render(area, buffer);
    }
}

pub fn job_run_at_position(
    area: Rect,
    table_offset: usize,
    column: u16,
    row: u16,
    job_run_count: usize,
) -> Option<usize> {
    let right = area.x.saturating_add(area.width);
    let bottom = area.y.saturating_add(area.height);
    let first_row = area.y.saturating_add(JOB_TABLE_ROW_START);
    let last_row_exclusive = bottom.saturating_sub(1);

    if column < area.x || column >= right || row < first_row || row >= last_row_exclusive {
        return None;
    }

    let index = table_offset.saturating_add(usize::from(row - first_row));
    (index < job_run_count).then_some(index)
}

fn job_run_row(job_run: &JobRunSummary) -> Row<'static> {
    let job_run_id = last_chars(&job_run.job_run_id, 5);
    let job = format!("{} ({job_run_id})", job_run.job_id);
    let duration = job_run_duration(job_run);
    let status = Cell::from(job_run.status.clone()).style(status_style(&job_run.status));
    let action = if job_run.status == "running" {
        "[ Watch logs ]"
    } else {
        "[ View logs ]"
    };

    Row::new([
        status,
        Cell::from(job),
        Cell::from(duration),
        Cell::from(action).style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ])
}

fn job_run_duration(job_run: &JobRunSummary) -> String {
    let Some(started_at) = job_run.started_at else {
        return "—".to_owned();
    };
    let ended_at = job_run.ended_at.unwrap_or_else(SystemTime::now);
    let duration = ended_at
        .duration_since(started_at)
        .unwrap_or(Duration::ZERO);

    format_duration(duration)
}

fn status_style(status: &str) -> Style {
    let color = match status {
        "running" => Color::Yellow,
        "succeeded" => Color::Green,
        "failed" => Color::Red,
        _ => Color::Reset,
    };

    Style::default().fg(color).add_modifier(Modifier::BOLD)
}

fn last_chars(value: &str, count: usize) -> &str {
    if count == 0 {
        return "";
    }

    let start = value
        .char_indices()
        .rev()
        .nth(count - 1)
        .map_or(0, |(index, _)| index);
    &value[start..]
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;

    use super::{job_run_at_position, last_chars};

    #[test]
    fn shortens_job_run_ids_from_the_end() {
        assert_eq!(last_chars("job-run-abcde", 5), "abcde");
        assert_eq!(last_chars("abc", 5), "abc");
        assert_eq!(last_chars("abc", 0), "");
    }

    #[test]
    fn maps_visible_table_rows_to_job_runs() {
        let area = Rect::new(0, 10, 80, 20);

        assert_eq!(job_run_at_position(area, 2, 10, 18, 10), Some(2));
        assert_eq!(job_run_at_position(area, 2, 10, 20, 10), Some(4));
        assert_eq!(job_run_at_position(area, 2, 10, 17, 10), None);
        assert_eq!(job_run_at_position(area, 2, 80, 18, 10), None);
    }
}
