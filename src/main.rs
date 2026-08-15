use std::{fmt, fs::File, io, path::PathBuf};

use anstyle::{AnsiColor, Color, Style};
use bpaf::Bpaf;
use judge::{
    compare::TokenComparator,
    config::LanguageMap,
    judge::{judge, Verdict},
    local::LocalWorkspace,
    package::Package,
    traits::Workspace,
};

#[derive(Clone, Debug, Bpaf)]
#[bpaf(options, version)]
struct Options {
    /// Source file path
    #[bpaf(short, long)]
    source: PathBuf,

    /// Package file path
    #[bpaf(short, long)]
    package: PathBuf,
}

struct CaseFormatter {
    pub number: usize,
    pub verdict: Verdict,
    pub time_usage_us: u64,
    pub memory_usage_bytes: u64,
}

fn main() -> io::Result<()> {
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
    workspace.run(&lang.compile, &lang.compile_args, None, None, None)?;
    let package = Package::open(&options.package)?;

    println!("{:8}{:24}{:8}{:8}", "Case", "Verdict", "Time", "Memory");
    for (index, case) in package.load_cases()?.iter().enumerate() {
        let result = judge(&package, case, &workspace, &lang.run, TokenComparator)?;
        println!(
            "{}",
            CaseFormatter {
                number: index + 1,
                verdict: result.verdict,
                time_usage_us: result.time_usage_us,
                memory_usage_bytes: result.memory_usage_bytes,
            }
        );
    }
    Ok(())
}

impl fmt::Display for CaseFormatter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const STYLE_OK: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)));
        const STYLE_NA: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));

        let number = self.number;
        let (label, style) = match self.verdict {
            Verdict::Accepted => ("Accepted", STYLE_OK),
            Verdict::WrongAnswer => ("Wrong Answer", STYLE_NA),
            Verdict::TimeLimitExceeded => ("Time Limit Exceeded", STYLE_NA),
            Verdict::MemoryLimitExceeded => ("Memory Limit Exceeded", STYLE_NA),
            Verdict::RuntimeError => ("Runtime Error", STYLE_NA),
            Verdict::SystemError => ("System Error", STYLE_NA),
        };
        let time_usage = format!("{} ms", self.time_usage_us / 1_000);
        let memory_usage = format!("{} KB", self.memory_usage_bytes / 1024);
        write!(
            f,
            "{number:<8}{style}{label:24}{style:#}{time_usage:8}{memory_usage:8}"
        )
    }
}
