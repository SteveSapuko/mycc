#[derive(Debug)]
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

#[derive(Debug)]
pub struct Lexeme {
    tok: Token,
    line: usize,
    col: usize,
}

impl Lexeme {
    pub fn new(line: usize, col: usize, tok: Token) -> Self {
        Self {
            line: line,
            col: col,
            tok: tok
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Key(s) => write!(f, "Key {}", s),
            Self::Op(s) => write!(f, "Op {}", s),
            Self::Cond(s) => write!(f, "Cond {}", s),
            Self::Id(s) => write!(f, "Id {}", s),
            Self::Lit(s) => write!(f, "Lit {}", s),
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
        write!(f, "{} at ln: {} col: {}", self.tok, self.line, self.col)
    }
}