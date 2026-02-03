use clap::Command;
use clap_complete::{Shell, generate};
use std::io::{self, Write};

/// Command to generate shell completions.
pub struct CompletionCommand;

impl CompletionCommand {
    /// Generates completion script for the specified shell.
    pub fn execute(cmd: &mut Command, shell: Shell) {
        let stdout = io::stdout();
        let mut writer = SafeWriter {
            inner: stdout.lock(),
        };
        generate(shell, cmd, "awspm", &mut writer);
    }
}

struct SafeWriter<W: Write> {
    inner: W,
}

impl<W: Write> Write for SafeWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.inner.write(buf) {
            Ok(n) => Ok(n),
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {
                // Silently ignore broken pipe
                Ok(buf.len())
            }
            Err(e) => Err(e),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.inner.flush() {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
            Err(e) => Err(e),
        }
    }
}
