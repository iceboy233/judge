use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, Read, Seek},
    path::Path,
    sync::Arc,
};

use positioned_io::{RandomAccessFile, SizeCursor};
use zip::{read::ZipFile, ZipArchive};

type FileCursor = SizeCursor<Arc<RandomAccessFile>>;

pub struct Package {
    archive: ZipArchive<FileCursor>,
}

pub struct Case {
    input_file_number: usize,
    output_file_number: usize,
    time_limit_us: u64,
    memory_limit_bytes: u64,
    score: u64,
}

pub struct PackageStream {
    file: ZipFile<'static, FileCursor>,
    _archive: Box<ZipArchive<FileCursor>>,
}

impl Package {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let archive = ZipArchive::new(SizeCursor::new(Arc::new(RandomAccessFile::open(path)?)))?;
        Ok(Self { archive })
    }

    pub fn load_cases(&self) -> io::Result<Box<[Case]>> {
        let mut archive = self.archive.clone();
        let mut canonical_names = HashMap::new();
        for file_number in 0..self.archive.len() {
            let file = archive.by_index_raw(file_number)?;
            canonical_names.insert(file.name().to_ascii_lowercase(), file_number);
        }
        parse_legacy_config(&mut archive, &canonical_names)
    }

    pub fn open_input(&self, case: &Case) -> io::Result<PackageStream> {
        self.open_file(case.input_file_number)
    }

    pub fn open_output(&self, case: &Case) -> io::Result<PackageStream> {
        self.open_file(case.output_file_number)
    }

    fn open_file(&self, file_number: usize) -> io::Result<PackageStream> {
        let mut archive = Box::new(self.archive.clone());
        let archive_ref: &'static mut ZipArchive<FileCursor> =
            unsafe { std::mem::transmute(archive.as_mut()) };
        let file = archive_ref
            .by_index(file_number)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(PackageStream {
            file,
            _archive: archive,
        })
    }
}

impl Case {
    pub fn time_limit_us(&self) -> u64 {
        self.time_limit_us
    }

    pub fn memory_limit_bytes(&self) -> u64 {
        self.memory_limit_bytes
    }

    pub fn score(&self) -> u64 {
        self.score
    }
}

impl Read for PackageStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

fn parse_legacy_config<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    canonical_names: &HashMap<String, usize>,
) -> io::Result<Box<[Case]>> {
    let file_number = get_file_number(&canonical_names, "config.ini")?;
    let mut lines = BufReader::new(archive.by_index(file_number)?).lines();
    let count_line = lines
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "no count line"))??;
    let count: usize = count_line
        .trim()
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid count line"))?;
    let mut cases = Vec::with_capacity(count);
    for _ in 0..count {
        let config_line = lines
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "no config line"))??;
        let parts: Vec<&str> = config_line.split('|').take(5).collect();
        if parts.len() < 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid config line",
            ));
        }
        let input_file_number = get_file_number(&canonical_names, &format!("input/{}", parts[0]))?;
        let output_file_number =
            get_file_number(&canonical_names, &format!("output/{}", parts[1]))?;
        let time_limit_us = parts[2]
            .parse::<f64>()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid time limit"))
            .map(|secs| (secs * 1_000_000.0) as u64)?;
        let score: u64 = parts[3]
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid score"))?;
        let memory_limit_bytes = if parts.len() >= 5 {
            parts[4]
                .parse::<u64>()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid memory limit"))
                .map(|kb| kb * 1024)?
        } else {
            268_435_456
        };
        cases.push(Case {
            input_file_number,
            output_file_number,
            time_limit_us,
            memory_limit_bytes,
            score,
        });
    }
    Ok(cases.into_boxed_slice())
}

fn get_file_number(canonical_names: &HashMap<String, usize>, name: &str) -> io::Result<usize> {
    canonical_names
        .get(&name.to_ascii_lowercase())
        .copied()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("file not found: {name}"),
            )
        })
}
