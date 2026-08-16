use std::{fs::File, io, path::PathBuf, process::ExitCode};

use bpaf::Bpaf;
use judge::{
    config::LanguageMap,
    local::LocalWorkspace,
    traits::{Stream, Workspace},
};

#[derive(Clone, Debug, Bpaf)]
#[bpaf(options, version)]
struct Options {
    /// Source file path
    #[bpaf(positional("SOURCE"))]
    source: PathBuf,
}

fn main() -> io::Result<ExitCode> {
    let options = options().run();
    let lang_map = LanguageMap::load()?;

    let source_ext = options
        .source
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Missing source extension"))?;
    let lang = lang_map.get_by_source_ext(source_ext).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Unsupported source extension: {source_ext}"),
        )
    })?;

    let source_file = File::open(&options.source)?;
    let workspace = LocalWorkspace::new()?;
    workspace.write_file(&lang.source, source_file)?;
    workspace.run(
        &lang.compile,
        &lang.compile_args,
        Stream::Discard,
        Stream::Discard,
        Stream::Inherit,
    )?;
    let wait_result = workspace.run(
        &lang.run,
        [""; 0],
        Stream::Inherit,
        Stream::Inherit,
        Stream::Inherit,
    )?;
    match wait_result.status.success() {
        true => Ok(ExitCode::SUCCESS),
        false => Ok(ExitCode::FAILURE),
    }
}
