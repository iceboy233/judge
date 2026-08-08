use std::{
    ffi::OsStr,
    fs::File,
    io::{self, PipeReader, PipeWriter, Read},
    process::{Command, Stdio},
};

use tempfile::TempDir;

use crate::{
    traits::Workspace,
    wait::{WaitExt, WaitResult},
};

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
    fn write_file(
        &self,
        path: impl AsRef<std::path::Path>,
        mut reader: impl Read,
    ) -> io::Result<()> {
        let path = self.base_dir.path().join(path);
        // TODO: create dir for nested files
        let mut file = File::create(&path)?;
        io::copy(&mut reader, &mut file).map(|_| ())
    }

    fn run(
        &self,
        program: impl AsRef<OsStr>,
        args: impl IntoIterator<Item = impl AsRef<OsStr>>,
        stdin: Option<PipeReader>,
        stdout: Option<PipeWriter>,
        stderr: Option<PipeWriter>,
    ) -> io::Result<WaitResult> {
        let mut cmd = Command::new(program);
        cmd.args(args);
        cmd.current_dir(self.base_dir.path());
        cmd.stdin(stdin.map_or_else(Stdio::null, Stdio::from));
        cmd.stdout(stdout.map_or_else(Stdio::null, Stdio::from));
        cmd.stderr(stderr.map_or_else(Stdio::null, Stdio::from));
        let mut child = cmd.spawn()?;
        let result = child.wait_with_usage()?;
        Ok(result)
    }
}
