use super::*;

impl Parser {
    pub fn parse_expr(&mut self) -> Result<Expr, &'static str> {
        self.parse_assign()
    }

    fn parse_assign(&mut self) -> Result<Expr, &'static str> {
        let mut left = self.parse_equality()?;

        if self.match_tok(Token::Op("=".to_string())) {
            let op = self.previous();
            let right = self.parse_assign()?;

            left = Expr::Assign(Box::new(BinaryExpr { left , operator: op, right }))
        }

        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, &'static str> {
        let mut left = self.parse_comparison()?;

        while self.match_tok(Token::Cond("==".to_string())) || self.match_tok(Token::Cond("!=".to_string())) {
            let operator = self.previous();
            let right = self.parse_comparison()?;

            left = Expr::Equality(Box::new(BinaryExpr { left, operator, right }))
        }


        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, &'static str> {
        let mut left = self.parse_term()?;

        while self.match_tok(Token::Cond(">".to_string())) || self.match_tok(Token::Cond("<".to_string())) ||
        self.match_tok(Token::Cond(">=".to_string())) || self.match_tok(Token::Cond("<=".to_string()))  ||
        self.match_tok(Token::Cond("&&".to_string())) || self.match_tok(Token::Cond("||".to_string())) {
            let operator = self.previous();
            let right = self.parse_term()?;

            left = Expr::Comparison(Box::new(BinaryExpr { left, operator, right }))
        }


        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr, &'static str> {
        let mut left = self.parse_shift()?;

        while self.match_tok(Token::Op("+".to_string())) || self.match_tok(Token::Op("-".to_string())) ||
        self.match_tok(Token::Op("&".to_string())) || self.match_tok(Token::Op("|".to_string())) ||
        self.match_tok(Token::Op("~|".to_string())) {
            let operator = self.previous();
            let right = self.parse_shift()?;

            left = Expr::Term(Box::new(BinaryExpr { left, operator, right }))
        }


        Ok(left)
    }

    fn parse_shift(&mut self) -> Result<Expr, &'static str> {
        let left = self.parse_unary()?;

        if self.match_tok(Token::Op("<<".to_string())) || self.match_tok(Token::Op(">>".to_string())) {
            let operator = self.previous();

            let shift_amount = self.parse_num_literal()?;

            return Ok(Expr::Shift(Box::new(left), operator, shift_amount))
        }


        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, &'static str> {
        if self.match_tok(Token::Op("!".to_string())) || self.match_tok(Token::Op("-".to_string())) {
            let operator = self.previous();

            let right = self.parse_cast()?;

            return Ok(Expr::Unary(operator, Box::new(right)))

        }

        Ok(self.parse_cast()?)
    }

    fn parse_cast(&mut self) -> Result<Expr, &'static str> {
        let left = self.parse_fn_call()?;

        if self.match_tok(Token::Op("as".to_string())) {
            let operator = self.previous();
            let to_type = self.parse_type_declr()?;

            return Ok(Expr::Cast(Box::new(left), operator, to_type))
        }

        Ok(left)
    }

    fn parse_fn_call(&mut self) -> Result<Expr, &'static str> {
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

            return Ok(Expr::FnCall(name, args))
        }

        Ok(self.parse_primary()?)
    }

    fn parse_primary(&mut self) -> Result<Expr, &'static str> {
        //NumLiteral
        if matches!(self.current().tok, Token::Lit(_)) {
            let lexeme = self.current();
            let num_literal = self.parse_num_literal()?;
            return Ok(Expr::Primary(Box::new(PrimaryExpr::NumLiteral(num_literal, lexeme))))

        }

        //Grouping
        if self.match_tok(Token::ParenOpen) {
            let e = self.parse_expr()?;

            if !self.match_tok(Token::ParenClose) {
                return Err("Expected Closing Parentheses")
            }

            return Ok(Expr::Primary(Box::new(PrimaryExpr::Grouping(e))))
        }

        //Ref
        if self.match_tok(Token::Op("&".to_string())) || self.match_tok(Token::Op("*".to_string())) {
            let operator = self.previous();
            let var = self.parse_var()?;

            return Ok(Expr::Primary(Box::new(PrimaryExpr::Ref(operator, var))))
        }

        if self.look_ahead().tok == Token::Col {
            if !self.match_tok_type(ID_TOKEN) {
                return Err("Expected Identifier for Enum Name")
            }

            let name = self.previous();
            self.match_tok(Token::Col);
            if !self.match_tok(Token::Col) {
                return Err("Expected Double Colon :: after Enum Name")
            }

            if !self.match_tok_type(ID_TOKEN) {
                return Err("Expected Identifier for Enum Variant")
            }
            let variant = self.previous();

            return Ok(Expr::Primary(Box::new(PrimaryExpr::EnumVariant(name, variant))))

        }
        
        //Variable Access
        let var = self.parse_var()?;

        Ok(Expr::Primary(Box::new(PrimaryExpr::Variable(var))))
    }

    fn parse_num_literal(&mut self) -> Result<NumLiteral, &'static str> {
        if self.match_tok_type(LIT_TOKEN) {
            let lit = self.previous();
            
            let num = match lit.data().parse::<u64>() {
                Ok(t) => t,
                Err(_) => return Err("Number Literal Error")
            };

            if num <= u32::MAX.into() {
                if num <= u16::MAX.into() {
                    if num <= u8::MAX.into() {
                        return Ok(NumLiteral::U8(num as u8))
                    }
                    return Ok(NumLiteral::U16(num as u16))
                }
                return Ok(NumLiteral::U32(num as u32))
            }

            return Ok(NumLiteral::U64(num))
        }

        Err("Expected Number Literal")
    }

    fn parse_args(&mut self) -> Result<Args, &'static str> {
        let mut args: Vec<Expr> = vec![];

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

    fn parse_var(&mut self) -> Result<Variable, &'static str> {
        if !self.match_tok_type(ID_TOKEN) {
            return Err("Variable Access Parsing Error")    
        }

        let id = self.previous();
        let mut result = Variable::Id(id);

        while self.current().tok == Token::Period || self.current().tok == Token::SquareOpen {
            if self.match_tok(Token::Period) {
                let field = self.parse_var()?;

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
}