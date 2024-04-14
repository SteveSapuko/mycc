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
}