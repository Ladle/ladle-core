use std::collections::BTreeMap;
use std::fmt;

use super::Input;

pub struct AnnotationBuilder<'a> {
    input: &'a Input,
    lines: BTreeMap<usize, Option<Underline>>,
    message: Option<String>
}

pub struct Underline {
    pub start: usize,
    pub len: usize
}

impl<'a> AnnotationBuilder<'a> {
    pub fn new(input: &'a Input) -> Self {
        AnnotationBuilder {
            input,
            lines: BTreeMap::new(),
            message: None
        }
    }

    pub fn add_line(&mut self, line: usize) {
        if !self.lines.contains_key(&line) {
            self.lines.insert(line, None);
        }
    }

    pub fn add_line_underlined(&mut self, line: usize, underline: Underline) {
        self.lines.insert(line, Some(underline));
    }

    pub fn set_message(&mut self, message: String) {
        self.message = Some(message);
    }
}

impl<'a> fmt::Display for AnnotationBuilder<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.lines.is_empty() {
            return write!(f, "AnnotationBuilder: No Contents to Display");
        }

        let min_line = self.lines.iter().next().unwrap().0;
        let max_line = self.lines.iter().next_back().unwrap().0;
        let margin_width = num_dec_digits(*max_line) + 1;
        let margin = " ".repeat(margin_width);

        if let Some(path_buf) = &self.input.path {
            write!(f, "{m}--> {p}:{ln}\n",
                p = path_buf.to_str().unwrap_or(""),
                ln = min_line,
                m = margin)?;
        };


        // Padding line
        write!(f, "{m} |\n", m = margin)?;

        let mut last_num = None;

        for (line_num, underline_opt) in self.lines.iter() {
            let line_num = *line_num;

            if let Some(last) = last_num {
                if last + 1 < line_num {
                    write!(f, "{m} | ...\n", m = margin)?;
                }
            }

            write!(f, "{lnum:w$} | {line}\n",
                lnum = line_num,
                w = margin_width,
                line = self.input.get_line_slice(line_num))?;

            if let Some(underline) = underline_opt {
                write!(f, "{m} | {u}\n",
                    m = margin,
                    u = make_underline(underline.start, underline.len))?;
            }

            last_num = Some(line_num);
        }
        
        // Padding line
        write!(f, "{m} |\n", m = margin)?;

        // Message line
        if let Some(message) = &self.message {
            write!(f, "{m} = {message}",
                    m = margin,
                    message = message)?;
        }

        Ok(())
    }
}

fn num_dec_digits(num: usize) -> usize {
    format!("{}", num).len()
}

fn make_underline(offset: usize, len: usize) -> String {
    match len {
        0 => {
            if offset > 0 {
                " ".repeat(offset - 1) + "><"
            } else {
                "<".into()
            }
        },
        1 => {
            " ".repeat(offset) + "^"
        },
        _ => {
            " ".repeat(offset) + "^" + &("-".repeat(len - 2)) + "^"
        }
    }
}
