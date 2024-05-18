use crate::token::*;
use fancy_regex::Regex;

#[derive(Debug)]
pub struct Lexer {
    data: String,
    ptr: usize,
    pub line: usize,
    pub col: usize,
}

impl Lexer {
    pub fn new(text: String) -> Result<Self, ()> {
        if !text.is_ascii() {
            return Err(())
        }

        Ok(Lexer {
            data: text,
            ptr: 0,
            line: 1,
            col: 1 })
    }

    pub fn lex(&mut self) -> Result<Vec<Lexeme>, ()> {
        let reg_key = Regex::new(r"(?x)
        ^let |
        ^if |
        ^fn |
        ^else |
        ^while |
        ^loop |
        ^for |
        ^return |
        ^continue |
        ^struct |
        ^enum |
        ^break").unwrap();

        let reg_op = Regex::new(r"(?x)
        ^ as(?=\s) |
        ^ \= (?!\=) |
        ^ \+ |
        ^ \- |
        ^ \& (?!\&)|
        ^ \* |
        ^ << |
        ^ >> |
        ^ \|(?!\|) | #single pipe
        ^ \! (?!\=)|
        ^ \~\|").unwrap();
        
        let reg_cond = Regex::new(r"(?x)
        ^ \|\| |
        ^ \&\& |
        ^ <\= |
        ^ >= |
        ^ < (?!<) |
        ^ > (?!>) |
        ^ \!\= |
        ^ \=\=").unwrap();

        let reg_id = Regex::new(r"^[_[[:alpha:]]][_@[[:alnum:]]]*").unwrap();
        
        let reg_lit = Regex::new(r"^-?\d+(?![[:alpha:]])").unwrap();

        let reg_arrow = Regex::new(r"^\->").unwrap();

        let mut lexeme_list: Vec<Lexeme> = vec![];

        while self.current() != 0 {
            self.skip_whitespace();

            if let Some(m) = reg_arrow.find(&self.data.as_str()[self.ptr..]).unwrap() {
                lexeme_list.push(Lexeme::new(self.line, self.col, Token::Arrow));
                self.advance(m.as_str().len());
                continue;
            }

            if let Some(m) = reg_key.find(&self.data.as_str()[self.ptr..]).unwrap() {
                lexeme_list.push(Lexeme::new(self.line, self.col, Token::Key(m.as_str().to_string())));
                self.advance(m.as_str().len());
                continue;
            }

            if let Some(m) = reg_op.find(&self.data.as_str()[self.ptr..]).unwrap() {
                lexeme_list.push(Lexeme::new(self.line, self.col, Token::Op(m.as_str().to_string())));
                self.advance(m.as_str().len());
                continue;
            }

            if let Some(m) = reg_cond.find(&self.data.as_str()[self.ptr..]).unwrap() {
                lexeme_list.push(Lexeme::new(self.line, self.col, Token::Cond(m.as_str().to_string())));
                self.advance(m.as_str().len());
                continue;
            }

            if let Some(m) = reg_id.find(&self.data.as_str()[self.ptr..]).unwrap() {
                lexeme_list.push(Lexeme::new(self.line, self.col, Token::Id(m.as_str().to_string())));
                self.advance(m.as_str().len());
                continue;
            }

            if let Some(m) = reg_lit.find(&self.data.as_str()[self.ptr..]).unwrap() {
                lexeme_list.push(Lexeme::new(self.line, self.col, Token::Lit(m.as_str().to_string())));
                self.advance(m.as_str().len());
                continue;
            }

            let mut no_match = false;

            match self.current() {
                b'(' => lexeme_list.push(Lexeme::new(self.line, self.col, Token::ParenOpen)),
                b')' => lexeme_list.push(Lexeme::new(self.line, self.col, Token::ParenClose)),
                b'[' => lexeme_list.push(Lexeme::new(self.line, self.col, Token::SquareOpen)),
                b']' => lexeme_list.push(Lexeme::new(self.line, self.col, Token::SquareClose)),
                b'{' => lexeme_list.push(Lexeme::new(self.line, self.col, Token::CurlyOpen)),
                b'}' => lexeme_list.push(Lexeme::new(self.line, self.col, Token::CurlyClose)),
                b';' => lexeme_list.push(Lexeme::new(self.line, self.col, Token::SemiCol)),
                b':' => lexeme_list.push(Lexeme::new(self.line, self.col, Token::Col)),
                b',' => lexeme_list.push(Lexeme::new(self.line, self.col, Token::Comma)),
                b'.' => lexeme_list.push(Lexeme::new(self.line, self.col, Token::Period)),
                _ => no_match = true
            }

            if no_match {
                return Err(())
            }

            self.advance(1);
        }

        lexeme_list.push(Lexeme::new(self.line, self.col, Token::EOF));
        Ok(lexeme_list)
    }

    fn current(&self) -> u8 {
        return self.data.as_bytes()[self.ptr]
    }

    fn advance(&mut self, n: usize) {
        for _ in 0..n {
            if self.current() == b'\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            self.ptr += 1;
        }
    }

    fn skip_whitespace(&mut self) {
        while (self.current() == b' ' || self.current() == b'\n' || self.current() == b'\t' || self.current() == b'\r') && self.current() != 0 {
            self.advance(1);
        }
    }
}