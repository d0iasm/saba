use crate::renderer::dom::api::get_element_by_id;
use crate::renderer::dom::node::Node as DomNode;
use crate::renderer::dom::node::NodeKind as DomNodeKind;
use crate::renderer::js::ast::Node;
use crate::renderer::js::ast::Program;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::cell::RefCell;
use core::fmt::Display;
use core::fmt::Formatter;
use core::ops::Add;
use core::ops::Sub;

#[derive(Debug, Clone)]
/// https://262.ecma-international.org/13.0/#sec-ecmascript-language-types
pub enum RuntimeValue {
    /// https://tc39.es/ecma262/#sec-numeric-types
    Number(u64),
    /// https://tc39.es/ecma262/#sec-ecmascript-language-types-string-type
    StringLiteral(String),
    /// https://dom.spec.whatwg.org/#interface-htmlcollection
    /// https://dom.spec.whatwg.org/#element
    HtmlElement {
        object: Rc<RefCell<DomNode>>,
        property: Option<String>,
    },
    Function(Function),
}

impl Display for RuntimeValue {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        let s = match self {
            RuntimeValue::Number(value) => format!("{}", value),
            RuntimeValue::StringLiteral(value) => value.to_string(),
            RuntimeValue::HtmlElement {
                object,
                property: _,
            } => {
                format!("HtmlElement: {:#?}", object)
            }
            RuntimeValue::Function(func) => format!("{}", func.id),
        };
        write!(f, "{}", s)
    }
}

impl PartialEq for RuntimeValue {
    fn eq(&self, other: &Self) -> bool {
        match self {
            RuntimeValue::Number(v1) => match other {
                RuntimeValue::Number(v2) => v1 == v2,
                _ => false,
            },
            RuntimeValue::StringLiteral(v1) => match other {
                RuntimeValue::StringLiteral(v2) => v1 == v2,
                _ => false,
            },
            RuntimeValue::HtmlElement {
                object: _,
                property: _,
            } => false,
            RuntimeValue::Function(func1) => match other {
                RuntimeValue::Function(func2) => func1.id == func2.id,
                _ => false,
            },
        }
    }
}

impl Add<RuntimeValue> for RuntimeValue {
    type Output = RuntimeValue;

    fn add(self, rhs: RuntimeValue) -> RuntimeValue {
        // https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-applystringornumericbinaryoperator
        if let (RuntimeValue::Number(left_num), RuntimeValue::Number(right_num)) = (&self, &rhs) {
            return RuntimeValue::Number(left_num + right_num);
        }

        RuntimeValue::StringLiteral(self.to_string() + &rhs.to_string())
    }
}

impl Sub<RuntimeValue> for RuntimeValue {
    type Output = RuntimeValue;

    fn sub(self, rhs: RuntimeValue) -> RuntimeValue {
        // https://tc39.es/ecma262/multipage/ecmascript-data-types-and-values.html#sec-numeric-types-number-subtract
        if let (RuntimeValue::Number(left_num), RuntimeValue::Number(right_num)) = (&self, &rhs) {
            return RuntimeValue::Number(left_num - right_num);
        }

        // NaN: Not a Number
        RuntimeValue::Number(u64::MIN)
    }
}

type VariableMap = Vec<(String, Option<RuntimeValue>)>;

/// https://262.ecma-international.org/12.0/#sec-environment-records
#[derive(Debug, Clone)]
pub struct Environment {
    variables: VariableMap,
    outer: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    fn new(outer: Option<Rc<RefCell<Environment>>>) -> Self {
        Self {
            variables: VariableMap::new(),
            outer,
        }
    }

    pub fn get_function(&self, name: String) -> Option<RuntimeValue> {
        for variable in &self.variables {
            if variable.0 == name {
                if let Some(RuntimeValue::Function(_)) = &variable.1 {
                    return variable.1.clone();
                }
            }
        }

        if let Some(env) = &self.outer {
            env.borrow_mut().get_function(name)
        } else {
            None
        }
    }

    pub fn get_variable(&self, name: String) -> Option<RuntimeValue> {
        for variable in &self.variables {
            if variable.0 == name {
                return variable.1.clone();
            }
        }

        if let Some(env) = &self.outer {
            env.borrow_mut().get_variable(name)
        } else {
            None
        }
    }

    fn add_variable(&mut self, name: String, value: Option<RuntimeValue>) {
        self.variables.push((name, value));
    }

    fn update_variable(&mut self, name: String, value: Option<RuntimeValue>) {
        for i in 0..self.variables.len() {
            // If find a varialbe, remove the old entry and add new entry.
            if self.variables[i].0 == name {
                self.variables.remove(i);
                self.variables.push((name, value));
                return;
            }
        }

        // Check the outer environment.
        if let Some(env) = &self.outer {
            env.borrow_mut().update_variable(name, value);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    id: String,
    params: Vec<Option<Rc<Node>>>,
    body: Option<Rc<Node>>,
}

impl Function {
    fn new(id: String, params: Vec<Option<Rc<Node>>>, body: Option<Rc<Node>>) -> Self {
        Self { id, params, body }
    }
}

#[derive(Debug, Clone)]
pub struct JsRuntime {
    dom_root: Option<Rc<RefCell<DomNode>>>,
    dom_modified: bool,
    functions: Vec<Function>,
    env: Rc<RefCell<Environment>>,
}

impl JsRuntime {
    pub fn new(dom_root: Rc<RefCell<DomNode>>) -> Self {
        let mut env = Environment::new(None);
        env.add_variable(
            "document".to_string(),
            Some(RuntimeValue::HtmlElement {
                object: dom_root.clone(),
                property: None,
            }),
        );

        env.add_variable(
            "getElementById".to_string(),
            Some(RuntimeValue::Function(Function::new(
                "getElementById".to_string(),
                vec![Node::new_identifier("target".to_string())],
                None,
            ))),
        );

        Self {
            dom_root: Some(dom_root),
            dom_modified: false,
            functions: Vec::new(),
            env: Rc::new(RefCell::new(env)),
        }
    }

    pub fn dom_root(&self) -> Option<Rc<RefCell<DomNode>>> {
        self.dom_root.clone()
    }

    pub fn dom_modified(&self) -> bool {
        self.dom_modified
    }

    /// https://developer.mozilla.org/en-US/docs/Web/API
    ///
    /// returns a tuple (bool, Option<RuntimeValue>)
    ///   bool: whether or not a Browser API is found
    ///   Option<RuntimeValue>: the result of a Browser API
    fn call_browser_api(
        &mut self,
        func: &RuntimeValue,
        arguments: &[Option<Rc<Node>>],
        env: Rc<RefCell<Environment>>,
    ) -> (bool, Option<RuntimeValue>) {
        if func == &RuntimeValue::StringLiteral("console.log".to_string()) {
            match self.eval(&arguments[0], env.clone()) {
                Some(_arg) => {
                    //println!("[console.log] {:?}", arg.to_string());
                    return (true, None);
                }
                None => return (false, None),
            }
        }

        if let RuntimeValue::HtmlElement {
            object: _,
            property,
        } = func
        {
            if property == &Some("getElementById".to_string()) {
                let arg = match self.eval(&arguments[0], env.clone()) {
                    Some(a) => a,
                    None => return (true, None),
                };
                let target = match get_element_by_id(self.dom_root.clone(), &arg.to_string()) {
                    Some(n) => n,
                    None => return (true, None),
                };
                return (
                    true,
                    Some(RuntimeValue::HtmlElement {
                        object: target,
                        property: None,
                    }),
                );
            }
        }

        (false, None)
    }

    fn eval(
        &mut self,
        node: &Option<Rc<Node>>,
        env: Rc<RefCell<Environment>>,
    ) -> Option<RuntimeValue> {
        let node = match node {
            Some(n) => n,
            None => return None,
        };

        match node.borrow() {
            Node::ExpressionStatement(expr) => self.eval(expr, env.clone()),
            Node::BlockStatement { body } => {
                let mut result: Option<RuntimeValue> = None;
                for stmt in body {
                    result = self.eval(stmt, env.clone());
                }
                result
            }
            Node::ReturnStatement { argument } => self.eval(argument, env.clone()),
            Node::FunctionDeclaration { id, params, body } => {
                let id = match self.eval(id, env.clone()) {
                    Some(value) => match value {
                        RuntimeValue::StringLiteral(s) => s,
                        _ => {
                            panic!("unexpected runtime value {:?}", node)
                        }
                    },
                    None => return None,
                };
                let cloned_body = body.as_ref().cloned();
                env.borrow_mut().add_variable(
                    id.to_string(),
                    Some(RuntimeValue::Function(Function::new(
                        id,
                        params.to_vec(),
                        cloned_body,
                    ))),
                );
                //self.functions
                //   .push(Function::new(id, params.to_vec(), cloned_body));
                None
            }
            Node::VariableDeclaration { declarations } => {
                for declaration in declarations {
                    self.eval(declaration, env.clone());
                }
                None
            }
            Node::VariableDeclarator { id, init } => {
                if let Some(node) = id {
                    if let Node::Identifier(id) = node.borrow() {
                        let init = self.eval(init, env.clone());
                        env.borrow_mut().add_variable(id.to_string(), init);
                    }
                }
                None
            }
            Node::BinaryExpression {
                operator,
                left,
                right,
            } => {
                let left_value = match self.eval(left, env.clone()) {
                    Some(value) => value,
                    None => return None,
                };
                let right_value = match self.eval(right, env.clone()) {
                    Some(value) => value,
                    None => return None,
                };

                // https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-applystringornumericbinaryoperator
                if operator == &'+' {
                    Some(left_value + right_value)
                } else if operator == &'-' {
                    Some(left_value - right_value)
                } else {
                    None
                }
            }
            Node::AssignmentExpression {
                operator,
                left,
                right,
            } => {
                if operator == &'=' {
                    // Variable reassignment.
                    if let Some(node) = left {
                        if let Node::Identifier(id) = node.borrow() {
                            let new_value = self.eval(&right, env.clone());
                            env.borrow_mut().update_variable(id.to_string(), new_value);
                            return None;
                        }
                    }

                    // If the left value is HtmlElement, update DOM.
                    let left_value = match self.eval(&left, env.clone()) {
                        Some(value) => value,
                        None => return None,
                    };
                    let right_value = match self.eval(&right, env.clone()) {
                        Some(value) => value,
                        None => return None,
                    };

                    match left_value {
                        RuntimeValue::HtmlElement { object, property } => {
                            if let Some(p) = property {
                                // this is the implementation of
                                // `document.getElementById("target").textContent = "foobar";`
                                if p == "textContent" {
                                    // Not necessary to set dom_modified=true because only text
                                    // content is changed.
                                    object.borrow_mut().set_first_child(Some(Rc::new(
                                        RefCell::new(DomNode::new(DomNodeKind::Text(
                                            right_value.to_string(),
                                        ))),
                                    )));
                                }
                                // this is the implementation of
                                // `document.getElementById("target").innerHTML = "foobar";`
                                // Currently, an assignment value should be a text like "foobar".
                                if p == "innerHTML" {
                                    self.dom_modified = true;
                                    object.borrow_mut().set_first_child(Some(Rc::new(
                                        RefCell::new(DomNode::new(DomNodeKind::Text(
                                            right_value.to_string(),
                                        ))),
                                    )));
                                }
                            }
                        }
                        _ => {}
                    }
                }
                None
            }
            Node::MemberExpression { object, property } => {
                let object_value = match self.eval(&object, env.clone()) {
                    Some(value) => value,
                    None => return None,
                };
                let property_value = match self.eval(&property, env.clone()) {
                    Some(value) => value,
                    // return RuntimeValue in `object` because of no `property`
                    None => return Some(object_value),
                };

                match object_value {
                    // return html element for DOM manipulation
                    RuntimeValue::HtmlElement { object, property } => {
                        match env.borrow_mut().get_function(property_value.to_string()) {
                            Some(func) => {
                                return Some(func);
                            }
                            None => {}
                        };

                        assert!(property.is_none());

                        // set `property` to the HtmlElement value.
                        Some(RuntimeValue::HtmlElement {
                            object,
                            property: Some(property_value.to_string()),
                        })
                    }
                    _ => {
                        if object_value == RuntimeValue::StringLiteral("document".to_string()) {
                            // TOOD: this is tricky to support member functions for document.*. find smarter way...
                            if property_value
                                == RuntimeValue::StringLiteral("getElementById".to_string())
                            {
                                return Some(
                                    object_value
                                        + RuntimeValue::StringLiteral(".".to_string())
                                        + property_value,
                                );
                            }

                            // set `property` to the HtmlElement value.
                            return Some(RuntimeValue::HtmlElement {
                                object: self.dom_root.clone().expect("failed to get root node"),
                                property: Some(property_value.to_string()),
                            });
                        }

                        /*
                        TODO: support window.location.href.
                        // dom_root.window().location()
                        if object_value == RuntimeValue::StringLiteral("location".to_string()) {
                            if property_value == RuntimeValue::StringLiteral("href".to_string()) {
                                //println!("[location.href] {:?}", self.url);
                                return Some(RuntimeValue::StringLiteral(self.url.clone()));
                            }

                            if property_value == RuntimeValue::StringLiteral("hash".to_string()) {
                                let hash = match self.url.find('#') {
                                    Some(i) => self.url[i..].to_string(),
                                    None => "".to_string(),
                                };
                                //println!("[location.hash] {:?}", hash);
                                return Some(RuntimeValue::StringLiteral(hash.clone()));
                            }
                        }
                        */

                        // return a concatenated string such as "console.log"
                        Some(
                            object_value
                                + RuntimeValue::StringLiteral(".".to_string())
                                + property_value,
                        )
                    }
                }
            }
            Node::CallExpression { callee, arguments } => {
                // Create a new scope.
                let new_env = Rc::new(RefCell::new(Environment::new(Some(env))));

                let callee_value = match self.eval(&callee, new_env.clone()) {
                    Some(value) => value,
                    None => return None,
                };

                // call a Browser API
                let api_result = self.call_browser_api(&callee_value, arguments, new_env.clone());
                if api_result.0 {
                    return api_result.1;
                }

                // find a function defined in the JS code
                /*
                let function = {
                    let mut f: Option<Function> = None;

                    for func in &self.functions {
                        if callee_value == RuntimeValue::StringLiteral(func.id.to_string()) {
                            f = Some(func.clone());
                        }
                    }

                    match f {
                        Some(f) => f,
                        None => unimplemented!("function {:?} doesn't exist", callee),
                    }
                };
                */
                let function = match callee_value {
                    RuntimeValue::Function(func) => func,
                    _ => {
                        panic!("{:#?} cannot be called", callee_value);
                    }
                };

                // assign arguments to params as local variables
                assert!(arguments.len() == function.params.len());
                for (i, item) in arguments.iter().enumerate() {
                    let name = match self.eval(&function.params[i], new_env.clone()) {
                        Some(value) => match value {
                            RuntimeValue::StringLiteral(s) => s,
                            _ => {
                                panic!("unexpected runtime value {:?}", node)
                            }
                        },
                        None => return None,
                    };

                    new_env
                        .borrow_mut()
                        .add_variable(name, self.eval(item, new_env.clone()));
                }

                // call function with arguments
                self.eval(&function.body.clone(), new_env.clone())
            }
            Node::Identifier(name) => {
                match env.borrow_mut().get_variable(name.to_string()) {
                    Some(v) => {
                        return Some(v);
                    }
                    // first time to evaluate this identifier
                    None => Some(RuntimeValue::StringLiteral(name.to_string())),
                }
            }
            Node::NumericLiteral(value) => Some(RuntimeValue::Number(*value)),
            Node::StringLiteral(value) => Some(RuntimeValue::StringLiteral(value.to_string())),
        }
    }

    pub fn execute(&mut self, program: &Program) {
        for node in program.body() {
            self.eval(&Some(node.clone()), self.env.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::js::ast::JsParser;
    use crate::renderer::js::token::JsLexer;

    #[test]
    fn test_num() {
        let dom = Rc::new(RefCell::new(DomNode::new(DomNodeKind::Document)));
        let input = "42".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new(dom);
        let expected = [Some(RuntimeValue::Number(42))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    #[test]
    fn test_add_nums() {
        let dom = Rc::new(RefCell::new(DomNode::new(DomNodeKind::Document)));
        let input = "1 + 2".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new(dom);
        let expected = [Some(RuntimeValue::Number(3))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    #[test]
    fn test_sub_nums() {
        let dom = Rc::new(RefCell::new(DomNode::new(DomNodeKind::Document)));
        let input = "2 - 1".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new(dom);
        let expected = [Some(RuntimeValue::Number(1))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    #[test]
    fn test_assign_variable() {
        let dom = Rc::new(RefCell::new(DomNode::new(DomNodeKind::Document)));
        let input = "var foo=42;".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new(dom);
        let expected = [None];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    #[test]
    fn test_add_variable_and_num() {
        let dom = Rc::new(RefCell::new(DomNode::new(DomNodeKind::Document)));
        let input = "var foo=42; foo+1".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new(dom);
        let expected = [None, Some(RuntimeValue::Number(43))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    #[test]
    fn test_reassign_variable() {
        let dom = Rc::new(RefCell::new(DomNode::new(DomNodeKind::Document)));
        let input = "var foo=42; foo=1; foo".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new(dom);
        let expected = [None, None, Some(RuntimeValue::Number(1))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    #[test]
    fn test_reassign_string() {
        let dom = Rc::new(RefCell::new(DomNode::new(DomNodeKind::Document)));
        let input = "var foo=42; foo=\"<h1>foo</h1>\"; foo".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new(dom);
        let expected = [
            None,
            None,
            Some(RuntimeValue::StringLiteral("<h1>foo</h1>".to_string())),
        ];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }
    #[test]
    fn test_add_function_and_num() {
        let dom = Rc::new(RefCell::new(DomNode::new(DomNodeKind::Document)));
        let input = "function foo() { return 42; } foo()+1".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new(dom);
        let expected = [None, Some(RuntimeValue::Number(43))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    #[test]
    fn test_define_function_with_args() {
        let dom = Rc::new(RefCell::new(DomNode::new(DomNodeKind::Document)));
        let input = "function foo(a, b) { return a + b; } foo(1, 2) + 3;".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new(dom);
        let expected = [None, Some(RuntimeValue::Number(6))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    #[test]
    fn test_local_variable() {
        let dom = Rc::new(RefCell::new(DomNode::new(DomNodeKind::Document)));
        let input = "var a=42; function foo() { var a=1; return a; } foo()+a".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new(dom);
        let expected = [None, None, Some(RuntimeValue::Number(43))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    #[test]
    fn test_browser_api() {
        let dom = Rc::new(RefCell::new(DomNode::new(DomNodeKind::Document)));
        let input = "document.getElementById(\"target\")".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new(dom);
        let expected = [None, None, Some(RuntimeValue::Number(43))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }
}
