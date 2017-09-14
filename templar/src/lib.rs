#[macro_use]
extern crate nom;

pub mod parse;
pub mod escape;
pub mod output;


#[derive(Debug, Clone)]
pub struct Element {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone)]
pub enum Node {
    Doctype(String),
    Directive { command: String, children: Vec<Node> },
    Text(String),
    RawText(String), // for javascript
    Element(Element),
}

impl Node {
    pub fn should_indent(&self) -> bool {
        match self {
            &Node::Element(_) | &Node::Text(_) | &Node::Doctype(_) => true,
            &Node::Directive { .. } | &Node::RawText(_) => false,
        }
    }

    pub fn supports_children(&self) -> bool {
        match self {
            &Node::Directive { .. } | &Node::Element(_) => true,
            &Node::Doctype(_) | &Node::Text(_) | &Node::RawText(_) => false,
        }
    }

    pub fn append_child(&mut self, node:Node) -> bool {
        match self {
            &mut Node::Doctype(_) => false,
            &mut Node::Directive { ref mut children, .. } => {
                children.push(node);
                true
            },
            &mut Node::Text(_) => false,
            &mut Node::RawText(_) => false, // for javascript
            &mut Node::Element(ref mut ele) => {
                ele.children.push(node);
                true
            },
        }
    }
}

pub fn element(name:&str, attributes: Vec<(&str, &str)>) -> Element {
    Element {
        name: name.into(),
        attributes: attributes.iter().map(|&(k, v)| (k.into(), v.into())).collect(),
        children: Vec::new(),
    }
}

pub fn contains<T, F>(opt: Option<T>, f: F) -> bool where F: Fn(&T) -> bool {
    opt.iter().any(f)
}


#[derive(Debug)]
pub struct TemplateContext {
    pub nodes:Vec<Node>,
}

impl TemplateContext {
    pub fn empty() -> TemplateContext {
        TemplateContext {
            nodes: Vec::new()
        }
    }
}