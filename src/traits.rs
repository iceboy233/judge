use std::{
    ffi::OsStr,
    io::{self, PipeReader, PipeWriter, Read},
    path::Path,
    process::Stdio,
};

use crate::wait::WaitResult;

pub enum Stream<P> {
    Discard,
    Inherit,
    Pipe(P),
}

pub trait Workspace {
    fn write_file(&self, path: impl AsRef<Path>, reader: impl Read) -> io::Result<()>;

    fn run(
        &self,
        program: impl AsRef<OsStr>,
        args: impl IntoIterator<Item = impl AsRef<OsStr>>,
        stdin: Stream<PipeReader>,
        stdout: Stream<PipeWriter>,
        stderr: Stream<PipeWriter>,
    ) -> io::Result<WaitResult>;
}

impl<P> From<Stream<P>> for Stdio
where
    P: Into<Stdio>,
{
    fn from(stream: Stream<P>) -> Self {
        match stream {
            Stream::Discard => Stdio::null(),
            Stream::Inherit => Stdio::inherit(),
            Stream::Pipe(pipe) => pipe.into(),
        }
    }
}
