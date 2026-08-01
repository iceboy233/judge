use std::{
    ffi::OsStr,
    io::{self, PipeReader, PipeWriter},
    path::Path,
};

use crate::wait::WaitResult;

pub trait Workspace {
    fn write_file(&self, path: impl AsRef<Path>, contents: &[u8]) -> io::Result<()>;

    fn run(
        &self,
        program: impl AsRef<OsStr>,
        args: impl IntoIterator<Item = impl AsRef<OsStr>>,
        stdin: Option<PipeReader>,
        stdout: Option<PipeWriter>,
        stderr: Option<PipeWriter>,
    ) -> io::Result<WaitResult>;
}
