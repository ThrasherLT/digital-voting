//! Module for user interaction with the node via CLI.
//! This API is in an early stage and will probably change as the implementation grows.

// TODO daemonize at least on Unix systems.

use std::borrow::Cow::{self, Borrowed, Owned};

use clap::Parser;
use crypto::signature::blind_sign;
use rustyline::{
    completion::FilenameCompleter,
    error::ReadlineError,
    highlight::{Highlighter, MatchingBracketHighlighter},
    hint::HistoryHinter,
    history::DefaultHistory,
    validate::MatchingBracketValidator,
    Completer, CompletionType, Config, EditMode, Editor, Helper, Hinter, Validator,
};
use thiserror::Error;

// Error type of the CLI module.
#[derive(Error, Debug)]
pub enum Error {
    /// There was an error reading a line from stdio.
    #[error("Read line error {}", .0)]
    ReadLine(#[from] ReadlineError),
}
type Result<T> = std::result::Result<T, Error>;

/// Command line arguments for the node.
/// All the stuff required to start the node.
#[derive(Parser, Clone, Debug)]
pub struct Args {
    /// The address the node will listen on.
    #[clap(short = 'a', long = "address", default_value = "127.0.0.1:8080")]
    pub socket_addr: std::net::SocketAddr,
    /// The public key of the election authority used to verify that the voters are eligible.
    #[clap(short = 'p', long = "authority-public-key")]
    pub authority_pk: blind_sign::Publickey,
    /// The command to execute. See `Cmd` for more details.
    #[clap(subcommand)]
    pub cmd: Cmd,
}

/// The command that the node should execute on startup.
#[derive(Parser, Clone, Debug)]
pub enum Cmd {
    /// Start a new blockchain and create a genesis block.
    #[clap(about = "Start new blokcchain")]
    Genesis {},
    /// Connect to an existing node of an existing blockchain.
    #[clap(about = "Connect to existing node")]
    Connect {
        /// The address of the existing node.
        peer_socket_addr: std::net::SocketAddr,
    },
}

/// `StdioReader` reads lines from stdio.
/// It also manages the command history so should only be dropped
/// when the application exits.
pub struct StdioReader {
    /// The rustyline editor.
    rl: Editor<MyHelper, DefaultHistory>,
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
    pub fn new() -> Result<Self> {
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
        let mut rl = Editor::with_config(config)?;
        rl.set_helper(Some(h));
        let _ = rl.load_history("node-cmd-history.txt");

        Ok(Self { rl })
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
    pub fn read_stdio_blocking(&mut self) -> Result<String> {
        let prompt = "Node$ ".to_string();
        self.rl.helper_mut().expect("No helper").colored_prompt =
            format!("\x1b[1;32m{prompt}\x1b[0m");
        let line = self.rl.readline(&prompt)?;

        Ok(line)
    }
}

impl Drop for StdioReader {
    /// The command history is saved to a file when the `StdioReader` is dropped.
    /// So `StdioReader` should only really be dropped when the program is exiting.
    fn drop(&mut self) {
        let _ = self.rl.save_history("node-cmd-history.txt");
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
