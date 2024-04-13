use crate::token::*;
use crate::expr::*;
use std::mem::discriminant;

pub struct Parser {
    data: Vec<Lexeme>,
    ptr: usize,
}

impl Parser {
    pub fn new(data: Vec<Lexeme>) -> Self {
        Self {
            data,
            ptr: 0
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Box<dyn Expr>>, &'static str> {
        let mut ast: Vec<Box<dyn Expr>> = vec![];

        while self.current().tok != Token::EOF {
            ast.push(self.parse_expr()?);
        }

        Ok(ast)
    }
    
    fn match_tok(&mut self, t: Token) -> bool {
        if self.current().tok == t {
            self.ptr += 1;
            return true
        }

        false
    }

    fn match_tok_type(&mut self, t: Token) -> bool {
        let temp = self.current();

        if discriminant(&temp.tok) == discriminant(&t) {
            self.ptr += 1;
            return true
        }

        false
    }

    pub fn current(&self) -> Lexeme {
        self.data[self.ptr].clone()
    }

    fn previous(&self) -> Lexeme {
        self.data[self.ptr - 1].clone()
    }

    fn look_ahead(&self) -> Lexeme {
        self.data[self.ptr + 1].clone()
    }

    fn go_back(&mut self) {
        self.ptr -= 1;
    }

    fn parse_expr(&mut self) -> Result<Box<dyn Expr>, &'static str> {
        self.parse_assign()
    }

    fn parse_assign(&mut self) -> Result<Box<dyn Expr>, &'static str> {
        let left = self.parse_equality()?;
        
        if self.match_tok(Token::Op("=".to_string())) {
            let op = self.previous();
            let right = self.parse_expr()?;
            return Ok(Box::new(BinaryExpr {
                left: left,
                operator: op,
                right: right
            }))
        }
        
        return Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Box<dyn Expr>, &'static str> {
        let mut left = self.parse_comparison()?;

        while self.match_tok(Token::Cond("==".to_string())) || self.match_tok(Token::Cond("!=".to_string())) {
            let op = self.previous();
            let right = self.parse_comparison()?;

            left = Box::new(BinaryExpr {left, operator: op, right});
        }

        return Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Box<dyn Expr>, &'static str> {
        let mut left = self.parse_term()?;

        while self.match_tok(Token::Cond("<".to_string())) || self.match_tok(Token::Cond(">".to_string())) ||
            self.match_tok(Token::Cond("<=".to_string())) || self.match_tok(Token::Cond(">=".to_string())) {

            let op = self.previous();
            let right = self.parse_term()?;

            left = Box::new(BinaryExpr {left, operator: op, right});
        }

        return Ok(left)
    }

    fn parse_term(&mut self) -> Result<Box<dyn Expr>, &'static str> {
        let mut left = self.parse_shift()?;

        while self.match_tok(Token::Op("-".to_string())) || self.match_tok(Token::Op("+".to_string())) {

            let op = self.previous();
            let right = self.parse_shift()?;

            left = Box::new(BinaryExpr {left, operator: op, right});
        }

        return Ok(left)
    }

    fn parse_shift(&mut self) -> Result<Box<dyn Expr>, &'static str> {
        let mut left = self.parse_unary()?;

        while self.match_tok(Token::Op("<<".to_string())) || self.match_tok(Token::Op(">>".to_string())) {

            let op = self.previous();
            let right = Box::new(self.parse_primary()?);

            left = Box::new(BinaryExpr {left, operator: op, right});
        }

        return Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Box<dyn Expr>, &'static str> {
        let right: Box<dyn Expr>;
        
        if self.match_tok(Token::Op("-".to_string())) || self.match_tok(Token::Op("!".to_string())) {
            let op = self.previous();
            right = Box::new(UnaryExpr {operator: op, right: self.parse_cast()?});

        } else {
            right = self.parse_cast()?;
        }


        return Ok(right);
    }

    fn parse_cast(&mut self) -> Result<Box<dyn Expr>, &'static str> {
        let mut left: Box<dyn Expr> = Box::new(self.parse_primary()?);

        if self.match_tok(Token::Op("as".to_string())) {
            if !self.match_tok_type(ID_TOKEN) {
                return Err("Expected Identifier for Cast Type")
            }

            let to_type = self.previous();

            left = Box::new(CastExpr {value: left, to_type});
        }

        return Ok(left)
    }

    fn parse_primary(&mut self) -> Result<PrimaryExpr, &'static str> {
        //Grouping
        if self.match_tok(Token::ParenOpen) {
            let e = self.parse_assign()?;

            if !self.match_tok(Token::ParenClose) {
                return Err("Expected Closing Parentheses After Grouping")
            }

            return Ok(PrimaryExpr::Grouping(e))
        }

        //Number Literal
        if self.match_tok_type(LIT_TOKEN) {
            let lit = self.previous();
            
            let num = match lit.data().parse::<u64>() {
                Ok(t) => t,
                Err(_) => {
                    self.go_back();
                    return Err("Number Literal Error")
                }
            };

            if num < u32::MAX.into() {
                if num < u16::MAX.into() {
                    if num < u8::MAX.into() {
                        return Ok(PrimaryExpr::NumLiteral(NumLiteral::U8(num as u8)))
                    }
                    return Ok(PrimaryExpr::NumLiteral(NumLiteral::U16(num as u16)))
                }
                return Ok(PrimaryExpr::NumLiteral(NumLiteral::U32(num as u32)))
            }

            return Ok(PrimaryExpr::NumLiteral(NumLiteral::U64(num)))
        }
        
        if self.match_tok(Token::Op("&".to_string())) || self.match_tok(Token::Op("*".to_string())) {
            let operator = self.previous();
            let ref_of = self.parse_variable()?;

            return Ok(PrimaryExpr::Ref(operator, ref_of))   
        }

        //Variable Access
        if self.look_ahead().tok != Token::Col {
            let var = self.parse_variable()?;
            return Ok(PrimaryExpr::Variable(var))
        }

        return Err("PrimaryExpr Error")
    }

    fn parse_variable(&mut self) -> Result<Variable, &'static str> {
        if self.match_tok_type(ID_TOKEN) {
            let id = self.previous();

            let mut result = Variable::Id(id);
            
            while self.current().tok == Token::Period || self.current().tok == Token::SquareOpen {
                if self.match_tok(Token::Period) {
                    let field = self.parse_variable()?;

                    result = Variable::StructField(Box::new((result, field)));
                }

                if self.match_tok(Token::SquareOpen) {
                    let index = self.parse_expr()?;
                    
                    if !self.match_tok(Token::SquareClose) {
                        return Err("Expected Closing Square Bracket for Array Index")
                    }

                    result = Variable::Array(Box::new(result), index);
                }
            }

            return Ok(result)
        }
        
        return Err("Variable Access Parsing Error")
    }
}