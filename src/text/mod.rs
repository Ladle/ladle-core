pub mod span;
pub mod annotation;

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
    pub line: usize,
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

    pub fn get_line_slice(&self, row: usize) -> &str {
        let min_index = self.get_line_start(row);
        let max_index = self.get_line_end(row);
        &self.text[min_index..max_index]
    }

    fn get_line_start(&self, line: usize) -> usize {
        if line == 0 {
            0
        } else {
            self.newline_table[line - 1] + 1
        }
    }

    fn get_line_end(&self, line: usize) -> usize {
        if line >= self.newline_table.len() {
            self.text.len()
        } else {
            self.newline_table[line]
        }
    }


    /// Lookup the position that goes with this index into the text
    /// Performs a binary search of the newline_table
    pub fn get_pos(&self, text_index: usize) -> Pos {
        let line = self.get_line_num(text_index);

        if line == 0 {
            Pos { line, col: text_index }
        } else {
            let col = text_index - self.get_line_start(line);
            Pos { line, col }
        }
    }

    fn get_line_num(&self, text_index: usize) -> usize {
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
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_based_tests() {
        let input = Input::new("1234\n5\n6\n78901\n234".into());
        assert_eq!(vec![4, 6, 8, 14], input.newline_table);

        assert_eq!(0, input.get_line_start(0));
        assert_eq!(5, input.get_line_start(1));
        assert_eq!(7, input.get_line_start(2));
        assert_eq!(9, input.get_line_start(3));
        assert_eq!(15, input.get_line_start(4));
        
        assert_eq!(4, input.get_line_end(0));
        assert_eq!(6, input.get_line_end(1));
        assert_eq!(8, input.get_line_end(2));
        assert_eq!(14, input.get_line_end(3));
        assert_eq!(18, input.get_line_end(4));

        assert_eq!(String::from("1234"),  input.get_line_slice(0));
        assert_eq!(String::from("5"),     input.get_line_slice(1));
        assert_eq!(String::from("6"),     input.get_line_slice(2));
        assert_eq!(String::from("78901"), input.get_line_slice(3));
        assert_eq!(String::from("234"),   input.get_line_slice(4));
    }

    #[test]
    fn newline_indices() {
        let num_newlines = 100;
        let input = Input::new("\n".repeat(num_newlines));
        let newlines: Vec<usize> = (0..num_newlines).collect();
        assert_eq!(newlines, input.newline_table);

        for i in 0..num_newlines {
            assert_eq!(i, input.get_line_num(i), "index is {}", i);
            let expected_pos = Pos { line: i, col: 0 };
            assert_eq!(expected_pos, input.get_pos(i), "index is {}", i);
        }
    }

    #[test]
    fn index_based_tests() {
        let input = Input::new("a;slajt\nleham\nc.a,mebuais;cmn\nbv,b\ne,mnbt\n".into());
        let newlines = vec![7, 13, 29, 34, 41];
        assert_eq!(newlines, input.newline_table);

        for i in 0..42 {
            let expected_line = match i {
                0..=7 => 0,
                8..=13 => 1,
                14..=29 => 2,
                30..=34 => 3,
                35..=41 => 4,
                _ => 10000
            };
            assert_eq!(expected_line, input.get_line_num(i), "index is {}", i);

            let expected_col = match expected_line {
                0 => i,
                _ => i - newlines[expected_line - 1] - 1
            };
            let expected_pos = Pos { line: expected_line, col: expected_col };

            assert_eq!(expected_pos, input.get_pos(i), "index is {}", i);
        }
    }
}
