//! https://github.com/estree/estree
//! https://astexplorer.net/

use crate::renderer::js::token::JsLexer;
use crate::renderer::js::token::Token;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::iter::Peekable;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    body: Vec<Rc<Node>>,
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

impl Program {
    pub fn new() -> Self {
        Self { body: Vec::new() }
    }

    pub fn set_body(&mut self, body: Vec<Rc<Node>>) {
        self.body = body;
    }

    pub fn body(&self) -> &Vec<Rc<Node>> {
        &self.body
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// https://github.com/estree/estree/blob/master/es5.md#expressionstatement
    ExpressionStatement(Option<Rc<Node>>),
    /// https://github.com/estree/estree/blob/master/es5.md#blockstatement
    BlockStatement { body: Vec<Option<Rc<Node>>> },
    /// https://github.com/estree/estree/blob/master/es5.md#returnstatement
    ReturnStatement { argument: Option<Rc<Node>> },
    /// https://github.com/estree/estree/blob/master/es5.md#functions
    /// https://github.com/estree/estree/blob/master/es5.md#functiondeclaration
    FunctionDeclaration {
        id: Option<Rc<Node>>,
        params: Vec<Option<Rc<Node>>>,
        body: Option<Rc<Node>>,
    },
    /// https://github.com/estree/estree/blob/master/es5.md#variabledeclaration
    VariableDeclaration { declarations: Vec<Option<Rc<Node>>> },
    /// https://github.com/estree/estree/blob/master/es5.md#variabledeclarator
    VariableDeclarator {
        id: Option<Rc<Node>>,
        init: Option<Rc<Node>>,
    },
    /// https://github.com/estree/estree/blob/master/es5.md#binaryexpression
    BinaryExpression {
        operator: char,
        left: Option<Rc<Node>>,
        right: Option<Rc<Node>>,
    },
    /// https://github.com/estree/estree/blob/master/es5.md#assignmentexpression
    AssignmentExpression {
        operator: char,
        left: Option<Rc<Node>>,
        right: Option<Rc<Node>>,
    },
    /// https://github.com/estree/estree/blob/master/es5.md#memberexpression
    MemberExpression {
        object: Option<Rc<Node>>,
        property: Option<Rc<Node>>,
    },
    /// https://github.com/estree/estree/blob/master/es5.md#callexpression
    CallExpression {
        callee: Option<Rc<Node>>,
        arguments: Vec<Option<Rc<Node>>>,
    },
    /// https://github.com/estree/estree/blob/master/es5.md#identifier
    /// https://262.ecma-international.org/12.0/#prod-Identifier
    Identifier(String),
    /// https://github.com/estree/estree/blob/master/es5.md#literal
    /// https://262.ecma-international.org/12.0/#prod-NumericLiteral
    NumericLiteral(u64),
    /// https://github.com/estree/estree/blob/master/es5.md#literal
    /// https://262.ecma-international.org/12.0/#prod-StringLiteral
    StringLiteral(String),
}

impl Node {
    pub fn new_binary_expression(
        operator: char,
        left: Option<Rc<Node>>,
        right: Option<Rc<Node>>,
    ) -> Option<Rc<Self>> {
        Some(Rc::new(Node::BinaryExpression {
            operator,
            left,
            right,
        }))
    }

    pub fn new_assignment_expression(
        operator: char,
        left: Option<Rc<Node>>,
        right: Option<Rc<Node>>,
    ) -> Option<Rc<Self>> {
        Some(Rc::new(Node::AssignmentExpression {
            operator,
            left,
            right,
        }))
    }

    pub fn new_expression_statement(expression: Option<Rc<Self>>) -> Option<Rc<Self>> {
        Some(Rc::new(Node::ExpressionStatement(expression)))
    }

    pub fn new_block_statement(body: Vec<Option<Rc<Self>>>) -> Option<Rc<Self>> {
        Some(Rc::new(Node::BlockStatement { body }))
    }

    pub fn new_return_statement(argument: Option<Rc<Self>>) -> Option<Rc<Self>> {
        Some(Rc::new(Node::ReturnStatement { argument }))
    }

    pub fn new_function_declaration(
        id: Option<Rc<Self>>,
        params: Vec<Option<Rc<Self>>>,
        body: Option<Rc<Self>>,
    ) -> Option<Rc<Self>> {
        Some(Rc::new(Node::FunctionDeclaration { id, params, body }))
    }

    pub fn new_variable_declarator(
        id: Option<Rc<Self>>,
        init: Option<Rc<Self>>,
    ) -> Option<Rc<Self>> {
        Some(Rc::new(Node::VariableDeclarator { id, init }))
    }

    pub fn new_variable_declaration(declarations: Vec<Option<Rc<Self>>>) -> Option<Rc<Self>> {
        Some(Rc::new(Node::VariableDeclaration { declarations }))
    }

    pub fn new_member_expression(
        object: Option<Rc<Self>>,
        property: Option<Rc<Self>>,
    ) -> Option<Rc<Self>> {
        Some(Rc::new(Node::MemberExpression { object, property }))
    }

    pub fn new_call_expression(
        callee: Option<Rc<Self>>,
        arguments: Vec<Option<Rc<Self>>>,
    ) -> Option<Rc<Self>> {
        Some(Rc::new(Node::CallExpression { callee, arguments }))
    }

    pub fn new_identifier(name: String) -> Option<Rc<Self>> {
        Some(Rc::new(Node::Identifier(name)))
    }

    pub fn new_numeric_literal(value: u64) -> Option<Rc<Self>> {
        Some(Rc::new(Node::NumericLiteral(value)))
    }

    pub fn new_string_literal(value: String) -> Option<Rc<Self>> {
        Some(Rc::new(Node::StringLiteral(value)))
    }
}

#[derive(Debug)]
pub struct JsParser {
    t: Peekable<JsLexer>,
}

impl JsParser {
    pub fn new(t: JsLexer) -> Self {
        Self { t: t.peekable() }
    }

    /// Literal ::= ( <DECIMAL_LITERAL> | <HEX_INTEGER_LITERAL> | <STRING_LITERAL> |
    ///               <BOOLEAN_LITERAL> | <NULL_LITERAL> | <REGULAR_EXPRESSION_LITERAL> )
    ///
    /// PrimaryExpression ::= "this"
    ///                     | ObjectLiteral
    ///                     | ( "(" Expression ")" )
    ///                     | Identifier
    ///                     | ArrayLiteral
    ///                     | Literal
    fn primary_expression(&mut self) -> Option<Rc<Node>> {
        let t = match self.t.next() {
            Some(token) => token,
            None => return None,
        };

        match t {
            Token::Identifier(value) => Node::new_identifier(value),
            // Literal
            Token::Number(value) => Node::new_numeric_literal(value),
            Token::StringLiteral(value) => Node::new_string_literal(value),
            _ => None,
        }
    }

    /// MemberExpressionPart ::= ( "[" Expression "]" ) | ( "." Identifier )
    ///
    /// MemberExpression ::= ( ( FunctionExpression | PrimaryExpression ) ( MemberExpressionPart)* )
    ///                    | AllocationExpression
    fn member_expression(&mut self) -> Option<Rc<Node>> {
        let expr = self.primary_expression();

        let t = match self.t.peek() {
            Some(token) => token,
            None => return expr,
        };

        match t {
            Token::Punctuator(c) => {
                if c == &'.' {
                    // consume '.'
                    assert!(self.t.next().is_some());
                    return Node::new_member_expression(expr, self.identifier());
                }

                expr
            }
            _ => expr,
        }
    }

    /// MemberExpression ::= ( ( FunctionExpression | PrimaryExpression ) ( MemberExpressionPart)* )
    ///                    | AllocationExpression
    ///
    /// Arguments ::= "(" ( ArgumentList )? ")"
    /// CallExpression ::= MemberExpression Arguments ( CallExpressionPart )*
    ///
    /// LeftHandSideExpression ::= CallExpression | MemberExpression
    fn left_hand_side_expression(&mut self) -> Option<Rc<Node>> {
        let expr = self.member_expression();

        let t = match self.t.peek() {
            Some(token) => token,
            None => return expr,
        };

        match t {
            Token::Punctuator(c) => {
                if c == &'(' {
                    // consume '('
                    assert!(self.t.next().is_some());
                    return Node::new_call_expression(expr, self.arguments());
                }

                // return MemberExpression
                expr
            }
            _ => expr,
        }
    }

    /// PostfixExpression ::= LeftHandSideExpression ( PostfixOperator )?
    /// UnaryExpression ::= ( PostfixExpression | ( UnaryOperator UnaryExpression )+ )
    /// MultiplicativeExpression ::= UnaryExpression ( MultiplicativeOperator UnaryExpression )*
    ///
    /// AdditiveExpression ::= MultiplicativeExpression ( AdditiveOperator MultiplicativeExpression )*
    fn additive_expression(&mut self) -> Option<Rc<Node>> {
        let left = self.left_hand_side_expression();

        let t = match self.t.peek() {
            Some(token) => token.clone(),
            None => return left,
        };

        // TODO: support MultiplicativeExpression ('*' and '/')
        match t {
            Token::Punctuator(c) => match c {
                // AdditiveExpression
                '+' | '-' => {
                    // consume '+' or '-'
                    assert!(self.t.next().is_some());
                    Node::new_binary_expression(c, left, self.assignment_expression())
                }
                /*
                // end of expression
                ';' => {
                    // consume ';'
                    assert!(self.t.next().is_some());
                    left
                }
                // end of expression wihtout consuming next token
                ',' | ')' => left,
                */
                _ => left,
            },
            _ => left,
        }
    }

    /// ShiftExpression ::= AdditiveExpression ( ShiftOperator AdditiveExpression )*
    /// RelationalExpression ::= ShiftExpression ( RelationalOperator ShiftExpression )*
    /// EqualityExpression  ::= RelationalExpression ( EqualityOperator RelationalExpression )*
    /// BitwiseANDExpression ::= EqualityExpression ( BitwiseANDOperator EqualityExpression )*
    /// BitwiseXORExpression ::= BitwiseANDExpression ( BitwiseXOROperator BitwiseANDExpression )*
    /// BitwiseORExpression ::= BitwiseXORExpression ( BitwiseOROperator BitwiseXORExpression )*
    /// LogicalANDExpression ::= BitwiseORExpression ( LogicalANDOperator BitwiseORExpression )*
    /// LogicalORExpression ::= LogicalANDExpression ( LogicalOROperator LogicalANDExpression )*
    /// ConditionalExpression ::= LogicalORExpression ( "?" AssignmentExpression ":" AssignmentExpression )?
    /// ConditionalExpression ::= LogicalORExpression ( "?" AssignmentExpression ":" AssignmentExpression )?
    ///
    /// AssignmentExpression ::= ( LeftHandSideExpression AssignmentOperator AssignmentExpression
    ///                          | ConditionalExpression )
    fn assignment_expression(&mut self) -> Option<Rc<Node>> {
        let expr = self.additive_expression();

        let t = match self.t.peek() {
            Some(token) => token,
            None => return expr,
        };

        match t {
            Token::Punctuator('=') => {
                // consume '='
                assert!(self.t.next().is_some());
                Node::new_assignment_expression('=', expr, self.assignment_expression())
            }
            _ => expr,
        }
    }

    /// Identifier ::= <IDENTIFIER_NAME>
    fn identifier(&mut self) -> Option<Rc<Node>> {
        let t = match self.t.next() {
            Some(token) => token,
            None => return None,
        };

        match t {
            Token::Identifier(name) => Node::new_identifier(name),
            _ => None,
        }
    }

    /// Initialiser ::= "=" AssignmentExpression
    fn initialiser(&mut self) -> Option<Rc<Node>> {
        let t = match self.t.next() {
            Some(token) => token,
            None => return None,
        };

        match t {
            Token::Punctuator('=') => self.assignment_expression(),
            _ => None,
        }
    }

    /// VariableDeclarationList ::= VariableDeclaration ( "," VariableDeclaration )*
    /// VariableDeclaration ::= Identifier ( Initialiser )?
    fn variable_declaration(&mut self) -> Option<Rc<Node>> {
        let ident = self.identifier();

        // TODO: support multiple declarator
        let declarator = Node::new_variable_declarator(ident, self.initialiser());

        let declarations = vec![declarator];

        Node::new_variable_declaration(declarations)
    }

    /// https://262.ecma-international.org/12.0/#prod-Statement
    ///
    /// AssignmentExpression ::= ( LeftHandSideExpression AssignmentOperator AssignmentExpression
    ///                          | ConditionalExpression )
    ///
    /// Expression ::= AssignmentExpression ( "," AssignmentExpression )*
    ///
    /// VariableStatement ::= "var" VariableDeclarationList ( ";" )?
    /// ExpressionStatement ::= Expression ( ";" )?
    /// ReturnStatement ::= "return" ( Expression )? ( ";" )?
    ///
    /// Statement ::= ExpressionStatement | VariableStatement | ReturnStatement
    fn statement(&mut self) -> Option<Rc<Node>> {
        let t = match self.t.peek() {
            Some(t) => t,
            None => return None,
        };

        let node = match t {
            Token::Keyword(keyword) => {
                if keyword == "var" {
                    // consume "var"
                    assert!(self.t.next().is_some());

                    self.variable_declaration()
                } else if keyword == "return" {
                    // consume "return"
                    assert!(self.t.next().is_some());

                    Node::new_return_statement(self.assignment_expression())
                } else {
                    None
                }
            }
            _ => Node::new_expression_statement(self.assignment_expression()),
        };

        if let Some(Token::Punctuator(c)) = self.t.peek() {
            // consume ';'
            if c == &';' {
                assert!(self.t.next().is_some());
            }
        }

        node
    }

    /// FunctionBody ::= "{" ( SourceElements )? "}"
    fn function_body(&mut self) -> Option<Rc<Node>> {
        // consume '{'
        match self.t.next() {
            Some(t) => match t {
                Token::Punctuator(c) => assert!(c == '{'),
                _ => unimplemented!("function should have open curly blacket but got {:?}", t),
            },
            None => unimplemented!("function should have open curly blacket but got None"),
        }

        let mut body = Vec::new();
        loop {
            // loop until hits '}'
            if let Some(Token::Punctuator(c)) = self.t.peek() {
                if c == &'}' {
                    // consume '}'
                    assert!(self.t.next().is_some());
                    return Node::new_block_statement(body);
                }
            }

            body.push(self.source_element());
        }
    }

    /// ArgumentList ::= AssignmentExpression ( "," AssignmentExpression )*
    ///
    /// Arguments ::= "(" ( ArgumentList )? ")"
    fn arguments(&mut self) -> Vec<Option<Rc<Node>>> {
        let mut arguments = Vec::new();

        loop {
            // push identifier to `arguments` until hits ')'
            match self.t.peek() {
                Some(t) => match t {
                    Token::Punctuator(c) => {
                        if c == &')' {
                            // consume ')'
                            assert!(self.t.next().is_some());
                            return arguments;
                        }
                        if c == &',' {
                            // consume ','
                            assert!(self.t.next().is_some());
                        }
                    }
                    _ => arguments.push(self.assignment_expression()),
                },
                None => return arguments,
            }
        }
    }

    /// FormalParameterList ::= Identifier ( "," Identifier )*
    fn parameter_list(&mut self) -> Vec<Option<Rc<Node>>> {
        let mut params = Vec::new();

        // consume '('
        match self.t.next() {
            Some(t) => match t {
                Token::Punctuator(c) => assert!(c == '(', "expect ( but got {:?}", c),
                _ => unimplemented!("function should have `(` but got {:?}", t),
            },
            None => unimplemented!("function should have `(` but got None"),
        }

        loop {
            // push identifier to `params` until hits ')'
            match self.t.peek() {
                Some(t) => match t {
                    Token::Punctuator(c) => {
                        if c == &')' {
                            // consume ')'
                            assert!(self.t.next().is_some());
                            return params;
                        }
                        if c == &',' {
                            // consume ','
                            assert!(self.t.next().is_some());
                        }
                    }
                    _ => {
                        params.push(self.identifier());
                    }
                },
                None => return params,
            }
        }
    }

    /// FunctionDeclaration ::= "function" Identifier ( "(" ( FormalParameterList )? ")" ) FunctionBody
    fn function_declaration(&mut self) -> Option<Rc<Node>> {
        let id = self.identifier();
        let params = self.parameter_list();
        Node::new_function_declaration(id, params, self.function_body())
    }

    /// SourceElement ::= FunctionDeclaration | Statement
    fn source_element(&mut self) -> Option<Rc<Node>> {
        let t = match self.t.peek() {
            Some(t) => t,
            None => return None,
        };

        match t {
            Token::Keyword(keyword) => {
                if keyword == "function" {
                    // consume "function"
                    assert!(self.t.next().is_some());
                    self.function_declaration()
                } else {
                    self.statement()
                }
            }
            _ => self.statement(),
        }
    }

    /// SourceElements ::= ( SourceElement )+
    ///
    /// Program ::= ( SourceElements )? <EOF>
    pub fn parse_ast(&mut self) -> Program {
        let mut program = Program::new();

        // interface Program <: Node {
        //   type: "Program";
        //   body: [ Directive | Statement ];
        // }
        let mut body = Vec::new();

        loop {
            let node = self.source_element();

            match node {
                Some(n) => body.push(n),
                None => {
                    program.set_body(body);
                    return program;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alloc::string::ToString;

    #[test]
    fn test_empty() {
        let input = "".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let expected = Program::new();
        assert_eq!(expected, parser.parse_ast());
    }

    #[test]
    fn test_num() {
        let input = "42".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let mut expected = Program::new();
        let mut body = Vec::new();
        body.push(Rc::new(Node::ExpressionStatement(Some(Rc::new(
            Node::NumericLiteral(42),
        )))));
        expected.set_body(body);
        assert_eq!(expected, parser.parse_ast());
    }

    #[test]
    fn test_add_nums() {
        let input = "1 + 2".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let mut expected = Program::new();
        let mut body = Vec::new();
        body.push(Rc::new(Node::ExpressionStatement(Some(Rc::new(
            Node::BinaryExpression {
                operator: '+',
                left: Some(Rc::new(Node::NumericLiteral(1))),
                right: Some(Rc::new(Node::NumericLiteral(2))),
            },
        )))));
        expected.set_body(body);
        assert_eq!(expected, parser.parse_ast());
    }

    #[test]
    fn test_assign_variable() {
        let input = "var foo=42;".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let mut expected = Program::new();
        let mut body = Vec::new();
        body.push(Rc::new(Node::VariableDeclaration {
            declarations: [Some(Rc::new(Node::VariableDeclarator {
                id: Some(Rc::new(Node::Identifier("foo".to_string()))),
                init: Some(Rc::new(Node::NumericLiteral(42))),
            }))]
            .to_vec(),
        }));
        expected.set_body(body);
        assert_eq!(expected, parser.parse_ast());
    }

    #[test]
    fn test_add_variable_and_num() {
        let input = "var foo=42; var result=foo+1;".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let mut expected = Program::new();
        let mut body = Vec::new();
        body.push(Rc::new(Node::VariableDeclaration {
            declarations: [Some(Rc::new(Node::VariableDeclarator {
                id: Some(Rc::new(Node::Identifier("foo".to_string()))),
                init: Some(Rc::new(Node::NumericLiteral(42))),
            }))]
            .to_vec(),
        }));
        body.push(Rc::new(Node::VariableDeclaration {
            declarations: [Some(Rc::new(Node::VariableDeclarator {
                id: Some(Rc::new(Node::Identifier("result".to_string()))),
                init: Some(Rc::new(Node::BinaryExpression {
                    operator: '+',
                    left: Some(Rc::new(Node::Identifier("foo".to_string()))),
                    right: Some(Rc::new(Node::NumericLiteral(1))),
                })),
            }))]
            .to_vec(),
        }));
        expected.set_body(body);
        assert_eq!(expected, parser.parse_ast());
    }

    #[test]
    fn test_reassign_variable() {
        let input = "var foo=42; foo=1;".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let mut expected = Program::new();
        let mut body = Vec::new();
        body.push(Rc::new(Node::VariableDeclaration {
            declarations: [Some(Rc::new(Node::VariableDeclarator {
                id: Some(Rc::new(Node::Identifier("foo".to_string()))),
                init: Some(Rc::new(Node::NumericLiteral(42))),
            }))]
            .to_vec(),
        }));
        body.push(Rc::new(Node::ExpressionStatement(Some(Rc::new(
            Node::AssignmentExpression {
                operator: '=',
                left: Some(Rc::new(Node::Identifier("foo".to_string()))),
                right: Some(Rc::new(Node::NumericLiteral(1))),
            },
        )))));
        expected.set_body(body);
        assert_eq!(expected, parser.parse_ast());
    }

    #[test]
    fn test_define_function() {
        let input = "function foo() { return 42; }".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let mut expected = Program::new();
        let mut body = Vec::new();
        body.push(Rc::new(Node::FunctionDeclaration {
            id: Some(Rc::new(Node::Identifier("foo".to_string()))),
            params: [].to_vec(),
            body: Some(Rc::new(Node::BlockStatement {
                body: [Some(Rc::new(Node::ReturnStatement {
                    argument: Some(Rc::new(Node::NumericLiteral(42))),
                }))]
                .to_vec(),
            })),
        }));
        expected.set_body(body);
        assert_eq!(expected, parser.parse_ast());
    }

    #[test]
    fn test_define_function_with_args() {
        let input = "function foo(a, b) { return a+b; }".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let mut expected = Program::new();
        let mut body = Vec::new();
        body.push(Rc::new(Node::FunctionDeclaration {
            id: Some(Rc::new(Node::Identifier("foo".to_string()))),
            params: [
                Some(Rc::new(Node::Identifier("a".to_string()))),
                Some(Rc::new(Node::Identifier("b".to_string()))),
            ]
            .to_vec(),
            body: Some(Rc::new(Node::BlockStatement {
                body: [Some(Rc::new(Node::ReturnStatement {
                    argument: Some(Rc::new(Node::BinaryExpression {
                        operator: '+',
                        left: Some(Rc::new(Node::Identifier("a".to_string()))),
                        right: Some(Rc::new(Node::Identifier("b".to_string()))),
                    })),
                }))]
                .to_vec(),
            })),
        }));
        expected.set_body(body);
        assert_eq!(expected, parser.parse_ast());
    }

    #[test]
    fn test_add_function_add_num() {
        let input = "function foo() { return 42; } var result = foo() + 1;".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let mut expected = Program::new();
        let mut body = Vec::new();
        body.push(Rc::new(Node::FunctionDeclaration {
            id: Some(Rc::new(Node::Identifier("foo".to_string()))),
            params: [].to_vec(),
            body: Some(Rc::new(Node::BlockStatement {
                body: [Some(Rc::new(Node::ReturnStatement {
                    argument: Some(Rc::new(Node::NumericLiteral(42))),
                }))]
                .to_vec(),
            })),
        }));
        body.push(Rc::new(Node::VariableDeclaration {
            declarations: [Some(Rc::new(Node::VariableDeclarator {
                id: Some(Rc::new(Node::Identifier("result".to_string()))),
                init: Some(Rc::new(Node::BinaryExpression {
                    operator: '+',
                    left: Some(Rc::new(Node::CallExpression {
                        callee: Some(Rc::new(Node::Identifier("foo".to_string()))),
                        arguments: [].to_vec(),
                    })),
                    right: Some(Rc::new(Node::NumericLiteral(1))),
                })),
            }))]
            .to_vec(),
        }));
        expected.set_body(body);
        assert_eq!(expected, parser.parse_ast());
    }
}
