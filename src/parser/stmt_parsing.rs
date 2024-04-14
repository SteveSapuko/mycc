use super::*;

impl Parser {
    pub fn parse_type_declr(&mut self) -> Result<TypeDeclr, &'static str> {
        if self.match_tok(Token::Key("@".to_string())) {
            let points_to = self.parse_type_declr()?;
            return Ok(TypeDeclr::Pointer(Box::new(points_to)))
        }

        if self.match_tok(Token::SquareOpen) {
            let array_of_type = self.parse_type_declr()?;
            
            if !self.match_tok(Token::SemiCol) {
                return Err("Expected Semicolon after Array Item Type")
            }

            if !self.match_tok_type(LIT_TOKEN) {
                return Err("Expected Number Literal for Array Size")
            }

            let array_size = self.previous();

            let array_size = match array_size.data().parse::<u16>() {
                Ok(t) => t,
                Err(_) => return Err("Array Size Error")
            };

            if !self.match_tok(Token::SquareClose) {
                return Err("Expected Closing Square Bracket for Array Type Declaration")
            }

            return Ok(TypeDeclr::Array(Box::new(array_of_type), array_size))
        }

        if self.match_tok_type(ID_TOKEN) {
            return Ok(TypeDeclr::Basic(self.previous()))
        }

        return Err("Expected Identifier for Type Declaration")
    }

    pub fn parse_stmt(&mut self) -> Result<Stmt, &'static str> {
        //VarDeclr
        if self.match_tok(Token::Key("let".to_string())) {
            if !self.match_tok_type(ID_TOKEN) {
                return Err("Expected Identifier for Variable Name")
            }
            let name = self.previous();

            if !self.match_tok(Token::Col) {
                return Err("Expected Colon after Variable Name")
            }

            let var_type = self.parse_type_declr()?;
            
            let mut value = None;
            if self.match_tok(Token::Op("=".to_string())) {
                let e = self.parse_expr()?;
                value = Some(e);
            }

            if !self.match_tok(Token::SemiCol) {
                return Err("Expected Semicolon after Variable Declaration")
            }

            return Ok(Stmt::VarDeclr(name, var_type, value))
        }

        //FnDeclr
        if self.match_tok(Token::Key("fn".to_string())) {
            if !self.match_tok_type(ID_TOKEN) {
                return Err("Expected Identifier for Function Name")
            }
            let name = self.previous();

            if !self.match_tok(Token::ParenOpen) {
                return Err("Expected Opening Parentheses after Function Name")
            }

            let params = self.parse_parameters()?;

            if !self.match_tok(Token::ParenClose) {
                return Err("Expected Closing Parentheses after Function Parameters")
            }

            if !self.match_tok(Token::Arrow) {
                return Err("Expected Arrow to Denote Return Type")
            }

            let ret_type = self.parse_type_declr()?;

            let body = self.parse_block()?;

            return Ok(Stmt::FnDeclr(name, params, ret_type, Box::new(body)))
        }

        //StructDeclr
        if self.match_tok(Token::Key("struct".to_string())) {
            if !self.match_tok_type(ID_TOKEN) {
                return Err("Expected Identifier for Struct Name")
            }
            let name = self.previous();

            if !self.match_tok(Token::CurlyOpen) {
                return Err("Expected Opening Curly Bracket for Struct Definition")
            }

            let fields = self.parse_parameters()?;

            if !self.match_tok(Token::CurlyClose) {
                return Err("Expected Closing Curly Bracket after Struct Definition")
            }

            return Ok(Stmt::StructDeclr(name, fields))
        }

        //EnumDeclr
        if self.match_tok(Token::Key("enum".to_string())) {
            if !self.match_tok_type(ID_TOKEN) {
                return Err("Expected Identifier for Enum Name")
            }
            let name = self.previous();

            if !self.match_tok(Token::CurlyOpen) {
                return Err("Expected Opening Curly Bracket after Enum Name")
            }

            let mut variants: Vec<Lexeme> = vec![];

            if self.match_tok_type(ID_TOKEN) {
                variants.push(self.previous());

                while self.match_tok(Token::Comma) {
                    if !self.match_tok_type(ID_TOKEN) {
                        return Err("Expected Identifier for Enum Variant Name")
                    }

                    variants.push(self.previous());
                }

            }

            if !self.match_tok(Token::CurlyClose) {
                return Err("Expected Closing Curly Bracket after Enum Definition")
            }

            return Ok(Stmt::EnumDeclr(name, variants))
        }

        //LoopStmt
        if self.match_tok(Token::Key("loop".to_string())) {
            let body = self.parse_block()?;

            return Ok(Stmt::LoopStmt(Box::new(body)))
        }

        //WhileStmt
        if self.match_tok(Token::Key("while".to_string())) {
            let cond = self.parse_expr()?;
            let body = self.parse_block()?;

            return Ok(Stmt::WhileStmt(cond, Box::new(body)))
        }

        //IfStmt
        if self.match_tok(Token::Key("if".to_string())) {
            let cond = self.parse_expr()?;
            let t_branch = self.parse_block()?;
            let mut f_branch = None;
            
            if self.match_tok(Token::Key("else".to_string())) {
                f_branch = Some(Box::new(self.parse_block()?));
            }

            return Ok(Stmt::IfStmt(cond, Box::new(t_branch), f_branch))

        }

        //ReturnStmt
        if self.match_tok(Token::Key("return".to_string())) {
            let key = self.previous();

            let value = self.parse_expr()?;
            return Ok(Stmt::ReturnStmt(key, value))
        }

        //BreakStmt
        if self.match_tok(Token::Key("break".to_string())) {
            return Ok(Stmt::BreakStmt(self.previous()))
        }

        //ExprStmt
        let e = self.parse_expr()?;
        if !self.match_tok(Token::SemiCol) {
            return Err("Expected Semicolon after Expression Statement")
        }

        return Ok(Stmt::ExprStmt(e))
    }

    fn parse_block(&mut self) -> Result<Stmt, &'static str> {
        if !self.match_tok(Token::CurlyOpen) {
            return Err("Expected Opening Curly Bracket for Block")
        }

        let mut body: Vec<Stmt> = vec![];

        while !self.match_tok(Token::CurlyClose) {
            body.push(self.parse_stmt()?);
        }

        return Ok(Stmt::Block(body))
    }

    fn parse_parameters(&mut self) -> Result<Parameters, &'static str> {
        let mut params: Vec<(Lexeme, TypeDeclr)> = vec![];

        if self.match_tok_type(ID_TOKEN) {
            let name = self.previous();

            if !self.match_tok(Token::Col) {
                return Err("Expected Colon after Parameter Name")
            }

            let t = self.parse_type_declr()?;
            params.push((name, t));

            while self.match_tok(Token::Comma) {
                if !self.match_tok_type(ID_TOKEN) {
                    return Err("Expected Identifier for Parameter Name")
                }

                let name = self.previous();

                if !self.match_tok(Token::Col) {
                    return Err("Expected Colon after Parameter Name")
                }

                let t = self.parse_type_declr()?;
                params.push((name, t));
            }
        }

        return Ok(Parameters { params })
    }
}