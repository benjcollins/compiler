#[derive(Copy, Clone, Debug)]
pub struct Position<'a> {
    line: i32,
    column: i32,
    source: &'a str,
}

impl<'a> Position<'a> {
    pub fn new(line: i32, column: i32, source: &'a str) -> Position<'a> {
        Position { line, column, source }
    }
    pub fn from_source(source: &'a str) -> Position<'a> {
        Position::new(1, 1, source)
    }
    pub fn next(&self) -> Option<(Position<'a>, char)> {
        let mut chars = self.source.chars();
        let ch = chars.next()?;
        Some(if ch == '\n' {
            (Position::new(self.line + 1, 0, chars.as_str()), '\n')
        } else {
            (Position::new(self.line, self.column + 1, chars.as_str()), ch)
        })
    }
    pub fn len(&self) -> usize {
        self.source.len()
    }
    pub fn slice(start: Position<'a>, end: Position<'a>) -> &'a str {
        &start.source[0..start.len()-end.len()]
    }
    pub fn next_while<F: Fn(char) -> bool>(&self, condition: F) -> Position<'a> {
        match self.next() {
            Some((pos, ch)) if condition(ch) => pos.next_while(condition),
            _ => *self
        }
    }
}