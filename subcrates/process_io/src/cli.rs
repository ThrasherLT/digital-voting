//! Module for handling STDIO CLI.

use rustyline::{
    completion::FilenameCompleter,
    error::ReadlineError,
    highlight::{Highlighter, MatchingBracketHighlighter},
    hint::HistoryHinter,
    history::DefaultHistory,
    validate::MatchingBracketValidator,
    Completer, CompletionType, Config, EditMode, Editor, Helper, Hinter, Validator,
};
use std::{
    borrow::Cow::{self, Borrowed, Owned},
    path::PathBuf,
};
use thiserror::Error;

// Error type of the CLI module.
#[derive(Error, Debug)]
pub enum Error {
    /// There was an error reading a line from stdio.
    #[error("Read line error {}", .0)]
    ReadLine(#[from] ReadlineError),
    /// There was an error reading while reading the name of the current executable.
    #[error("Error while trying to read current exec name {}", .0)]
    CurrentExec(#[from] std::io::Error),
    /// There was an error reading while reading the name of the current executable.
    #[error(transparent)]
    MismatchedQuotes(#[from] shellwords::MismatchedQuotes),
    /// The provided path was invalid.
    #[error("Invalid path")]
    Path,
}
type Result<T> = std::result::Result<T, Error>;

/// `StdioReader` reads lines from stdio.
/// It also manages the command history so should only be dropped
/// when the application exits.
pub struct StdioReader {
    /// The rustyline editor.
    rl: Editor<MyHelper, DefaultHistory>,
    /// The name of the executable.
    /// Used for adding to the input command, so that `clap` can parse it.
    /// Storing this here to avoid the extra operations needed to retrieve it.
    exec_path: String,
    /// Path to where the command history file will be stored.
    cmd_history_path: PathBuf,
}

impl StdioReader {
    /// Create a new `StdioReader`.
    ///
    /// # Returns
    ///
    /// A new `StdioReader`.
    ///
    /// # Errors
    ///
    /// If there was an error creating the Editor for Rustyline.
    pub fn new(cmd_history_path: PathBuf) -> Result<Self> {
        let config = Config::builder()
            .completion_type(CompletionType::List)
            .auto_add_history(true)
            .edit_mode(EditMode::Emacs)
            .build();
        let h = MyHelper {
            completer: FilenameCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter::new(),
            colored_prompt: String::new(),
            validator: MatchingBracketValidator::new(),
        };
        let exec_path = std::env::current_exe()?;
        let exec_path = exec_path.to_string_lossy().to_string();
        let mut rl = Editor::with_config(config)?;
        rl.set_helper(Some(h));
        if let Some(parent) = cmd_history_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let _ = rl.load_history(&cmd_history_path);

        Ok(Self {
            rl,
            exec_path,
            cmd_history_path,
        })
    }

    /// Read a line from stdio. This function blocks until a line is read.
    ///
    /// # Returns
    ///
    /// The line read from stdio.
    ///
    /// # Panics
    ///
    /// If the helper struct isn't set, but that shouldn't happen.
    ///
    /// # Errors
    ///
    /// If there was an error reading from stdio.
    pub fn read_stdio_blocking(&mut self) -> Result<Vec<String>> {
        let prompt = "Node$ ".to_string();
        self.rl.helper_mut().expect("No helper").colored_prompt =
            format!("\x1b[1;32m{prompt}\x1b[0m");
        let line = self.rl.readline(&prompt)?;

        let mut line = shellwords::split(&line)?;
        line.insert(0, self.exec_path.clone());

        Ok(line)
    }
}

impl Drop for StdioReader {
    /// The command history is saved to a file when the `StdioReader` is dropped.
    /// So `StdioReader` should only really be dropped when the program is exiting.
    fn drop(&mut self) {
        let _ = self.rl.save_history(&self.cmd_history_path);
    }
}

/// Helper struct for the rustyline library.
#[derive(Helper, Completer, Hinter, Validator)]
struct MyHelper {
    #[rustyline(Completer)]
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    #[rustyline(Validator)]
    validator: MatchingBracketValidator,
    #[rustyline(Hinter)]
    hinter: HistoryHinter,
    colored_prompt: String,
}

impl Highlighter for MyHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize, forced: bool) -> bool {
        self.highlighter.highlight_char(line, pos, forced)
    }
}
