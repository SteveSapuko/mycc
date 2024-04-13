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

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Key(s) => write!(f, "{}", s),
            Self::Op(s) => write!(f, "{}", s),
            Self::Cond(s) => write!(f, "{}", s),
            Self::Id(s) => write!(f, "{}", s),
            Self::Lit(s) => write!(f, "{}", s),
            Self::ParenOpen => write!(f, "("),
            Self::ParenClose => write!(f, ")"),
            Self::SquareOpen => write!(f, "["),
            Self::SquareClose => write!(f, "]"),
            Self::CurlyOpen => write!(f, "{{"),
            Self::CurlyClose => write!(f, "}}"),
            Self::SemiCol => write!(f, ";"),
            Self::Col => write!(f, ":"),
            Self::Comma => write!(f, ","),
            Self::Period => write!(f, "."),
            Self::Arrow => write!(f, "->"),
            Self::EOF => write!(f, "EOF"),
        }
    }
}

impl std::fmt::Display for Lexeme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tok)
    }
}