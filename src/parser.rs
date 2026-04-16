use crate::ast::{ Expr, Stmt, Type };
use crate::token::{ Literal, Token, TokenType };

#[derive(Debug, Clone)]
pub struct ParseError {
    pub token: Token,
    pub message: String,
    pub hints: Vec<String>,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    pub errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            errors: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    self.errors.push(err);
                    self.synchronize();
                }
            }
        }

        statements
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token_type == token_type
    }

    fn check_next(&self, token_type: TokenType) -> bool {
        if self.current + 1 >= self.tokens.len() {
            return false;
        }

        self.tokens[self.current + 1].token_type == token_type
    }

    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for &t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<&Token, ParseError> {
        if self.check(token_type) {
            return Ok(self.advance());
        }

        let mut err_token = self.peek().clone();
        let mut hints = Vec::new();

        if token_type == TokenType::Semicolon {
            err_token = self.previous().clone();
            hints.push("Maybe you forgot to use ';' after this instruction?".to_string());
        }

        Err(ParseError {
            token: err_token,
            message: message.to_string(),
            hints,
        })
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        let mut types = vec![self.parse_single_type()?];

        while self.match_token(&[TokenType::BitwiseOr]) {
            types.push(self.parse_single_type()?);
        }

        if types.len() == 1 {
            Ok(types.pop().unwrap())
        } else {
            Ok(Type::Union(types))
        }
    }

    fn parse_single_type(&mut self) -> Result<Type, ParseError> {
        if self.match_token(&[TokenType::Caret]) {
            let _is_const = self.match_token(&[TokenType::Const]);
            let inner_type = self.parse_single_type()?;
            return Ok(Type::Pointer(Box::new(inner_type)));
        }

        if self.match_token(&[TokenType::BitwiseAnd]) {
            let is_mut = if self.match_token(&[TokenType::Mut]) {
                true
            } else if self.check(TokenType::Identifier) && self.peek().lexeme == "mut" {
                self.advance();
                true
            } else {
                false
            };

            let inner_type = self.parse_single_type()?;
            if is_mut {
                return Ok(Type::MutReference(Box::new(inner_type)));
            } else {
                return Ok(Type::Reference(Box::new(inner_type)));
            }
        }

        if self.match_token(&[TokenType::LeftBracket]) {
            let inner_type = self.parse_type()?;
            self.consume(TokenType::Semicolon, "Expected ';' after array/slice type.")?;

            if self.match_token(&[TokenType::Question]) {
                self.consume(TokenType::RightBracket, "Expected ']' after '?'.")?;
                return Ok(Type::Slice(Box::new(inner_type)));
            } else {
                let size_token = self.consume(TokenType::Number, "Expected array size.")?;
                let size: usize = size_token.lexeme.parse().unwrap_or(0);
                self.consume(TokenType::RightBracket, "Expected ']'.")?;
                return Ok(Type::Array(size, Box::new(inner_type)));
            }
        }

        if self.match_token(&[TokenType::Fn]) {
            return self.parse_function_type();
        }

        if self.match_token(&[TokenType::LeftParen]) {
            return self.parse_tuple_type();
        }

        if self.check(TokenType::Identifier) {
            let name_token = self.advance().clone();
            let name = name_token.lexeme;

            if self.match_token(&[TokenType::Less]) {
                let mut type_args = Vec::new();

                if !self.check(TokenType::Greater) {
                    loop {
                        type_args.push(self.parse_type()?);

                        if !self.match_token(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }

                self.consume(TokenType::Greater, "Expected '>' after generic type arguments.")?;
                return Ok(Type::Generic(name, type_args));
            }

            return match name.as_str() {
                "void" => Ok(Type::Void),
                "bool" => Ok(Type::Bool),
                "string" => Ok(Type::String),

                "u8" => Ok(Type::U8),
                "u16" => Ok(Type::U16),
                "u32" => Ok(Type::U32),
                "u64" => Ok(Type::U64),

                "i8" => Ok(Type::I8),
                "i16" => Ok(Type::I16),
                "i32" => Ok(Type::I32),
                "i64" => Ok(Type::I64),

                "f16" => Ok(Type::F16),
                "f32" => Ok(Type::F32),
                "f64" => Ok(Type::F64),

                _ => Ok(Type::Custom(name)),
            };
        }

        Err(ParseError {
            token: self.peek().clone(),
            message: "Expected valid type annotation.".to_string(),
            hints: vec![],
        })
    }

    fn parse_function_type(&mut self) -> Result<Type, ParseError> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'fn' in type annotation.")?;

        let mut param_types = Vec::new();
        let mut is_variadic = false;

        if !self.check(TokenType::RightParen) {
            loop {
                if self.match_token(&[TokenType::Spread]) {
                    is_variadic = true;
                    break;
                }

                param_types.push(self.parse_type()?);
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expected ')' after parameter types.")?;

        self.consume(TokenType::Arrow, "Expected '->' to define function return type.")?;
        let return_type = Box::new(self.parse_type()?);

        Ok(Type::Function(param_types, return_type, is_variadic))
    }

    fn parse_tuple_type(&mut self) -> Result<Type, ParseError> {
        let mut element_types = Vec::new();

        if !self.check(TokenType::RightParen) {
            loop {
                element_types.push(self.parse_type()?);
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expected ')' after tuple types.")?;

        Ok(Type::Tuple(element_types))
    }

    fn parse_generic_params(&mut self) -> Result<Vec<crate::ast::GenericParam>, ParseError> {
        let mut params = Vec::new();
        if self.match_token(&[TokenType::Less]) {
            if !self.check(TokenType::Greater) {
                loop {
                    let name = self
                        .consume(TokenType::Identifier, "Expected generic parameter name (ex: T).")?
                        .clone();
                    let mut constraints = Vec::new();

                    if self.match_token(&[TokenType::Colon]) {
                        loop {
                            constraints.push(self.parse_type()?);
                            if !self.match_token(&[TokenType::Plus]) {
                                break;
                            }
                        }
                    }

                    params.push(crate::ast::GenericParam { name, constraints });

                    if !self.match_token(&[TokenType::Comma]) {
                        break;
                    }
                }
            }
            self.consume(TokenType::Greater, "Expected '>' after generic parameters.")?;
        }
        Ok(params)
    }

    // PARSER
    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(&[TokenType::Struct]) {
            return self.struct_declaration();
        }

        if self.match_token(&[TokenType::Fn]) {
            let is_async = self.match_token(&[TokenType::Async]);
            return self.function("function", is_async, false);
        }

        if self.match_token(&[TokenType::Pub]) {
            self.consume(TokenType::Fn, "Expected 'fn' after 'pub'.")?;
            let is_async = self.match_token(&[TokenType::Async]);
            return self.function("function", is_async, true);
        }

        if self.match_token(&[TokenType::Let]) {
            return self.destructuring_declaration();
        }

        if self.match_token(&[TokenType::Trait]) {
            return self.trait_declaration();
        }

        if self.match_token(&[TokenType::Enum]) {
            return self.enum_declaration();
        }

        if self.match_token(&[TokenType::Using]) {
            return self.using_declaration();
        }

        if self.match_token(&[TokenType::Impl]) {
            return self.impl_declaration();
        }

        if self.match_token(&[TokenType::Alias]) {
            return self.alias_declaration();
        }

        if self.match_token(&[TokenType::Extern]) {
            return self.extern_declaration();
        }

        self.statement()
    }

    fn struct_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier, "Expected struct name.")?.clone();
        let type_params = self.parse_generic_params()?;

        self.consume(TokenType::LeftBrace, "Expected '{' before struct type.")?;

        let mut fields = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let is_public = self.match_token(&[TokenType::Pub]);

            let field_name = self.consume(TokenType::Identifier, "Expected field name.")?.clone();

            self.consume(TokenType::Colon, "Expected ':' after field name.")?;
            let field_type = self.parse_type()?;

            fields.push((field_name, field_type, is_public));

            self.consume(TokenType::Comma, "Expected ',' after struct field declaration.")?;
        }
        self.consume(TokenType::RightBrace, "Expected '}' after struct body.")?;

        Ok(Stmt::Struct {
            name,
            type_params,
            fields,
        })
    }

    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        let keyword = self.previous().clone();

        if self.match_token(&[TokenType::Identifier]) {
            let key = self.previous().clone();
            let mut value = None;

            if self.match_token(&[TokenType::Comma]) {
                value = Some(self.consume(TokenType::Identifier, "Expected value name.")?.clone());
            }

            self.consume(TokenType::In, "Expected 'in' after variables.")?;
            let iterable = self.expression()?;
            self.consume(TokenType::LeftBrace, "Expected '{' before loop body.")?;

            let body = self.block()?;
            return Ok(Stmt::ForIn {
                keyword,
                key,
                value,
                iterable,
                body,
            });
        } else if self.match_token(&[TokenType::LeftParen]) {
            let mut initializer = None;
            if self.match_token(&[TokenType::Let]) {
                initializer = Some(Box::new(self.destructuring_declaration()?));
            } else if !self.match_token(&[TokenType::Semicolon]) {
                initializer = Some(Box::new(self.expression_statement()?));
            }

            let mut condition = None;
            if !self.check(TokenType::Semicolon) {
                condition = Some(self.expression()?);
            }
            self.consume(TokenType::Semicolon, "Expected ';' after loop condition.")?;

            let mut increment = None;
            if !self.check(TokenType::RightParen) {
                increment = Some(self.expression()?);
            }
            self.consume(TokenType::RightParen, "Expected ')' after 'for' clauses.")?;
            self.consume(TokenType::LeftBrace, "Expected '{' before 'for' body.")?;

            let body = self.block()?;
            return Ok(Stmt::For {
                keyword,
                initializer,
                condition,
                increment,
                body,
            });
        }

        Err(ParseError {
            token: self.peek().clone(),
            message: "Expected variable declaration or expression after 'for'.".to_string(),
            hints: vec![],
        })
    }

    fn using_declaration(&mut self) -> Result<Stmt, ParseError> {
        let keyword = self.previous().clone();
        let mut names = Vec::new();

        if self.match_token(&[TokenType::LeftBrace]) {
            loop {
                names.push(self.consume(TokenType::Identifier, "Expected identifier.")?.clone());
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
            self.consume(TokenType::RightBrace, "Expected '}' after identifiers.")?;
        } else {
            names.push(self.consume(TokenType::Identifier, "Expected identifier.")?.clone());
        }

        self.consume(TokenType::From, "Expected 'from' in 'using' declaration.")?;
        let source = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after 'using' declaration.")?;

        Ok(Stmt::Using {
            keyword,
            names,
            source,
        })
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(&[TokenType::If]) {
            return self.if_statement();
        }
        if self.match_token(&[TokenType::While]) {
            return self.while_statement();
        }
        if self.match_token(&[TokenType::For]) {
            return self.for_statement();
        }
        if self.match_token(&[TokenType::Return]) {
            return self.return_statement();
        }
        if self.match_token(&[TokenType::LeftBrace]) {
            return Ok(Stmt::Block(self.block()?));
        }

        if self.match_token(&[TokenType::Throw]) {
            return self.throw_statement();
        }
        if self.match_token(&[TokenType::Try]) {
            return self.try_statement();
        }

        self.expression_statement()
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expected '}' after block.")?;
        Ok(statements)
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        let if_token = self.previous().clone();
        let condition = self.expression()?;

        let then_branch = Box::new(self.statement()?);
        let mut else_branch = None;

        if self.match_token(&[TokenType::Else]) {
            else_branch = Some(Box::new(self.statement()?));
        }

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
            if_token,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        let while_token = self.previous().clone();
        let condition = self.expression()?;
        let body = Box::new(self.statement()?);

        Ok(Stmt::While {
            condition,
            body,
            while_token,
        })
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after expression.")?;
        Ok(Stmt::Expression(expr))
    }

    fn return_statement(&mut self) -> Result<Stmt, ParseError> {
        let keyword = self.previous().clone();
        let mut value = None;

        if !self.check(TokenType::Semicolon) {
            value = Some(self.expression()?);
        }

        self.consume(TokenType::Semicolon, "Expected ';' after return value.")?;
        Ok(Stmt::Return { keyword, value })
    }

    fn function(
        &mut self,
        kind: &str,
        is_async: bool,
        is_public: bool
    ) -> Result<Stmt, ParseError> {
        let name = self
            .consume(TokenType::Identifier, &format!("Expected {} name.", kind))?
            .clone();

        let type_params = self.parse_generic_params()?;

        self.consume(TokenType::LeftParen, &format!("Expected '(' after {} name.", kind))?;

        let mut params = Vec::new();

        if !self.check(TokenType::RightParen) {
            loop {
                let param_name = if self.match_token(&[TokenType::This]) {
                    self.previous().clone()
                } else {
                    self.consume(TokenType::Identifier, "Expected parameter name.")?.clone()
                };

                let mut type_annotation = None;
                if self.match_token(&[TokenType::Colon]) {
                    type_annotation = Some(self.parse_type()?);
                }

                let mut default_value = None;
                if self.match_token(&[TokenType::Equal]) {
                    default_value = Some(self.expression()?);
                }

                if params.len() >= 255 {
                    let err_token = self.peek().clone();
                    self.errors.push(ParseError {
                        token: err_token,
                        message: "Cannot have more than 255 parameters.".to_string(),
                        hints: vec![],
                    });
                }

                params.push(crate::ast::FunctionParam {
                    name: param_name,
                    type_annotation,
                    default_value,
                });

                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expected ')' after parameters.")?;

        let mut return_type = None;
        if self.match_token(&[TokenType::Arrow]) {
            return_type = Some(self.parse_type()?);
        }

        let mut body = None;
        let mut is_abstract = false;

        if self.match_token(&[TokenType::Semicolon]) {
            is_abstract = true;
        } else if self.match_token(&[TokenType::Arrow]) {
            let expr = self.expression()?;

            self.consume(
                TokenType::Semicolon,
                &format!("Expected ';' after {} (arrow function).", kind)
            )?;

            let ret_token = Token::synthetic(TokenType::Return, "return");
            body = Some(
                vec![Stmt::Return {
                    keyword: ret_token,
                    value: Some(expr),
                }]
            );
        } else {
            self.consume(TokenType::LeftBrace, &format!("Expected '{{' before {} body.", kind))?;
            body = Some(self.block()?);
        }

        Ok(Stmt::Function {
            name,
            type_params,
            params,
            return_type,
            body,
            is_abstract,
            is_async,
            is_public,
        })
    }

    fn destructuring_declaration(&mut self) -> Result<Stmt, ParseError> {
        let keyword = self.previous().clone();

        if self.match_token(&[TokenType::LeftBracket]) {
            let mut bindings = Vec::new();
            loop {
                bindings.push(
                    self
                        .consume(TokenType::Identifier, "Expected variable name in destructuring.")?
                        .clone()
                );
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
            self.consume(
                TokenType::RightBracket,
                "Expected ']' after array destructuring pattern."
            )?;
            self.consume(TokenType::Equal, "Expected '=' after pattern.")?;
            let initializer = self.expression()?;
            self.consume(TokenType::Semicolon, "Expected ';' after declaration.")?;
            return Ok(Stmt::ArrayDestructuring {
                keyword,
                bindings,
                initializer,
            });
        }

        self.var_declaration()
    }

    fn trait_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier, "Expected trait name.")?.clone();
        let type_params = self.parse_generic_params()?;
        let traits = self.with_clause()?;

        self.consume(TokenType::LeftBrace, "Expected '{' before trait body.")?;
        let mut methods = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let is_async = self.match_token(&[TokenType::Async]);
            self.consume(TokenType::Fn, "Expected 'fn' in trait.")?;

            methods.push(self.function("trait method", is_async, false)?);
        }
        self.consume(TokenType::RightBrace, "Expected '}' after trait body.")?;

        Ok(Stmt::Trait {
            name,
            type_params,
            traits,
            methods,
        })
    }

    fn enum_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier, "Expected enum name.")?.clone();
        let type_params = self.parse_generic_params()?;

        self.consume(TokenType::LeftBrace, "Expected '{' before enum body.")?;
        let mut cases = Vec::new();

        loop {
            let case_name = self.consume(TokenType::Identifier, "Expected variant name.")?.clone();
            let mut parameters = Vec::new();

            if self.match_token(&[TokenType::LeftParen]) {
                if !self.check(TokenType::RightParen) {
                    loop {
                        let param_name = self
                            .consume(TokenType::Identifier, "Expected parameter name.")?
                            .clone();

                        let mut param_type = None;
                        if self.match_token(&[TokenType::Colon]) {
                            param_type = Some(self.parse_type()?);
                        }

                        parameters.push((param_name, param_type));
                        if !self.match_token(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }
                self.consume(TokenType::RightParen, "Expected ')' after variant parameters.")?;
            }

            cases.push(crate::ast::EnumCase {
                name: case_name,
                parameters,
            });

            if
                !self.match_token(&[TokenType::Comma, TokenType::BitwiseOr]) ||
                self.check(TokenType::RightBrace)
            {
                break;
            }
        }

        self.consume(TokenType::RightBrace, "Expected '}' after enum variants.")?;
        Ok(Stmt::Enum {
            name,
            type_params,
            cases,
        })
    }

    fn impl_declaration(&mut self) -> Result<Stmt, ParseError> {
        let keyword = self.previous().clone();
        let type_params = self.parse_generic_params()?;

        let first_type = self.parse_type()?;
        let mut trait_name = None;
        let target_type;

        if self.match_token(&[TokenType::For]) {
            trait_name = Some(first_type);
            target_type = self.parse_type()?;
        } else {
            target_type = first_type;
        }

        self.consume(TokenType::LeftBrace, "Expected '{' before impl body.")?;
        let mut methods = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let is_public = self.match_token(&[TokenType::Pub]);

            self.consume(TokenType::Fn, "Expected 'fn' in impl.")?;
            let is_async = self.match_token(&[TokenType::Async]);

            methods.push(self.function("método", is_async, is_public)?);
        }
        self.consume(TokenType::RightBrace, "Expected '}' after impl.")?;

        Ok(Stmt::Impl {
            keyword,
            type_params,
            trait_name,
            target_type,
            methods,
        })
    }

    fn alias_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier, "Expected alias name.")?.clone();
        self.consume(TokenType::Equal, "Expected '=' after alias name.")?;
        let target = self.parse_type()?;
        self.consume(TokenType::Semicolon, "Expected ';' after alias declaration.")?;

        Ok(Stmt::Alias { name, target })
    }

    fn extern_declaration(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::Fn, "Expected 'fn' after 'extern'.")?;
        let name = self.consume(TokenType::Identifier, "Expected extern function name.")?.clone();

        self.consume(TokenType::LeftParen, "Expected '(' after extern function name.")?;
        let mut params = Vec::new();
        let mut is_variadic = false;

        if !self.check(TokenType::RightParen) {
            loop {
                if self.match_token(&[TokenType::Spread]) {
                    is_variadic = true;
                    break;
                }

                let mut param_name = self.peek().clone();
                let type_annotation;

                if self.check(TokenType::Identifier) && self.check_next(TokenType::Colon) {
                    param_name = self.advance().clone();
                    self.advance();
                    type_annotation = Some(self.parse_type()?);
                } else {
                    param_name.lexeme = "".to_string();
                    type_annotation = Some(self.parse_type()?);
                }

                params.push(crate::ast::FunctionParam {
                    name: param_name,
                    type_annotation,
                    default_value: None,
                });

                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expected ')'.")?;

        let mut return_type = None;
        if self.match_token(&[TokenType::Arrow]) {
            return_type = Some(self.parse_type()?);
        }
        self.consume(TokenType::Semicolon, "Expected ';' after extern function declaration.")?;

        Ok(Stmt::ExternFunction {
            name,
            params,
            return_type,
            is_variadic,
        })
    }

    fn throw_statement(&mut self) -> Result<Stmt, ParseError> {
        let keyword = self.previous().clone();
        let thrown = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after 'throw' value.")?;
        Ok(Stmt::Throw { keyword, thrown })
    }

    fn try_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftBrace, "Expected '{' after 'try'.")?;
        let try_body = self.block()?;

        self.consume(TokenType::Catch, "Expected 'catch' after 'try' block.")?;
        let exception = self.consume(TokenType::Identifier, "Expected exception name.")?.clone();

        let mut exception_type = None;
        if self.match_token(&[TokenType::Colon]) {
            exception_type = Some(self.parse_type()?);
        }

        self.consume(TokenType::LeftBrace, "Expected '{' before 'catch' body.")?;
        let catch_body = self.block()?;

        Ok(Stmt::TryCatch {
            try_body,
            catch_body,
            exception,
            exception_type,
        })
    }

    fn with_clause(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut traits = Vec::new();
        if self.match_token(&[TokenType::With]) {
            loop {
                let name = self.consume(TokenType::Identifier, "Expected identifier.")?.clone();
                traits.push(Expr::Variable(name));
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        Ok(traits)
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier, "Expected variable name.")?.clone();

        let mut type_annotation = None;
        if self.match_token(&[TokenType::Colon]) {
            type_annotation = Some(self.parse_type()?);
        }

        let mut initializer = None;
        if self.match_token(&[TokenType::Equal]) {
            initializer = Some(self.expression()?);
        }

        self.consume(TokenType::Semicolon, "Expected ';' after variable declaration.")?;

        Ok(Stmt::Var {
            name,
            type_annotation,
            initializer,
        })
    }

    pub fn expression(&mut self) -> Result<Expr, ParseError> {
        self.ternary()
    }

    fn ternary(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.null_coalescing()?;

        let true_token: Token;
        if self.match_token(&[TokenType::Question]) {
            true_token = self.previous().clone();
            let then_branch = Box::new(self.expression()?);
            self.consume(TokenType::Colon, "Expected ':' after ternary expression 'then' branch.")?;
            let else_branch = Box::new(self.ternary()?);

            expr = Expr::Ternary {
                condition: Box::new(expr),
                then_branch,
                else_branch,
                true_token,
            };
        }

        Ok(expr)
    }

    fn null_coalescing(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.assignment()?;

        while self.match_token(&[TokenType::QuestionQuestion]) {
            let operator = self.previous().clone();
            let right = Box::new(self.assignment()?);
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.or()?;

        if self.match_token(&[TokenType::Equal]) {
            let equals = self.previous().clone();
            let value = Box::new(self.assignment()?);

            return match expr {
                Expr::Variable(name) => Ok(Expr::Assign { name, value }),
                Expr::Get { object, name } =>
                    Ok(Expr::Set {
                        object,
                        name,
                        value,
                    }),

                Expr::SubscriptGet { indexee, bracket, index } =>
                    Ok(Expr::SubscriptSet {
                        indexee,
                        bracket,
                        index,
                        value,
                    }),

                Expr::Dereference { operator, operand } => {
                    return Ok(Expr::DereferenceSet {
                        operator,
                        ptr: operand,
                        value: Box::new(*value),
                    });
                }

                _ =>
                    Err(ParseError {
                        token: equals,
                        message: "Invalid assignment target.".to_string(),
                        hints: vec![],
                    }),
            };
        }

        if self.match_token(&[TokenType::PlusEqual, TokenType::MinusEqual]) {
            let operator_token = self.previous().clone();
            let value = self.assignment()?;

            let (op_type, op_str) = if operator_token.token_type == TokenType::PlusEqual {
                (TokenType::Plus, "+")
            } else {
                (TokenType::Minus, "-")
            };

            let synthetic_token = Token::synthetic(op_type, op_str);

            if let Expr::Variable(name) = expr.clone() {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(Expr::Binary {
                        left: Box::new(expr),
                        operator: synthetic_token,
                        right: Box::new(value),
                    }),
                });
            } else {
                return Err(ParseError {
                    token: operator_token,
                    message: "Invalid compound assignment target.".to_string(),
                    hints: vec![],
                });
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and()?;
        while self.match_token(&[TokenType::LogicalOr]) {
            let operator = self.previous().clone();
            let right = Box::new(self.and()?);
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right,
            };
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;
        while self.match_token(&[TokenType::LogicalAnd]) {
            let operator = self.previous().clone();
            let right = Box::new(self.equality()?);
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right,
            };
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;
        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = Box::new(self.comparison()?);
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;
        while
            self.match_token(
                &[
                    TokenType::Greater,
                    TokenType::GreaterEqual,
                    TokenType::Less,
                    TokenType::LessEqual,
                    TokenType::DotDot,
                ]
            )
        {
            let operator = self.previous().clone();
            let right = Box::new(self.term()?);
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;
        while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = Box::new(self.factor()?);
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.cast()?;

        while
            self.match_token(
                &[TokenType::Slash, TokenType::Star, TokenType::Exponentiation, TokenType::Percent]
            )
        {
            let operator = self.previous().clone();
            let right = Box::new(self.cast()?);
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }
        Ok(expr)
    }

    fn cast(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenType::As]) {
            let operator = self.previous().clone();

            let target_type = self.parse_single_type()?;

            expr = Expr::Cast {
                value: Box::new(expr),
                operator,
                target_type,
            };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if
            self.match_token(
                &[
                    TokenType::Bang,
                    TokenType::Minus,
                    TokenType::PlusPlus,
                    TokenType::MinusMinus,
                    TokenType::BitwiseAnd,
                    TokenType::Star,
                    TokenType::Caret,
                ]
            )
        {
            let operator = self.previous().clone();
            let right = Box::new(self.unary()?);

            return match operator.token_type {
                TokenType::BitwiseAnd =>
                    Ok(Expr::AddressOf {
                        operator,
                        operand: right,
                    }),
                TokenType::Caret =>
                    Ok(Expr::Dereference {
                        operator,
                        operand: right,
                    }),

                _ => Ok(Expr::Unary { operator, right }),
            };
        }
        self.call()
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&[TokenType::Dot]) {
                let name = if self.match_token(&[TokenType::Number]) {
                    self.previous().clone()
                } else {
                    self.consume(
                        TokenType::Identifier,
                        "Expected property name after '.'."
                    )?.clone()
                };
                expr = Expr::Get {
                    object: Box::new(expr),
                    name,
                };
            } else if self.match_token(&[TokenType::LeftBracket]) {
                let bracket = self.previous().clone();
                let index = Box::new(self.expression()?);
                self.consume(TokenType::RightBracket, "Expected ']' after index.")?;
                expr = Expr::SubscriptGet {
                    indexee: Box::new(expr),
                    bracket,
                    index,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let mut arguments = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    let err_token = self.peek().clone();
                    self.errors.push(ParseError {
                        token: err_token,
                        message: "Cannot have more than 255 arguments.".to_string(),
                        hints: vec![],
                    });
                }
                arguments.push(self.expression()?);
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren = self.consume(TokenType::RightParen, "Expected ')' after arguments.")?.clone();
        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
    }

    fn parse_generic_type_args(&mut self) -> Result<Vec<Type>, ParseError> {
        self.consume(TokenType::Less, "Expected '<' before generic type arguments.")?;

        let mut type_args = Vec::new();
        if !self.check(TokenType::Greater) {
            loop {
                type_args.push(self.parse_type()?);
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::Greater, "Expected '>' after generic type arguments.")?;
        Ok(type_args)
    }

    fn is_next_struct_init_generic(&self) -> bool {
        if !self.check(TokenType::Less) {
            return false;
        }

        let mut depth = 0;
        let mut idx = self.current;

        while idx < self.tokens.len() {
            match self.tokens[idx].token_type {
                TokenType::Less => {
                    depth += 1;
                }
                TokenType::Greater => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                }
                TokenType::LeftBrace => {
                    return depth == 0;
                }
                TokenType::LeftParen => {
                    return false;
                }
                TokenType::RightBrace => {
                    return false;
                }
                _ => {}
            }
            idx += 1;
        }

        false
    }

    fn struct_init(&mut self, class_name: Token, type_args: Vec<Type>) -> Result<Expr, ParseError> {
        let mut properties = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let name = self
                .consume(TokenType::Identifier, "Expected field name in struct initializer.")?
                .clone();

            // shorthand: `foo { x }` is equivalent to `foo { x: x }`
            let value = if self.match_token(&[TokenType::Colon]) {
                self.expression()?
            } else {
                Expr::Variable(name.clone())
            };

            properties.push((name, value));

            if !self.check(TokenType::RightBrace) {
                self.consume(TokenType::Comma, "Expected ',' after field initializer.")?;
            }
        }

        let brace = self
            .consume(TokenType::RightBrace, "Expected '}' after struct initializer.")?
            .clone();
        Ok(Expr::StructInit { class_name, type_args, properties, brace })
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(&[TokenType::False]) {
            return Ok(Expr::Literal(Literal::Bool(false)));
        }

        if self.match_token(&[TokenType::True]) {
            return Ok(Expr::Literal(Literal::Bool(true)));
        }

        if self.match_token(&[TokenType::Null]) {
            return Ok(Expr::Literal(Literal::None));
        }

        if self.match_token(&[TokenType::Number, TokenType::String]) {
            return Ok(Expr::Literal(self.previous().literal.clone()));
        }

        if self.match_token(&[TokenType::This]) {
            return Ok(Expr::This(self.previous().clone()));
        }

        if self.match_token(&[TokenType::Identifier]) {
            let class_name = self.previous().clone();
            let mut type_args = Vec::new();

            if self.is_next_struct_init_generic() {
                type_args = self.parse_generic_type_args()?;
            }

            if self.match_token(&[TokenType::LeftBrace]) {
                return self.struct_init(class_name, type_args);
            }

            return Ok(Expr::Variable(class_name));
        }

        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            if self.match_token(&[TokenType::Comma]) {
                let token = self.previous().clone();
                let mut elements = vec![expr];
                loop {
                    elements.push(self.expression()?);
                    if !self.match_token(&[TokenType::Comma]) {
                        break;
                    }
                }
                self.consume(TokenType::RightParen, "Expected ')' after tuple.")?;
                return Ok(Expr::TupleLiteral { elements, token });
            } else {
                self.consume(TokenType::RightParen, "Expected ')' after expression.")?;
                return Ok(Expr::Grouping(Box::new(expr)));
            }
        }

        if self.match_token(&[TokenType::LeftBracket]) {
            return self.array();
        }

        if self.match_token(&[TokenType::Async]) {
            self.consume(TokenType::Fn, "Expected 'fn' after 'async'.")?;
            return self.lambda(true);
        }

        if self.match_token(&[TokenType::Fn]) {
            return self.lambda(false);
        }

        if self.match_token(&[TokenType::Lazy]) {
            if self.match_token(&[TokenType::LeftBrace]) {
                let statements = self.block()?;
                return Ok(Expr::Lazy {
                    expr: None,
                    statements: Some(statements),
                });
            }

            let expr = self.expression()?;
            return Ok(Expr::Lazy {
                expr: Some(Box::new(expr)),
                statements: None,
            });
        }

        if self.match_token(&[TokenType::Await]) {
            let keyword = self.previous().clone();
            let value = Box::new(self.expression()?);
            return Ok(Expr::Await { keyword, value });
        }

        if self.match_token(&[TokenType::Match]) {
            return self.match_expression();
        }

        Err(ParseError {
            token: self.peek().clone(),
            message: "Expected an expression.".to_string(),
            hints: vec![],
        })
    }

    fn array(&mut self) -> Result<Expr, ParseError> {
        let mut elements = Vec::new();
        if !self.check(TokenType::RightBracket) {
            loop {
                if self.match_token(&[TokenType::Spread]) {
                    let op = self.previous().clone();
                    elements.push(Expr::Spread {
                        operator: op,
                        right: Box::new(self.expression()?),
                    });
                } else {
                    elements.push(self.expression()?);
                }
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        let bracket = self
            .consume(TokenType::RightBracket, "Expected ']' after array elements.")?
            .clone();
        Ok(Expr::Array { bracket, elements })
    }

    fn lambda(&mut self, is_async: bool) -> Result<Expr, ParseError> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'fn' in lambda.")?;

        let mut params = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                let name = self.consume(TokenType::Identifier, "Expected parameter name.")?.clone();

                let mut type_annotation = None;
                if self.match_token(&[TokenType::Colon]) {
                    type_annotation = Some(self.parse_type()?);
                }

                params.push(crate::ast::FunctionParam {
                    name,
                    type_annotation,
                    default_value: None,
                });

                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expected ')' after lambda parameters.")?;

        let mut return_type = None;
        if self.match_token(&[TokenType::Arrow]) {
            return_type = Some(self.parse_type()?);
        }

        let mut body = Vec::new();
        if self.match_token(&[TokenType::Arrow]) {
            let expr = self.expression()?;
            let ret_token = Token::synthetic(TokenType::Return, "return");
            body.push(Stmt::Return {
                keyword: ret_token,
                value: Some(expr),
            });
        } else {
            self.consume(TokenType::LeftBrace, "Expected '{' before lambda body.")?;
            body = self.block()?;
        }

        Ok(Expr::Lambda {
            params,
            return_type,
            body,
            is_async,
        })
    }

    fn match_expression(&mut self) -> Result<Expr, ParseError> {
        let keyword = self.previous().clone();
        let value = Box::new(self.expression()?);
        self.consume(TokenType::LeftBrace, "Expected '{' before match cases.")?;

        let mut cases = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            cases.push(self.match_case()?);
        }

        self.consume(TokenType::RightBrace, "Expected '}' after match cases.")?;
        Ok(Expr::Match {
            keyword,
            value,
            cases,
        })
    }

    fn match_case(&mut self) -> Result<crate::ast::MatchCase, ParseError> {
        let pattern = self.pattern()?;
        let mut guard = None;

        if self.match_token(&[TokenType::If]) {
            guard = Some(self.expression()?);
        }

        self.consume(TokenType::Arrow, "Expected '->' after match patterns.")?;

        let body;
        if self.match_token(&[TokenType::LeftBrace]) {
            body = self.block()?;
        } else {
            let expr = self.expression()?;
            self.consume(TokenType::Semicolon, "Expected ';' after match pattern.")?;
            let ret_token = Token::synthetic(TokenType::Return, "return");
            body = vec![Stmt::Return {
                keyword: ret_token,
                value: Some(expr),
            }];
        }

        Ok(crate::ast::MatchCase {
            pattern,
            guard,
            body,
        })
    }

    fn pattern(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(&[TokenType::Star]) {
            return Ok(Expr::WildcardPattern(self.previous().clone()));
        }

        if self.match_token(&[TokenType::LeftBracket]) {
            return self.list_pattern();
        }

        if self.match_token(&[TokenType::LeftBrace]) {
            return self.object_pattern();
        }

        if self.check(TokenType::Identifier) && self.check_next(TokenType::LeftParen) {
            let case_name = self.advance().clone();
            self.consume(TokenType::LeftParen, "Expected '('.")?;

            let mut bindings = Vec::new();
            if !self.check(TokenType::RightParen) {
                loop {
                    bindings.push(
                        self.consume(TokenType::Identifier, "Expected variable name.")?.clone()
                    );
                    if !self.match_token(&[TokenType::Comma]) {
                        break;
                    }
                }
            }
            self.consume(TokenType::RightParen, "Expected ')'.")?;
            return Ok(Expr::UnionPattern {
                case_name,
                bindings,
            });
        }

        if self.match_token(&[TokenType::Identifier]) {
            return Ok(Expr::Variable(self.previous().clone()));
        }

        self.primary()
    }

    fn list_pattern(&mut self) -> Result<Expr, ParseError> {
        let mut elements = Vec::new();
        let mut rest = None;

        while !self.check(TokenType::RightBracket) && !self.is_at_end() {
            if self.match_token(&[TokenType::Spread]) {
                rest = Some(Box::new(self.expression()?));
                break;
            }

            elements.push(self.expression()?);

            if !self.check(TokenType::RightBracket) {
                self.consume(TokenType::Comma, "Expected ',' after pattern element.")?;
            }
        }

        self.consume(TokenType::RightBracket, "Expected ']' after list pattern.")?;
        Ok(Expr::ListPattern { elements, rest })
    }

    fn object_pattern(&mut self) -> Result<Expr, ParseError> {
        let mut properties = Vec::new();
        let mut rest = None;

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            if self.match_token(&[TokenType::Spread]) {
                rest = Some(Box::new(self.expression()?));
                break;
            }

            let name = self.consume(TokenType::Identifier, "Expected property name.")?.clone();
            self.consume(TokenType::Colon, "Expected ':' after property name.")?;

            let pattern = self.expression()?;
            properties.push((name, pattern));

            if !self.check(TokenType::RightBrace) {
                self.consume(TokenType::Comma, "Expected ',' after property.")?;
            }
        }

        self.consume(TokenType::RightBrace, "Expected '}' after object pattern.")?;
        Ok(Expr::ObjectPattern { properties, rest })
    }

    fn synchronize(&mut self) {
        match self.peek().token_type {
            | TokenType::Struct
            | TokenType::Fn
            | TokenType::Let
            | TokenType::For
            | TokenType::If
            | TokenType::While
            | TokenType::Return
            | TokenType::Trait
            | TokenType::Enum
            | TokenType::Impl
            | TokenType::Pub
            | TokenType::Eof => {}
            _ => {
                self.advance();
            }
        }

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                | TokenType::Struct
                | TokenType::Fn
                | TokenType::Let
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Return
                | TokenType::Trait
                | TokenType::Enum
                | TokenType::Impl
                | TokenType::Pub => {
                    return;
                }
                _ => {}
            }
            self.advance();
        }
    }
}
