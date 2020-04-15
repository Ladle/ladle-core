use std::collections::BTreeMap;
use std::fmt;

use super::Input;

/// Describes a specific markup of an Input to format
pub struct AnnotationBuilder<'a> {
    input: &'a Input,
    lines: BTreeMap<usize, Option<Underline>>,
    message: Option<String>
}

/// Represents a range of a line to underline
pub struct Underline {
    pub start: usize,
    pub len: usize
}

impl<'a> AnnotationBuilder<'a> {
    /// Create an AnnotationBuilder which annotates the given input
    pub fn new(input: &'a Input) -> Self {
        AnnotationBuilder {
            input,
            lines: BTreeMap::new(),
            message: None
        }
    }

    /// Show this line in the annotation
    pub fn add_line(&mut self, line: usize) {
        if !self.lines.contains_key(&line) {
            self.lines.insert(line, None);
        }
    }

    /// Show this line in the annotation with the given underline
    pub fn add_line_underlined(&mut self, line: usize, underline: Underline) {
        self.lines.insert(line, Some(underline));
    }

    /// Set the message of the annotation
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
            // The continuation indicator
            if let Some(last) = last_num {
                if last + 1 < line_num {
                    write!(f, "{m} | ...\n", m = margin)?;
                }
            }
            // The line itself
            write!(f, "{lnum:w$} | {line}\n",
                lnum = line_num,
                w = margin_width,
                line = self.input.get_line_slice(line_num))?;
            // The underline
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
            " ".repeat(offset) + &("^".repeat(len))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_annotation() {
        let input = Input::new("1\n12\n123\n1234\n12345\n123456".into());

        let annotation = AnnotationBuilder::new(&input);

        assert_eq!("AnnotationBuilder: No Contents to Display", format!("{}", annotation));
    }
    
    #[test]
    fn test_one_line_annotation() {
        let input = Input::new("1\n12\n123\n1234\n12345\n123456".into());

        let mut annotation = AnnotationBuilder::new(&input);
        annotation.add_line(2);

        let expected = [
            "   |\n",
            " 2 | 123\n",
            "   |\n"
        ].concat();

        assert_eq!(expected, format!("{}", annotation));
    }
    
    #[test]
    fn test_two_line_annotation() {
        let input = Input::new("1\n12\n123\n1234\n12345\n123456".into());

        let mut annotation = AnnotationBuilder::new(&input);
        annotation.add_line(2);
        annotation.add_line(3);

        let expected = [
            "   |\n",
            " 2 | 123\n",
            " 3 | 1234\n",
            "   |\n"
        ].concat();

        assert_eq!(expected, format!("{}", annotation));
    }
    
    #[test]
    fn test_two_line_gap_annotation() {
        let input = Input::new("1\n12\n123\n1234\n12345\n123456".into());

        let mut annotation = AnnotationBuilder::new(&input);
        annotation.add_line(2);
        annotation.add_line(4);

        let expected = [
            "   |\n",
            " 2 | 123\n",
            "   | ...\n",
            " 4 | 12345\n",
            "   |\n"
        ].concat();

        assert_eq!(expected, format!("{}", annotation));
    }
    
    #[test]
    fn test_one_line_underlined_annotation() {
        let input = Input::new("1\n12\n123\n1234\n12345\n123456".into());

        let mut annotation = AnnotationBuilder::new(&input);
        annotation.add_line_underlined(2, Underline { start: 0, len: 3 });

        let expected = [
            "   |\n",
            " 2 | 123\n",
            "   | ^^^\n",
            "   |\n"
        ].concat();

        assert_eq!(expected, format!("{}", annotation));
    }
    
    #[test]
    fn test_two_line_underlined_annotation() {
        let input = Input::new("1\n12\n123\n1234\n12345\n123456".into());

        let mut annotation = AnnotationBuilder::new(&input);
        annotation.add_line_underlined(2, Underline { start: 0, len: 3 });
        annotation.add_line_underlined(3, Underline { start: 0, len: 4 });

        let expected = [
            "   |\n",
            " 2 | 123\n",
            "   | ^^^\n",
            " 3 | 1234\n",
            "   | ^^^^\n",
            "   |\n"
        ].concat();

        assert_eq!(expected, format!("{}", annotation));
    }

    #[test]
    fn test_two_line_gap_underlined_annotation() {
        let input = Input::new("1\n12\n123\n1234\n12345\n123456".into());

        let mut annotation = AnnotationBuilder::new(&input);
        annotation.add_line_underlined(2, Underline { start: 0, len: 3 });
        annotation.add_line_underlined(4, Underline { start: 0, len: 5 });

        let expected = [
            "   |\n",
            " 2 | 123\n",
            "   | ^^^\n",
            "   | ...\n",
            " 4 | 12345\n",
            "   | ^^^^^\n",
            "   |\n"
        ].concat();

        assert_eq!(expected, format!("{}", annotation));
    }
}
