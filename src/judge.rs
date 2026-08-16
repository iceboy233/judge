use std::io::{self, pipe};

use crate::{
    compare::{Comparator, CompareResult},
    package::{Case, Package},
    traits::{Stream, Workspace},
    wait::WaitResult,
};

pub struct JudgeResult {
    pub verdict: Verdict,
    pub time_usage_us: u64,
    pub memory_usage_bytes: u64,
}

pub enum Verdict {
    Accepted,
    WrongAnswer,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    RuntimeError,
    SystemError,
}

pub fn judge<W, C>(
    package: &Package,
    case: &Case,
    workspace: &W,
    program: &str,
    comparator: C,
) -> io::Result<JudgeResult>
where
    W: Workspace,
    C: Comparator + Send + 'static,
{
    let (stdin_reader, mut stdin_writer) = pipe()?;
    let mut input = package.open_input(&case)?;
    let stdin_thread = std::thread::spawn(move || -> io::Result<()> {
        io::copy(&mut input, &mut stdin_writer)?;
        Ok(())
    });

    let (mut stdout_reader, stdout_writer) = pipe()?;
    let mut output = package.open_output(&case)?;
    let stdout_thread = std::thread::spawn(move || -> io::Result<CompareResult> {
        comparator.compare(&mut output, &mut stdout_reader)
    });

    let wait_result = workspace.run(
        program,
        [""; 0],
        Stream::Pipe(stdin_reader),
        Stream::Pipe(stdout_writer),
        Stream::Discard,
    )?;
    let compare_result = stdout_thread.join().unwrap().unwrap();
    stdin_thread.join().unwrap().unwrap();

    let verdict = Verdict::evaluate(
        &wait_result,
        &compare_result,
        case.time_limit_us(),
        case.memory_limit_bytes(),
    );
    let time_usage_us = wait_result.cpu_time_us;
    let memory_usage_bytes = wait_result.peak_memory_bytes;

    Ok(JudgeResult {
        verdict,
        time_usage_us,
        memory_usage_bytes,
    })
}

impl Verdict {
    pub fn evaluate(
        wait_result: &WaitResult,
        compare_result: &CompareResult,
        time_limit_us: u64,
        memory_limit_bytes: u64,
    ) -> Self {
        if wait_result.peak_memory_bytes >= memory_limit_bytes {
            return Verdict::MemoryLimitExceeded;
        }
        if wait_result.cpu_time_us >= time_limit_us {
            return Verdict::TimeLimitExceeded;
        }
        if !wait_result.status.success() {
            return Verdict::RuntimeError;
        }
        if !compare_result.ok {
            return Verdict::WrongAnswer;
        }
        Verdict::Accepted
    }
}
