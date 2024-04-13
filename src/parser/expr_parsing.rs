use std::f32::consts::E;

use super::*;

impl Parser {
    pub fn parse_expr(&mut self) -> Result<Box<dyn Expr>, &'static str> {
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
        let mut left: Box<dyn Expr> = self.parse_fn_call()?;

        if self.match_tok(Token::Op("as".to_string())) {
            if !self.match_tok_type(ID_TOKEN) {
                return Err("Expected Identifier for Cast Type")
            }

            let to_type = self.previous();

            left = Box::new(CastExpr {value: left, to_type});
        }

        return Ok(left)
    }

    fn parse_fn_call(&mut self) -> Result<Box<dyn Expr>, &'static str> {
        if self.look_ahead().tok == Token::ParenOpen {
            if !self.match_tok_type(ID_TOKEN) {
                return Err("Expected Identifier for Function Name")
            }

            let name = self.previous();

            self.match_tok(Token::ParenOpen);

            let args = self.parse_args()?;

            if !self.match_tok(Token::ParenClose) {
                return Err("Expected Closing Parentheses after Function Call")
            }
            return Ok(Box::new(FnCall {name, args}))
            
        }

        Ok(Box::new(self.parse_primary()?))
    }

    fn parse_args(&mut self) -> Result<Args, &'static str> {
        let mut args: Vec<Box<dyn Expr>> = vec![];

        if self.match_tok(Token::ParenClose) || self.match_tok(Token::SquareClose) || self.match_tok(Token::CurlyClose) {
            self.go_back();
            return Ok(Args { items: args })
        }
        
        let e = self.parse_expr()?;
        args.push(e);

        while self.match_tok(Token::Comma) {
            let e = self.parse_expr()?;
            args.push(e);
        }

        return Ok(Args { items: args })
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
        
        //Ref
        if self.match_tok(Token::Op("&".to_string())) || self.match_tok(Token::Op("*".to_string())) {
            let operator = self.previous();
            let ref_of = self.parse_variable()?;

            return Ok(PrimaryExpr::Ref(operator, ref_of))   
        }

        if self.look_ahead().tok == Token::Col {
            if !self.match_tok_type(ID_TOKEN) {
                return Err("Expected Identifier for Enum Name")
            }
            let enum_name = self.previous();
            self.match_tok(Token::Col);

            if !self.match_tok(Token::Col) {
                return Err("Expected Double Colon for Enum Variant")
            }

            if !self.match_tok_type(ID_TOKEN) {
                return Err("Expected Double Colon for Enum Variant")
            }

            let variant = self.previous();

            return Ok(PrimaryExpr::EnumVariant(enum_name, variant))
        }

        //Variable Access
        let var = self.parse_variable()?;
        return Ok(PrimaryExpr::Variable(var))
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