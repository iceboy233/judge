use std::{
    ffi::OsStr,
    fs,
    io::{self, PipeReader, PipeWriter},
    process::{Command, ExitStatus, Stdio},
};

use tempfile::TempDir;

use crate::traits::Workspace;

pub struct LocalWorkspace {
    base_dir: TempDir,
}

impl LocalWorkspace {
    pub fn new() -> io::Result<Self> {
        let base_dir = tempfile::Builder::new().prefix("judge-local-").tempdir()?;
        Ok(Self { base_dir })
    }
}

impl Workspace for LocalWorkspace {
    fn write_file(&self, path: impl AsRef<std::path::Path>, contents: &[u8]) -> io::Result<()> {
        let path = self.base_dir.path().join(path);
        // TODO: create dir for nested files
        fs::write(&path, contents)
    }

    fn run(
        &self,
        program: impl AsRef<OsStr>,
        args: impl IntoIterator<Item = impl AsRef<OsStr>>,
        stdin: Option<PipeReader>,
        stdout: Option<PipeWriter>,
        stderr: Option<PipeWriter>,
    ) -> io::Result<ExitStatus> {
        let mut cmd = Command::new(program);
        cmd.args(args);
        cmd.current_dir(self.base_dir.path());
        cmd.stdin(stdin.map_or_else(Stdio::null, Stdio::from));
        cmd.stdout(stdout.map_or_else(Stdio::null, Stdio::from));
        cmd.stderr(stderr.map_or_else(Stdio::null, Stdio::from));
        let mut child = cmd.spawn()?;
        let status = child.wait()?;
        Ok(status)
    }
}
