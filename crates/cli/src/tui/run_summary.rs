use std::time::{Duration, SystemTime};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Padding, Paragraph, Row, Table, Widget},
};

use crate::commands::{JobRunSummary, WorkflowRunSummary};

use super::format_duration;

pub struct WorkflowRunView<'a> {
    summary: &'a WorkflowRunSummary,
    target: &'a str,
    input_uri: &'a str,
}

impl<'a> WorkflowRunView<'a> {
    pub fn new(summary: &'a WorkflowRunSummary, target: &'a str, input_uri: &'a str) -> Self {
        Self {
            summary,
            target,
            input_uri,
        }
    }
}

impl Widget for WorkflowRunView<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [overview_area, jobs_area] =
            Layout::vertical([Constraint::Length(6), Constraint::Fill(1)]).areas(area);

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

        let header = Row::new(["Status", "Job", "Duration"])
            .style(Style::default().add_modifier(Modifier::BOLD));
        let rows = self.summary.job_runs.iter().map(job_run_row);
        let widths = [
            Constraint::Length(10),
            Constraint::Fill(1),
            Constraint::Length(14),
        ];
        let jobs_block = Block::default()
            .title(Line::from(Span::styled(
                format!(" Job Runs  ({}) ", self.summary.job_runs.len()),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray))
            .padding(Padding::horizontal(1));
        let table = Table::new(rows, widths)
            .header(header)
            .column_spacing(2)
            .block(jobs_block);

        table.render(jobs_area, buffer);
    }
}

fn job_run_row(job_run: &JobRunSummary) -> Row<'static> {
    let job_run_id = last_chars(&job_run.job_run_id, 5);
    let job = format!("{} ({job_run_id})", job_run.job_id);
    let duration = job_run_duration(job_run);
    let status = Cell::from(job_run.status.clone()).style(status_style(&job_run.status));

    Row::new([status, Cell::from(job), Cell::from(duration)])
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
    use super::last_chars;

    #[test]
    fn shortens_job_run_ids_from_the_end() {
        assert_eq!(last_chars("job-run-abcde", 5), "abcde");
        assert_eq!(last_chars("abc", 5), "abc");
        assert_eq!(last_chars("abc", 0), "");
    }
}
