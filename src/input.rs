use std::fmt;
use std::path::PathBuf;

/// Input represents input to tokenizing and parsing operations
/// It contains text and associated metadata
#[derive(Clone, PartialEq, Eq)]
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
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    /// The row index of the position
    pub row: usize,
    /// The column index of the position
    pub col: usize
}

impl Input {
    /// Borrows the text as a &str
    pub fn as_str(&self) -> &str {
        &self.text
    }


    /// Lookup the position that goes with this index into the text
    /// Performs a binary search of the newline_table
    pub fn get_pos(&self, text_index: usize) -> Pos {
        let newline_index = self.find_nearest_newline(text_index, 0);
        let row = newline_index;
        let col = text_index - self.newline_table[newline_index];

        Pos { row, col }
    }
    
    /// Lookup the position that goes with this index into the text using a hint
    /// The hint is the lowest row that the input could possibly be on
    /// This restricts the binary search to items after this
    pub fn get_pos_hint(&self, text_index: usize, min_row: usize) -> Pos {
        let newline_index = self.find_nearest_newline(text_index, min_row);
        let row = newline_index;
        let col = self.newline_table[newline_index];

        Pos { row, col }
    }

    fn find_nearest_newline(&self, text_index: usize, min_row: usize) -> usize {
        let mut left = min_row;
        let mut right = self.newline_table.len() - 1;

        if text_index < self.newline_table[left] {
            return left;
        }
        if text_index > self.newline_table[right] {
            return right;
        }

        while left != right {
            let middle = ( left + right ) / 2;
            let middle_val = self.newline_table[middle];

            if text_index == middle_val {
                return middle;
            } else if text_index < middle_val {
                right = middle;
            } else if text_index > middle_val {
                left = middle;
            }
        }

        return left;
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

impl <'a, T> SpanSeq<'a, T> {
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
        let stops = self.stops;
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


impl<'a, T> Span<'a, T> {
    pub fn map_contents<B, F>(&self, func: F) -> Span<'a, B>
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
        let upper_pos = self.input.get_pos_hint(self.stop, lower_pos.row);

        use std::cmp::max;
        let margin_width = max(num_dec_digits(lower_pos.row), num_dec_digits(upper_pos.row)) + 1;
        let margin = " ".repeat(margin_width);

        write!(f, "{m    }-->{lr}:{lc}\n\
                   {m    } |\n",
                lr = lower_pos.row,
                lc = lower_pos.col,
                m = margin)?;

        let row_diff = upper_pos.row - lower_pos.row;
        match row_diff {
            0 => {
                write!(f, "{lr:w$} | {line}\n",
                        lr = lower_pos.row,
                        w = margin_width,
                        line = "")?; // TODO: line
            },

            1 => {
                write!(f, "{lr:w$} | {line}\n\
                           {ur:w$} | {continued_line}\n",
                        lr = lower_pos.row,
                        ur = upper_pos.row,
                        w = margin_width,
                        line = "", continued_line = "")?; // TODO: line, continued_line
            },

            _ => {
                write!(f, "{lr:w$} | {line}\n\
                           {m    } | ...\n\
                           {ur:w$} | {continued_line}\n",
                        lr = lower_pos.row,
                        ur = upper_pos.row,
                        m = margin,
                        w = margin_width,
                        line = "", continued_line = "")?; // TODO: line, continued_line
            }
        };

        // TODO: Add underline

        write!(f, "{m    } |\n\
                   {m    } = {contents}",
                m = margin,
                contents = self.contents)?;
        Ok(())
    }
}

fn num_dec_digits(num: usize) -> usize {
    format!("{}", num).len()
}
