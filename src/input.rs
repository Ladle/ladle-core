use std::fmt;
use std::path::PathBuf;

/// Input represents input to tokenizing and parsing operations
/// It contains text and associated metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    /// The path of the input if known
    path: Option<PathBuf>,

    /// The text being represented
    text: String,

    /// A Vec containing the index of every newline in the file
    /// For each index/value pair in the table,
    /// the index indicates which newline this is and the value indicates where in the input it occurs
    newline_table: Vec<usize>
}

/// Pos represents a position in the Input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    /// The row index of the position
    pub row: usize,
    /// The column index of the position
    pub col: usize
}

impl Input {
    pub fn new(text: String) -> Self {
        let newline_table = Input::find_newlines(&text);
        Input {
            path: None,
            text, newline_table
        }
    }

    pub fn new_with_path(text: String, path: PathBuf) -> Self {
        let newline_table = Input::find_newlines(&text);
        Input {
            path: Some(path),
            text, newline_table 
        }
    }

    fn find_newlines(text: &str) -> Vec<usize> {
        text.char_indices()
            .filter(|(_i, c)| *c == '\n')
            .map(|(i, _c)| i)
            .collect()
    }

    /// Borrows the text as a &str
    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn get_row_slice(&self, row: usize) -> &str {
        let min_index = self.row_start(row);
        let max_index = self.row_end(row);
        &self.text[min_index..max_index]
    }

    fn row_start(&self, row: usize) -> usize {
        if row == 0 {
            0
        } else {
            self.newline_table[row - 1] + 1
        }
    }

    fn row_end(&self, row: usize) -> usize {
        if row >= self.newline_table.len() {
            self.text.len()
        } else {
            self.newline_table[row]
        }
    }


    /// Lookup the position that goes with this index into the text
    /// Performs a binary search of the newline_table
    pub fn get_pos(&self, text_index: usize) -> Pos {
        let row = self.get_row_num(text_index);

        if row == 0 {
            Pos { row, col: text_index }
        } else {
            let col = text_index - self.row_start(row);
            Pos { row, col }
        }
    }

    fn get_row_num(&self, text_index: usize) -> usize {
        if self.newline_table.is_empty() {
            return 0;
        }

        let overall_low = self.newline_table[0];
        let overall_high = *self.newline_table.last().unwrap();

        if text_index <= overall_low {
            return 0;
        } else if text_index > overall_high {
            return self.newline_table.len();
        } else if self.newline_table.len() == 2 {
            return 1;
        }

        let mut left = 0;
        let mut right = self.newline_table.len();

        while left != right && left + 1 != right {
            let middle = ( left + right ) / 2;
            let middle_low = self.newline_table[middle];
            let middle_high = self.newline_table[middle + 1];

            if text_index == middle_low {
                return middle;
            }

            if text_index < middle_low {
                right = middle;
            } else if text_index > middle_high {
                left = middle;
            } else {
                return middle + 1;
            }
        }

        // println!("{} is 1 less than {}: result is {}", left, right, left+1);
        return left + 1;
    }

    /// Create a SpanSeq representing a division of this Input into a sequence of
    /// distinct regions called Spans associated with a value called its contents.
    /// The stops Vec contains the upper exclusive bounds for each Span 
    pub fn get_span_seq<T>(&self, stops: Vec<usize>, contents: Vec<T>) -> SpanSeq<'_, T> {
        SpanSeq {
            input: self,
            stops, contents
        }
    }

    /// Create a Span representing the specified region of this Input
    /// The start/stop pair is an inclusive/exclusive range
    pub fn get_span<T>(&self, start: usize, stop: usize, contents: T) -> Span<'_, T> {
        Span {
            input: self,
            start,
            stop,
            contents
        }
    }
}

/// Represents a division of an entire Input into contiguous Spans
#[derive(Clone, PartialEq, Eq)]
pub struct SpanSeq<'a, T> {
    /// The Input this is a part of
    input: &'a Input,
    /// The upper exclusive bounds for each Span
    stops: Vec<usize>,
    /// The contents associated with each Span
    contents: Vec<T>
}

impl <'a, T> SpanSeq<'a, T>
    where
        T: Copy {

    pub fn get_span(&self, index: usize) -> Span<'a, T> {
        let start = if index == 0 { 0 } else {
            self.stops[index - 1]
        };
        let stop = self.stops[index];
        let contents = self.contents[index];

        Span {
            input: self.input,
            start, stop, contents
        }
    }

    pub fn get_range_as_span<B>(&self, lower: usize, upper: usize, contents: B) -> Span<'a, B> {
        let start = if lower == 0 { 0 } else {
            self.stops[lower - 1]
        };
        let stop = self.stops[upper];
        Span {
            input: self.input,
            start, stop, contents
        }
    }

    pub fn map<B, F>(&self, func: F) -> SpanSeq<'a, B>
        where
            F: FnMut(&T) -> B {

        let input = self.input;
        let stops = self.stops.clone();
        let contents = self.contents.iter().map(func).collect();

        SpanSeq { input, stops, contents }
    }
}


#[derive(Clone, PartialEq, Eq)]
pub struct Span<'a, T> {
    input: &'a Input,
    start: usize,
    stop: usize,
    pub contents: T
}


impl<'a, T> Span<'a, T> 
    where
        T: Copy {

    pub fn map_contents<B, F>(&self, mut func: F) -> Span<'a, B>
        where
            F: FnMut(T) -> B {

        Span {
            input: self.input,
            start: self.start,
            stop: self.stop,
            contents: func(self.contents)
        }
    }
}

impl<'a, T> fmt::Display for Span<'a, T>
    where
        T: fmt::Display {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lower_pos = self.input.get_pos(self.start);
        let upper_pos = self.input.get_pos(self.stop);

        use std::cmp::{max, min};
        let margin_width = max(num_dec_digits(lower_pos.row), num_dec_digits(upper_pos.row)) + 1;
        let margin = " ".repeat(margin_width);
        let min_col = min(lower_pos.col, upper_pos.col);
        let max_col = max(lower_pos.col, upper_pos.col);

        write!(f, "{m    }--> {p}{lr}:{lc}\n\
                   {m    } |\n",
                p = "", //TODO Path
                lr = lower_pos.row,
                lc = lower_pos.col,
                m = margin)?;

        match upper_pos.row - lower_pos.row {
            0 => {
                write!(f, "{lr:w$} | {line}\n",
                        lr = lower_pos.row,
                        w = margin_width,
                        line = self.input.get_row_slice(lower_pos.row))?;
            },

            1 => {
                write!(f, "{lr:w$} | {line}\n\
                           {ur:w$} | {continued_line}\n",
                        lr = lower_pos.row,
                        ur = upper_pos.row,
                        w = margin_width,
                        line = self.input.get_row_slice(lower_pos.row),
                        continued_line = self.input.get_row_slice(upper_pos.row))?;
            },

            _ => {
                write!(f, "{lr:w$} | {line}\n\
                           {m    } | ...\n\
                           {ur:w$} | {continued_line}\n",
                        lr = lower_pos.row,
                        ur = upper_pos.row,
                        m = margin,
                        w = margin_width,
                        line = self.input.get_row_slice(lower_pos.row),
                        continued_line = self.input.get_row_slice(upper_pos.row))?;
            }
        };

        write!(f, "{m    } | {u}\n",
                m = margin,
                u = underline(min_col, max_col - min_col))?;

        write!(f, "{m    } |\n\
                   {m    } = {contents}",
                m = margin,
                contents = self.contents)?;
        Ok(())
    }
}

fn underline(offset: usize, len: usize) -> String {
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

fn num_dec_digits(num: usize) -> usize {
    format!("{}", num).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_based_tests() {
        let input = Input::new("1234\n5\n6\n78901\n234".into());
        assert_eq!(vec![4, 6, 8, 14], input.newline_table);

        assert_eq!(0, input.row_start(0));
        assert_eq!(5, input.row_start(1));
        assert_eq!(7, input.row_start(2));
        assert_eq!(9, input.row_start(3));
        assert_eq!(15, input.row_start(4));
        
        assert_eq!(4, input.row_end(0));
        assert_eq!(6, input.row_end(1));
        assert_eq!(8, input.row_end(2));
        assert_eq!(14, input.row_end(3));
        assert_eq!(18, input.row_end(4));

        assert_eq!(String::from("1234"),  input.get_row_slice(0));
        assert_eq!(String::from("5"),     input.get_row_slice(1));
        assert_eq!(String::from("6"),     input.get_row_slice(2));
        assert_eq!(String::from("78901"), input.get_row_slice(3));
        assert_eq!(String::from("234"),   input.get_row_slice(4));
    }

    #[test]
    fn newline_indices() {
        let num_newlines = 100;
        let input = Input::new("\n".repeat(num_newlines));
        let newlines: Vec<usize> = (0..num_newlines).collect();
        assert_eq!(newlines, input.newline_table);

        for i in 0..num_newlines {
            assert_eq!(i, input.get_row_num(i), "index is {}", i);
            let expected_pos = Pos { row: i, col: 0 };
            assert_eq!(expected_pos, input.get_pos(i), "index is {}", i);
        }
    }

    #[test]
    fn index_based_tests() {
        let input = Input::new("a;slajt\nleham\nc.a,mebuais;cmn\nbv,b\ne,mnbt\n".into());
        let newlines = vec![7, 13, 29, 34, 41];
        assert_eq!(newlines, input.newline_table);

        for i in 0..42 {
            let expected_row = match i {
                0..=7 => 0,
                8..=13 => 1,
                14..=29 => 2,
                30..=34 => 3,
                35..=41 => 4,
                _ => 10000
            };
            assert_eq!(expected_row, input.get_row_num(i), "index is {}", i);

            let expected_col = match expected_row {
                0 => i,
                _ => i - newlines[expected_row - 1] - 1
            };
            let expected_pos = Pos { row: expected_row, col: expected_col };

            assert_eq!(expected_pos, input.get_pos(i), "index is {}", i);
        }
    }
}
