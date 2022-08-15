use crate::renderer::html::dom::get_element_by_id;
use crate::renderer::html::dom::Node as DomNode;
use crate::renderer::html::dom::NodeKind as DomNodeKind;
use crate::renderer::js::ast::Node;
use crate::renderer::js::ast::Program;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Add;
use std::rc::Rc;
use std::string::{String, ToString};
use std::vec::Vec;

#[derive(Debug, Clone)]
/// https://262.ecma-international.org/13.0/#sec-ecmascript-language-types
pub enum RuntimeValue {
    /// https://262.ecma-international.org/13.0/#sec-terms-and-definitions-number-value
    /// https://262.ecma-international.org/13.0/#sec-numeric-types
    Number(u64),
    /// https://262.ecma-international.org/13.0/#sec-terms-and-definitions-string-value
    /// https://262.ecma-international.org/13.0/#sec-ecmascript-language-types-string-type
    StringLiteral(String),
    /// https://dom.spec.whatwg.org/#interface-htmlcollection
    /// https://dom.spec.whatwg.org/#element
    HtmlElement {
        object: Rc<RefCell<DomNode>>,
        property: Option<String>,
    },
}

impl RuntimeValue {
    fn to_string(&self) -> String {
        match self {
            RuntimeValue::Number(value) => format!("{}", value),
            RuntimeValue::StringLiteral(value) => value.to_string(),
            RuntimeValue::HtmlElement {
                object,
                property: _,
            } => {
                format!("{:?}", object.borrow().kind())
            }
        }
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

        return RuntimeValue::StringLiteral(self.to_string() + &rhs.to_string());
    }
}

type VariableMap = HashMap<String, Option<RuntimeValue>>;

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

    pub fn get_variable(&self, name: String) -> Option<RuntimeValue> {
        match self.variables.get(&name) {
            Some(val) => val.clone(),
            None => {
                if let Some(p) = &self.outer {
                    p.borrow_mut().get_variable(name)
                } else {
                    None
                }
            }
        }
    }

    fn add_variable(&mut self, name: String, value: Option<RuntimeValue>) {
        self.variables.insert(name, value);
    }

    /*
    fn assign_variable(&mut self, name: String, value: Option<RuntimeValue>) {
        let entry = self.variables.entry(name.clone());
        match entry {
            Entry::Occupied(_) => {
                entry.insert(value);
            }
            Entry::Vacant(_) => {
                if let Some(p) = &self.outer {
                    p.borrow_mut().assign_variable(name, value);
                } else {
                    entry.insert(value);
                }
            }
        }
    }
    */
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
    url: String,
    pub global_variables: HashMap<String, Option<RuntimeValue>>,
    pub functions: Vec<Function>,
    pub env: Rc<RefCell<Environment>>,
}

impl JsRuntime {
    pub fn new(dom_root: Rc<RefCell<DomNode>>, url: String) -> Self {
        Self {
            dom_root: Some(dom_root),
            dom_modified: false,
            url,
            global_variables: HashMap::new(),
            functions: Vec::new(),
            env: Rc::new(RefCell::new(Environment::new(None))),
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
    ///   bool: whether or not a Web API is found
    ///   Option<RuntimeValue>: the result of a Web API
    fn call_web_api(
        &mut self,
        func: &RuntimeValue,
        arguments: &Vec<Option<Rc<Node>>>,
        env: Rc<RefCell<Environment>>,
    ) -> (bool, Option<RuntimeValue>) {
        if func == &RuntimeValue::StringLiteral("console.log".to_string()) {
            match self.eval(&arguments[0], env.clone()) {
                Some(arg) => {
                    println!("[console.log] {:?}", arg.to_string());
                    return (true, None);
                }
                None => return (false, None),
            }
        }

        if func == &RuntimeValue::StringLiteral("document.getElementById".to_string()) {
            let arg = match self.eval(&arguments[0], env.clone()) {
                Some(a) => a,
                None => return (true, None),
            };
            let target = match get_element_by_id(self.dom_root.clone(), &arg.to_string()) {
                Some(n) => n,
                None => return (true, None),
            };
            println!(
                "[document.getElementById] {:?}\n{:?}",
                arg.to_string(),
                target
            );
            return (
                true,
                Some(RuntimeValue::HtmlElement {
                    object: target,
                    property: None,
                }),
            );
        }

        /*
        if func == &RuntimeValue::StringLiteral("setTimeout".to_string()) {
            return (true, None);
        }
        */

        (false, None)
    }

    fn eval(
        &mut self,
        node: &Option<Rc<Node>>,
        env: Rc<RefCell<Environment>>,
    ) -> Option<RuntimeValue> {
        use std::borrow::Borrow;

        let node = match node {
            Some(n) => n,
            None => return None,
        };

        match node.borrow() {
            Node::ExpressionStatement(expr) => return self.eval(&expr, env.clone()),
            Node::BlockStatement { body } => {
                let mut result: Option<RuntimeValue> = None;
                for stmt in body {
                    result = self.eval(&stmt, env.clone());
                }
                result
            }
            Node::ReturnStatement { argument } => {
                return self.eval(&argument, env.clone());
            }
            Node::FunctionDeclaration { id, params, body } => {
                let id = match self.eval(&id, env.clone()) {
                    Some(value) => match value {
                        RuntimeValue::Number(n) => {
                            unimplemented!("id should be string but got {:?}", n)
                        }
                        RuntimeValue::StringLiteral(s) => s,
                        RuntimeValue::HtmlElement {
                            object: node,
                            property: _,
                        } => {
                            panic!("unexpected runtime value {:?}", node)
                        }
                    },
                    None => return None,
                };
                let cloned_body = match body {
                    Some(b) => Some(b.clone()),
                    None => None,
                };
                self.functions
                    .push(Function::new(id, params.to_vec(), cloned_body));
                None
            }
            Node::VariableDeclaration { declarations } => {
                for declaration in declarations {
                    self.eval(&declaration, env.clone());
                }
                None
            }
            Node::VariableDeclarator { id, init } => {
                if let Some(node) = id {
                    if let Node::Identifier(id) = node.borrow() {
                        let init = self.eval(&init, env.clone());
                        env.borrow_mut().add_variable(id.to_string(), init);
                        //self.global_variables.insert(id.to_string(), init);
                    }
                }
                None
            }
            Node::BinaryExpression {
                operator,
                left,
                right,
            } => {
                let left_value = match self.eval(&left, env.clone()) {
                    Some(value) => value,
                    None => return None,
                };
                let right_value = match self.eval(&right, env.clone()) {
                    Some(value) => value,
                    None => return None,
                };

                // https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-applystringornumericbinaryoperator
                if operator == &'+' {
                    Some(left_value + right_value)
                } else {
                    return None;
                }
            }
            Node::AssignmentExpression {
                operator,
                left,
                right,
            } => {
                if operator == &'=' {
                    let left_value = match self.eval(&left, env.clone()) {
                        Some(value) => value,
                        None => return None,
                    };
                    let right_value = match self.eval(&right, env.clone()) {
                        Some(value) => value,
                        None => return None,
                    };

                    println!("AssignmentExpression {:?} = {:?}", left_value, right_value);

                    match left_value {
                        RuntimeValue::Number(n) => panic!("unexpected value {:?}", n),
                        RuntimeValue::StringLiteral(_s) => {
                            // TODO: update variable here
                        }
                        RuntimeValue::HtmlElement { object, property } => {
                            if let Some(p) = property {
                                // this is the implementation of
                                // `document.getElementById("target").innerHTML = "foobar";`
                                // Currently, an assignment value should be a text like "foobar".
                                if p == "innerHTML" {
                                    self.dom_modified = true;
                                    object.borrow_mut().update_first_child(Some(Rc::new(
                                        RefCell::new(DomNode::new(DomNodeKind::Text(
                                            right_value.to_string(),
                                        ))),
                                    )));
                                }
                            }
                        }
                    }
                }
                return None;
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
                        assert!(property.is_none());

                        // set `property` to the HtmlElement value.
                        return Some(RuntimeValue::HtmlElement {
                            object,
                            property: Some(property_value.to_string()),
                        });
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

                        if object_value == RuntimeValue::StringLiteral("location".to_string()) {
                            if property_value == RuntimeValue::StringLiteral("href".to_string()) {
                                println!("[location.href] {:?}", self.url);
                                return Some(RuntimeValue::StringLiteral(self.url.clone()));
                            }

                            if property_value == RuntimeValue::StringLiteral("hash".to_string()) {
                                let hash = match self.url.find("#") {
                                    Some(i) => self.url[i..].to_string(),
                                    None => "".to_string(),
                                };
                                println!("[location.hash] {:?}", hash);
                                return Some(RuntimeValue::StringLiteral(hash.clone()));
                            }
                        }

                        // return a concatenated string such as "console.log"
                        return Some(
                            object_value
                                + RuntimeValue::StringLiteral(".".to_string())
                                + property_value,
                        );
                    }
                }
            }
            Node::CallExpression { callee, arguments } => {
                let env = Rc::new(RefCell::new(Environment::new(Some(env))));
                let callee_value = match self.eval(&callee, env.clone()) {
                    Some(value) => value,
                    None => return None,
                };

                // call a Web API
                let web_api_result = self.call_web_api(&callee_value, arguments, env.clone());
                if web_api_result.0 {
                    return web_api_result.1;
                }

                /*
                if callee_value
                    == RuntimeValue::StringLiteral("document.getElementById".to_string())
                {
                    let arg = match self.eval(&arguments[0], env.clone()) {
                        Some(a) => a,
                        None => return None,
                    };
                    let target = match get_element_by_id(self.dom_root.clone(), &arg.to_string()) {
                        Some(n) => n,
                        None => return None,
                    };
                    println!(
                        "[document.getElementById] {:?}\n{:?}",
                        arg.to_string(),
                        target
                    );
                    return Some(RuntimeValue::HtmlElement {
                        object: target,
                        property: None,
                    });
                }
                */

                let mut new_local_variables: VariableMap = VariableMap::new();

                // find a function defined in the JS code
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

                // assign arguments to params as local variables
                assert!(arguments.len() == function.params.len());
                for i in 0..arguments.len() {
                    let name = match self.eval(&function.params[i], env.clone()) {
                        Some(value) => match value {
                            RuntimeValue::Number(n) => {
                                unimplemented!("id should be string but got {:?}", n)
                            }
                            RuntimeValue::StringLiteral(s) => s,
                            RuntimeValue::HtmlElement {
                                object,
                                property: _,
                            } => {
                                panic!("unexpected runtime value {:?}", object)
                            }
                        },
                        None => return None,
                    };

                    new_local_variables.insert(name, self.eval(&arguments[i], env.clone()));
                }

                // call function with arguments
                self.eval(&function.body.clone(), env.clone())
            }
            Node::Identifier(name) => {
                /*
                // find a value from global variables
                for (var_name, var_value) in &self.global_variables {
                    if name == var_name && var_value.is_some() {
                        return var_value.clone();
                    }
                }
                */

                match env.borrow_mut().get_variable(name.to_string()) {
                    Some(v) => Some(v),
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
