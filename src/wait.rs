use std::{
    io,
    process::{Child, ExitStatus},
};

pub struct WaitResult {
    pub status: ExitStatus,
    pub cpu_time_us: u64,
    pub peak_memory_bytes: u64,
}

pub trait WaitExt {
    fn wait_with_usage(&mut self) -> io::Result<WaitResult>;
}

#[cfg(unix)]
impl WaitExt for Child {
    fn wait_with_usage(&mut self) -> io::Result<WaitResult> {
        let pid = self.id() as libc::pid_t;
        let mut status: libc::c_int = 0;
        let mut rusage: libc::rusage = unsafe { std::mem::zeroed() };

        let res = unsafe { libc::wait4(pid, &mut status, 0, &mut rusage) };
        if res < 0 {
            return Err(io::Error::last_os_error());
        }

        use std::os::unix::process::ExitStatusExt;
        let status = ExitStatus::from_raw(status);

        let cpu_time_us = rusage.ru_utime.tv_sec as u64 * 1_000_000
            + rusage.ru_utime.tv_usec as u64
            + rusage.ru_stime.tv_sec as u64 * 1_000_000
            + rusage.ru_stime.tv_usec as u64;

        let peak_memory_bytes = if cfg!(target_os = "linux") {
            rusage.ru_maxrss as u64 * 1024
        } else {
            rusage.ru_maxrss as u64
        };

        Ok(WaitResult {
            status,
            cpu_time_us,
            peak_memory_bytes,
        })
    }
}
