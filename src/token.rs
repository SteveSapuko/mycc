#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Key(String),
    Op(String),
    Cond(String),
    Id(String),
    Lit(String),
    ParenOpen,
    ParenClose,
    SquareOpen,
    SquareClose,
    CurlyOpen,
    CurlyClose,
    SemiCol,
    Col,
    Comma,
    Period,
    Arrow,
    EOF,
}

pub const ID_TOKEN: Token = Token::Id(String::new());
pub const LIT_TOKEN: Token = Token::Lit(String::new());

pub const BLANK_LEXEME: Lexeme = Lexeme {
    tok: Token::Arrow,
    line: 0,
    col: 0
};

#[derive(Debug, Clone, PartialEq)]
pub struct Lexeme {
    pub tok: Token,
    pub line: usize,
    pub col: usize,
}

impl Lexeme {
    pub fn new(line: usize, col: usize, tok: Token) -> Self {
        Self {
            line: line,
            col: col,
            tok: tok
        }
    }

    pub fn data(&self) -> String {
        match self.tok.clone() {
            Token::Key(s) => s,
            Token::Op(s) => s,
            Token::Cond(s) => s,
            Token::Id(s) => s,
            Token::Lit(s) => s,
            _ => panic!("No data for this token")
        }
    }
}