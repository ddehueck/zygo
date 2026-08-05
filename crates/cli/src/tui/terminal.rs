use anyhow::{Context, Result};
use ratatui::{DefaultTerminal, TerminalOptions, Viewport};

/// Creates an inline TUI with the requested maximum height.
pub fn inline_terminal<T>(
    height: u16,
    cb: impl FnOnce(&mut DefaultTerminal) -> Result<T>,
) -> Result<T> {
    let options = TerminalOptions {
        viewport: Viewport::Inline(height),
    };

    let mut terminal =
        ratatui::try_init_with_options(options).context("failed to initialize terminal")?;

    let result = cb(&mut terminal);

    // Restore the terminal state before returning.
    // Without it the terminal may be left in an undefined state and behave weird.
    let restore_result = ratatui::try_restore().context("failed to restore terminal");

    match result {
        Ok(value) => {
            restore_result?;
            Ok(value)
        }
        Err(err) => {
            let _ = restore_result;
            Err(app_error)
        }
    }
}
