use std::array;

use super::*;

impl Parser {
    pub fn parse_declr(&mut self) -> Result<Box<dyn Stmt>, &'static str> {
        //VarDeclr
        if self.match_tok(Token::Key("let".to_string())) {
            if !self.match_tok_type(ID_TOKEN) {
                return Err("Expected Identifier for Variable Name")
            }
            let name = self.previous();

            if !self.match_tok(Token::Col) {
                return Err("Expected Colon After Variable Name")
            }

            let var_type = self.parse_type_declr()?;

            let value: Option<Box<dyn Expr>>;

            if self.match_tok(Token::Op("=".to_string())) {
                value = Some(self.parse_expr()?);
            } else {
                value = None
            }

            if !self.match_tok(Token::SemiCol) {
                return Err("Expected Semicolon after Variable Declaration")
            }

            return Ok(Box::new(VarDeclr {name, var_type, value}))
        }
        
        //StructDeclr
        if self.match_tok(Token::Key("struct".to_string())) {
            if !self.match_tok_type(ID_TOKEN) {
                return Err("Expected Identifier for Struct Name")
            }

            let name = self.previous();

            if !self.match_tok(Token::CurlyOpen) {
                return Err("Expected Opening Curly Bracket after Struct Name")
            }

            let fields = self.parse_parameters()?;

            if !self.match_tok(Token::CurlyClose) {
                return Err("Expected Closing Curly Bracket after Struct Fields")
            }

            return Ok(Box::new(StructDeclr {name, fields}))
        }

        //FnDeclr
        if self.match_tok(Token::Key("fn".to_string())) {
            if !self.match_tok_type(ID_TOKEN) {
                return Err("Expected Identifier for Variable Name")
            }

            let name = self.previous();

            if !self.match_tok(Token::ParenOpen) {
                return Err("Expected Opening Parentheses after Function Name")
            }

            let parameters = self.parse_parameters()?;

            if !self.match_tok(Token::ParenClose) {
                return Err("Expected Closing Parentheses after Function Parameters")
            }

            if !self.match_tok(Token::Arrow) {
                return Err("Expected Arrow to Denote Return Type")
            }

            let ret_type = self.parse_type_declr()?;

            let body = self.parse_block()?;

            return Ok(Box::new(FnDeclr {name, parameters, ret_type, body}))

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

            if !self.match_tok_type(ID_TOKEN) {
                return Err("Expected Identifier for Enum Variant")
            }
            variants.push(self.previous());

            while self.match_tok(Token::Comma) {
                if !self.match_tok_type(ID_TOKEN) {
                    return Err("Expected Identifier for Enum Variant")
                }
                variants.push(self.previous());
            }

            if !self.match_tok(Token::CurlyClose) {
                return Err("Expected Closing Curly Bracket after Enum Declaration")
            }

            return Ok(Box::new(EnumDeclr {name, variants}))

        }

        self.parse_stmt()
    }
    
    
    fn parse_stmt(&mut self) -> Result<Box<dyn Stmt>, &'static str> {
        //loop
        if self.match_tok(Token::Key("loop".to_string())) {
            let block = self.parse_block()?;
            return Ok(Box::new(LoopStmt {body: block}))
        }

        //if
        if self.match_tok(Token::Key("if".to_string())) {
            let condition = self.parse_expr()?;
            
            let true_branch = self.parse_block()?;

            let mut false_branch = None;

            if self.match_tok(Token::Key("else".to_string())) {
                false_branch = Some(self.parse_block()?);
            }

            return Ok(Box::new(IfStmt {condition, true_branch, false_branch}))
        }


        //while
        if self.match_tok(Token::Key("while".to_string())) {
            let condition = self.parse_expr()?;
            
            let body = self.parse_block()?;

            return Ok(Box::new(WhileStmt {condition, body}))
        }

        //break
        if self.match_tok(Token::Key("break".to_string())) {
            return Ok(Box::new(BreakStmt {key: self.previous()}))
        }

        //return
        if self.match_tok(Token::Key("return".to_string())) {
            let key = self.previous();
            let value = self.parse_expr()?;

            return Ok(Box::new(ReturnStmt {key, value}))
        }

        //ExprStmt if this point is reached
        let e = self.parse_expr()?;
        if !self.match_tok(Token::SemiCol) {
            return Err("Expected Semicolon after Expression Statement")
        }
        return Ok(Box::new(ExprStmt {e}))
    }

    fn parse_type_declr(&mut self) -> Result<TypeDeclr, &'static str> {
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

    fn parse_parameters(&mut self) -> Result<Parameters, &'static str> {
        let mut result: Vec<(Lexeme, TypeDeclr)> = vec![];
        
        if self.match_tok_type(ID_TOKEN) {
            let name = self.previous();

            if !self.match_tok(Token::Col) {
                return Err("Expected Colon After Parameter Name")
            }

            let param_type = self.parse_type_declr()?;

            result.push((name, param_type));

            while self.match_tok(Token::Comma) {
                if !self.match_tok_type(ID_TOKEN) {
                    return Err("Expected Identifier for Parameter Name")
                }

                let name = self.previous();

                if !self.match_tok(Token::Col) {
                    return Err("Expected Colon After Parameter Name")
                }

                let param_type = self.parse_type_declr()?;

                result.push((name, param_type));
            }
        }
        
        Ok(Parameters { params: result })
    }

    fn parse_block(&mut self) -> Result<BlockStmt, &'static str> {
        if !self.match_tok(Token::CurlyOpen) {
            return Err("Expected Opening Curly Brace for Block");
        }

        let mut body: Vec<Box<dyn Stmt>> = vec![];

        while !self.match_tok(Token::CurlyClose) {
            body.push(self.parse_declr()?);
        }

        Ok(BlockStmt {body})
    }
}