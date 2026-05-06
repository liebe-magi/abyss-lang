//! I/O bridge abstraction shared by `RuntimeEnv` and the stdlib `unveil` /
//! `summon` builtins.
//!
//! The interpreter delegates writes (`unveil`) and prompts (`summon`)
//! through this trait instead of touching `std::io::stdout` / `stdin`
//! directly, so non-CLI hosts (the v0.6.0 Wasm playground, future LSP,
//! embedded REPL) can capture both streams without intercepting the
//! actual file descriptors. `RuntimeEnv` carries one
//! `Box<dyn IoBridge>`; CLI builds default to [`StdIoBridge`], and the
//! Wasm crate swaps in its own implementation that buffers writes into a
//! shared `String` and refuses reads.

use std::io;

/// Behaviour the interpreter needs from its embedding environment to
/// implement `unveil` (write) and `summon` (read-line). Output is
/// buffered by the consumer; the interpreter does not assume the bytes
/// are actually flushed to a TTY.
pub trait IoBridge {
    fn write_str(&mut self, content: &str) -> io::Result<()>;
    fn flush(&mut self) -> io::Result<()>;
    fn read_line(&mut self, buffer: &mut String) -> io::Result<()>;
}

/// Default bridge for CLI builds: writes go to `std::io::stdout()` and
/// reads come from `std::io::stdin()`. `RuntimeEnv::new()` installs this
/// automatically, so the existing CLI / REPL paths get the same
/// behaviour they had before the abstraction landed.
pub struct StdIoBridge;

impl IoBridge for StdIoBridge {
    fn write_str(&mut self, content: &str) -> io::Result<()> {
        use std::io::Write;
        io::stdout().write_all(content.as_bytes())
    }

    fn flush(&mut self) -> io::Result<()> {
        use std::io::Write;
        io::stdout().flush()
    }

    fn read_line(&mut self, buffer: &mut String) -> io::Result<()> {
        io::stdin().read_line(buffer).map(|_| ())
    }
}
