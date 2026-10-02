//! The PTY's reader and writer on Unix: non-blocking copies of the master that wait
//! with `poll` in short steps, so they stop once the session has ended. A blocking
//! write to a terminal that nobody reads would otherwise wait forever.

use std::io::{self, ErrorKind, Read, Write};
use std::os::fd::{AsRawFd, RawFd};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use filedescriptor::{FileDescriptor, POLLIN, POLLOUT, poll, pollfd};
use portable_pty::MasterPty;

use super::other;

/// How long one wait for the PTY lasts before the reader or writer checks whether
/// the session has ended.
const STEP: Duration = Duration::from_millis(100);

/// The master's descriptor, for `FileDescriptor::dup`, which copies it.
struct Master(RawFd);

impl AsRawFd for Master {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

/// One end of the master: the reader or the writer.
struct Side {
    fd: FileDescriptor,
    closed: Arc<AtomicBool>,
}

/// A reader and a writer on copies of `master`. Both stop once `closed` is set.
pub(super) fn reader_writer(
    master: &dyn MasterPty,
    closed: Arc<AtomicBool>,
) -> io::Result<(Box<dyn Read + Send>, Box<dyn Write + Send>)> {
    let raw = master
        .as_raw_fd()
        .ok_or_else(|| other("the PTY has no file descriptor"))?;
    let mut read_fd = FileDescriptor::dup(&Master(raw)).map_err(other)?;
    // The copies share one open file, so this makes the writer's copy non-blocking too.
    read_fd.set_non_blocking(true).map_err(other)?;
    let write_fd = read_fd.try_clone().map_err(other)?;
    let reader = Side {
        fd: read_fd,
        closed: closed.clone(),
    };
    let writer = Side {
        fd: write_fd,
        closed,
    };
    Ok((Box::new(reader), Box::new(writer)))
}

impl Side {
    fn closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    /// Waits up to [`STEP`] for the PTY to be ready for `events`.
    fn wait(&self, events: i16) {
        let mut fds = [pollfd {
            fd: self.fd.as_raw_fd(),
            events,
            revents: 0,
        }];
        // `select` on macOS refuses descriptors past 1024; then wait the step out.
        if poll(&mut fds, Some(STEP)).is_err() {
            thread::sleep(STEP / 10);
        }
    }
}

impl Read for Side {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            if self.closed() {
                return Ok(0);
            }
            match self.fd.read(buf) {
                Err(error) if error.kind() == ErrorKind::WouldBlock => self.wait(POLLIN),
                Err(error) if error.kind() == ErrorKind::Interrupted => {}
                result => return result,
            }
        }
    }
}

impl Write for Side {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        loop {
            if self.closed() {
                return Err(io::Error::new(
                    ErrorKind::BrokenPipe,
                    "the terminal session has ended",
                ));
            }
            match self.fd.write(buf) {
                Err(error) if error.kind() == ErrorKind::WouldBlock => self.wait(POLLOUT),
                Err(error) if error.kind() == ErrorKind::Interrupted => {}
                result => return result,
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
