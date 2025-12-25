use std::{
    io::{IoSlice, Write},
    process::{Command, Stdio},
};

use crate::{
    descriptions::Description,
    man::{ManpageBuffer, ManpageType},
};

const DESC_START_SYMBOL: &str = "__#DESCRIPTION_START#__";

#[derive(PartialEq)]
enum ReadState {
    FindDescription,
    FindEnd,
}

type SectionBoundarires = (usize, usize);

fn find_description_section(mut buf: &mut ManpageBuffer) -> Option<SectionBoundarires> {
    let mut manpage_type = ManpageType::Unknown;
    let mut read_state = ReadState::FindDescription;

    let mut start: Option<usize> = None;
    let mut end: Option<usize> = None;

    loop {
        let line = match buf.next() {
            Some(v) => v,
            None => break,
        };

        let index = match line.char_indices().nth(3) {
            Some((i, _)) => i,
            _ => continue,
        };

        let (macro_seq, rest) = line.split_at(index);

        let macro_manpage_type = match macro_seq {
            ".SH" => ManpageType::Man,
            ".Sh" => ManpageType::Mdoc,
            _ => ManpageType::Unknown,
        };

        if macro_manpage_type == ManpageType::Unknown {
            continue;
        } else if manpage_type == ManpageType::Unknown {
            manpage_type = macro_manpage_type;
        } else if manpage_type != macro_manpage_type {
            return None;
        }

        let cursor_pos = buf.get_cursor_ref().position() as usize;

        match read_state {
            ReadState::FindDescription => {
                if rest.trim() != "DESCRIPTION" {
                    continue;
                }

                start = Some(cursor_pos);
                read_state = ReadState::FindEnd;
            }
            ReadState::FindEnd => {
                end = Some(cursor_pos - line.len());
                break;
            }
        };
    }

    return match (start, end) {
        (Some(s), Some(e)) => Some((s, e)),
        _ => None,
    };
}

pub struct DescriptionSection {
    inner: String,
    start: usize,
    end: usize,
}

impl DescriptionSection {
    pub fn get_description(self) -> Option<Description> {
        let mut groff_command = Command::new("groff");

        let groff = groff_command
            .arg("-Tascii")
            .stderr(Stdio::null())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());

        let mut groff = match groff.spawn() {
            Ok(c) => c,
            _ => return None,
        };

        let slices: [&str; 5] = [
            ".nh\n",
            &self.inner[..self.start],
            "\\&",
            DESC_START_SYMBOL,
            &self.inner[self.start..self.end],
        ];

        let io_slices = slices.map(|s| IoSlice::new(s.as_bytes()));

        let stdin = groff.stdin.as_mut().unwrap();

        if let Err(_) = stdin.write_vectored(&io_slices) {
            return None;
        }

        let output = match groff.wait_with_output() {
            Ok(v) => v,
            _ => return None,
        };

        if !output.status.success() {
            return None;
        }

        if let Ok(out) = String::from_utf8(output.stdout) {
            let symbol_index = match out.find(DESC_START_SYMBOL) {
                Some(i) => i,
                None => return None,
            };

            let start_index = symbol_index + DESC_START_SYMBOL.len();
            let value = (&out[start_index..]).trim().to_owned();

            return Some(Description::new(value));
        }

        None
    }
}

impl From<(String, SectionBoundarires)> for DescriptionSection {
    fn from((inner, boundarires): (String, SectionBoundarires)) -> Self {
        let (start, end) = boundarires;
        Self { inner, start, end }
    }
}

pub fn extract_description_section(mut buf: ManpageBuffer) -> Option<DescriptionSection> {
    let section = match find_description_section(&mut buf) {
        Some(v) => v,
        None => return None,
    };

    let inner = match buf.into_inner() {
        Ok(v) => v,
        _ => return None,
    };

    Some((inner, section).into())
}
