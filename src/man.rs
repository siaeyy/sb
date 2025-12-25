use std::{
    fs::File,
    io::{self, BufRead, Cursor, Read},
    path::Path,
    process::{Command, Stdio},
    string::FromUtf8Error,
};

use flate2::read::GzDecoder;

#[derive(Debug, PartialEq)]
pub enum ManpageType {
    Man,
    Mdoc,
    Unknown,
}

pub struct ManpageBuffer {
    cursor: Cursor<Vec<u8>>,
}

impl ManpageBuffer {
    pub fn new(vec: Vec<u8>) -> Self {
        Self {
            cursor: Cursor::new(vec),
        }
    }

    pub fn get_cursor_ref<'a>(&'a self) -> &'a Cursor<Vec<u8>> {
        &self.cursor
    }

    pub fn into_inner(self) -> Result<String, FromUtf8Error> {
        let buf = self.cursor.into_inner();
        String::from_utf8(buf)
    }
}

impl From<File> for ManpageBuffer {
    fn from(mut file: File) -> Self {
        let mut content = Vec::new();
        let _ = file.read_to_end(&mut content);

        ManpageBuffer::new(content)
    }
}

impl TryFrom<&Path> for ManpageBuffer {
    type Error = io::Error;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let is_gz = match path.extension() {
            Some(extension) => extension == "gz",
            None => false,
        };

        let file = File::open(path)?;

        if !is_gz {
            return Ok(ManpageBuffer::from(file));
        }

        let mut content = Vec::new();
        let mut decoder = GzDecoder::new(file);
        let _ = decoder.read_to_end(&mut content);

        Ok(ManpageBuffer::new(content))
    }
}

impl Read for ManpageBuffer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.cursor.read(buf)
    }
}

impl BufRead for ManpageBuffer {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.cursor.fill_buf()
    }

    fn consume(&mut self, amount: usize) {
        self.cursor.consume(amount);
    }
}

impl Iterator for &mut ManpageBuffer {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = String::new();
        match self.cursor.read_line(&mut buf) {
            Ok(0) => None,
            Ok(_) => Some(buf),
            Err(_) => None,
        }
    }
}

const MANPATH_ERR_PREFIX: &str = "No manual entry for ";

pub struct Manpaths {
    pub paths: Vec<String>,
    pub not_founds: Vec<String>,
}

pub fn get_manpaths<'a>(names: Vec<String>) -> Option<Manpaths> {
    let mut man_command = Command::new("man");

    let man = man_command
        .arg("-w")
        .args(names)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped());

    let man = match man.spawn() {
        Ok(c) => c,
        Err(_) => return None,
    };

    let output = match man.wait_with_output() {
        Ok(v) => v,
        Err(_) => return None,
    };

    match output.status.code() {
        Some(0 | 16) => {}
        _ => return None,
    }

    let (stdout, stderr) = match (
        String::from_utf8(output.stdout),
        String::from_utf8(output.stderr),
    ) {
        (Ok(stdout), Ok(stderr)) => (stdout, stderr),
        _ => return None,
    };
    
    let paths = stdout
        .lines()
        .map(|line| line.to_owned())
        .collect::<Vec<String>>();

    let not_founds = stderr
        .lines()
        .filter_map(|v| {
            if v.starts_with(MANPATH_ERR_PREFIX) {
                Some(v[MANPATH_ERR_PREFIX.len()..].to_string())
            } else {
                None
            }
        })
        .collect::<Vec<String>>();

    let manpaths = Manpaths {
        paths,
        not_founds,
    };

    Some(manpaths)
}
