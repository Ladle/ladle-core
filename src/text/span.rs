use std::fmt;

use super::annotation::{ AnnotationBuilder, Underline };
use super::Input;

impl Input {
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
}

#[derive(Clone, PartialEq, Eq)]
pub struct Span<'a, T> {
    input: &'a Input,
    start: usize,
    stop: usize,
    pub contents: T
}

impl<'a, T> fmt::Display for Span<'a, T>
    where
        T: fmt::Display {


    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = AnnotationBuilder::new(self.input);
        
        let lower_pos = self.input.get_pos(self.start);
        let upper_pos = self.input.get_pos(self.stop);

        match upper_pos.line - lower_pos.line {
            0 => {
                let underline = Underline {
                    start: lower_pos.col,
                    len: upper_pos.col - lower_pos.col
                };
                builder.add_line_underlined(lower_pos.line, underline);
            },
            _ => {
                let underline1 = Underline {
                    start: lower_pos.col,
                    len: self.input.get_line_end(lower_pos.line) - lower_pos.col
                };
                builder.add_line_underlined(lower_pos.line, underline1);
                
                let underline2 = Underline {
                    start: 0,
                    len: upper_pos.col
                };
                builder.add_line_underlined(lower_pos.line, underline2);
            }
        }

        builder.set_message(format!("{}", self.contents));

        write!(f, "{}", builder)
    }
}

