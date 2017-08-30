use nom::*; // {digit, space, alphanumeric}
use std::str;
use std::cmp::max;

use contains;

fn always<T>(_:T) -> bool {
    true
}

fn is_identifier(c:char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_'
}

fn is_spacer(c:char) -> bool {
    c.is_whitespace()
}

fn noneify_blank_string(str: &str) -> Option<String> {
    if str.is_empty() {
        None
    }  else {
        Some(str.into())
    }
}

named!(identifier<&str, &str>,
    take_while1!(is_identifier)
);

named!(element_id<&str, &str>,
    do_parse!(
        tag!("#") >>
        id : identifier >>
        (id)
    )
);

named!(rest<&str, &str>,
    take_while!(always)
);

named!(element_class<&str, &str>,
    do_parse!(
        tag!(".") >>
        class_name : identifier >>
        (class_name)
    )
);

named!(quoted_value<&str, &str>,
    delimited!(
        tag!("\""),
        take_until!("\""),
        tag!("\"")
    )
);

named!(attribute_content<&str, &str>,
    take_till!(is_spacer)
);

named!(key_value_pair<&str, (String, String)>,
    do_parse!(
        k: identifier >>
        tag!("=") >>
        v: alt_complete!( quoted_value | attribute_content ) >>
        (k.into(), v.into())
    )
);

named!(text_line<&str, LineContent>,
    do_parse!(
        tag!("|") >>
        rr: map!(rest, |s| s.trim() ) >>
        ( LineContent::Text(rr.to_string()) )
    )
);

named!(javascript_text_line<&str, LineContent>,
    do_parse!(
        rr: rest >>
        ( LineContent::Text( rr.to_string()) )
    )
);

named!(directive_line<&str, LineContent>,
    do_parse!(
        tag!("=") >>
        rr : rest >>
        ( LineContent::Directive(rr.trim().to_string()) )
    )
);

named!(comment_line<&str, LineContent>,
    do_parse!(
        tag!("/") >>
        rr : rest >>
        ( LineContent::Comment(rr.trim().to_string()))
    )
);

named!(doctype_line<&str, LineContent>,
    do_parse!(
        tag!("doctype") >>
        space >>
        rr: rest >>
        ( LineContent::Doctype(rr.trim().to_string()) )
    )
);

named!(javascript_line<&str, LineContent>,
    do_parse!(
        tag!(":javascript") >>
        rr : rest >>
        ( LineContent::Javascript )
    )
);

named!(tag_element_line<&str, LineContent>,
    do_parse!(
        tag: identifier >>
        class_ids: many0!(
            alt_complete!(
                map!(element_class, |s| ClassId::Class(s.to_string())) |
                map!(element_id, |s| ClassId::Id(s.to_string()))
            )
        ) >>
        kvps: many0!(ws!(complete!(key_value_pair))) >>
        rr : rest >>
        ( LineContent::Element(HtmlElement {
                tag: Some(tag.to_string()),
                classes_ids: class_ids,
                attributes: kvps,
                inner_text: noneify_blank_string(rr.trim()),
          })
        )
    )
);

named!(class_id_only_line<&str, LineContent>,
    do_parse!(
        class_ids: many1!(
            alt_complete!(
                map!(element_class, |s| ClassId::Class(s.to_string())) |
                map!(element_id, |s| ClassId::Id(s.to_string()))
            )
        ) >>
        kvps: many0!(ws!(complete!(key_value_pair))) >>
        rr : rest >>
        ( LineContent::Element(HtmlElement {
                tag: None,
                classes_ids: class_ids,
                attributes: kvps,
                inner_text: noneify_blank_string(rr.trim()),
          })
        )
    )
);

named!(line_p<&str, LineContent>,
    alt_complete!(doctype_line | comment_line | javascript_line | tag_element_line | class_id_only_line | directive_line | text_line)
);

#[derive(Debug)]
enum ClassId {
    Id(String),
    Class(String),
}

#[derive(Debug)]
struct HtmlElement {
    tag: Option<String>,
    classes_ids: Vec<ClassId>,
    attributes: Vec<(String, String)>,
    inner_text: Option<String>,
}

#[derive(Debug)]
enum LineContent {
    Comment(String),
    Javascript,
    Doctype(String),
    Element(HtmlElement),
    Directive(String),
    Text(String),
}

fn indentation(str: &str) -> Option<usize> {
    str.chars().position(|c| !c.is_whitespace())
}

use super::{Node, Element, element};

pub type ParseResult = Result<Vec<Node>, ParseError>;

#[derive(Debug)]
pub struct ParseError {
    pub line_number: usize,
    pub context: Vec<String>, // last few lines
    pub character: Option<u64>,
    pub reason:ErrorReason,
}

#[derive(Debug, Clone)]
pub enum ErrorReason {
    MisplacedDocType,
    MultipleIds,
    IllegalNesting(String),
    Parse(String),
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
enum ParseMode {
    Normal,
    InlineJavascript,
}

fn element_for(html_element: HtmlElement) -> Result<Element, ErrorReason> {
    let name = html_element.tag.unwrap_or_else(|| "div".into());

    let mut attributes = html_element.attributes;

    let mut id : Option<String> = None;
    let mut classes : Vec<String> = Vec::new();

    for class_id in html_element.classes_ids {
        match class_id {
            ClassId::Id(string) => {
                if id.is_some() {
                    return Err(ErrorReason::MultipleIds);
                } else {
                    id = Some(string);
                }
            }
            ClassId::Class(string) => {
                classes.push(string);
            }
        }
    }

    if let Some(id) = id {
        attributes.push(("id".into(), id));
    }
    if !classes.is_empty() {
        attributes.push(("class".into(), classes.join(" ")));
    }

    let mut children = Vec::new();

    if let Some(text) = html_element.inner_text {
        children.push(Node::Text(text));
    }

    Ok(Element {
        name,
        attributes,
        children,
    })
}


pub fn parse(content:&str) -> ParseResult {
    let mut out_nodes: Vec<Node> = Vec::new();
    let mut out_stack: Vec<(Node, usize)> = Vec::new();

    let mut mode = ParseMode::Normal;

    let lines : Vec<String> = content.lines().map(|s|s.to_string()).collect();

    let produce_context = |line_number: usize| -> Vec<String> {
        let start_line = max((line_number as i64) - 5, 0) as usize;
        let end_line = line_number + 1;
        let ter = &lines[start_line..end_line];
        ter.iter().cloned().collect()
    };

    for (line_idx, line) in lines.iter().enumerate() {
        // indentation and slicing first
//        println!("-> {}", line);
        if let Some(indent) = indentation(line) {
            let (_, rest) = line.split_at(indent);

//            println!("!indent is {:?}", indent);

            while contains(out_stack.last(), |&&(_, n)| n >= indent ) {
                let (node, _) = out_stack.pop().expect("the top element");

                if let Some(&mut (ref mut next_down, _)) = out_stack.last_mut() {
//                    println!("! push top element {:?} to next down {:?}", ele.name, next_down.name);
                    if !next_down.append_child(node.clone()) {
                        return Err(ParseError {
                            line_number: line_idx,
                            context: produce_context(line_idx),
                            character: None,
                            reason: ErrorReason::IllegalNesting(format!("parent -> {:?} child -> {:?}", next_down, node)),
                        });
                    }
                } else {
//                    println!("! push top element {:?} to out", ele.name);
                    out_nodes.push(node);
                }
                mode = ParseMode::Normal
            }

            let line_content_result = match mode {
                ParseMode::InlineJavascript => javascript_text_line(rest),
                ParseMode::Normal => line_p(rest)
            };

            match line_content_result {
                IResult::Done(_, line_content) => {
//                    println!("Done-> {}", format!("{:?}",line_content).green());

                    match (mode, line_content) {
                        (ParseMode::InlineJavascript, LineContent::Text(string)) => {
//                            println!("!added some javascript content");
                            let &mut (ref mut next_down, _) = out_stack.last_mut().expect("a javascript node");
                            let node = Node::RawText(string);
                            if !next_down.append_child(node.clone()) {
                                return Err(ParseError {
                                    line_number: line_idx,
                                    context: produce_context(line_idx),
                                    character: None,
                                    reason: ErrorReason::IllegalNesting(format!("parent -> {:?} child -> {:?}", next_down, node)),
                                });
                            }
                        },
                        (ParseMode::InlineJavascript, _) => {
                            panic!("uhh")
                        },
                        (ParseMode::Normal, content) => {
                            match content {
                                LineContent::Comment(_) => {
                                    // ignore
                                },
                                LineContent::Javascript => {
//                                    println!("!javasript element, startin javascript mode");
                                    let mut ele = element("script", vec![("type", "text/javascript")]);
                                    ele.children.push(Node::RawText("\n".into()));
                                    mode = ParseMode::InlineJavascript;
                                    out_stack.push((Node::Element(ele), indent));
                                },
                                LineContent::Doctype(string) => {
                                    if !out_stack.is_empty() {
                                        return Err(ParseError {
                                            line_number: line_idx,
                                            context: produce_context(line_idx),
                                            character: None,
                                            reason: ErrorReason::MisplacedDocType,
                                        });
                                    }
//                                    println!("!doctype to out");
                                    out_nodes.push(Node::Doctype(string));
                                },
                                LineContent::Element(ele) => {
                                    match element_for(ele) {
                                        Ok(e) => {
//                                            println!("!{}", format!("pushing element {:?}", e.name));
                                            out_stack.push((Node::Element(e), indent));
                                        },
                                        Err(reason) => {
                                            return Err(ParseError {
                                                line_number: line_idx,
                                                context: produce_context(line_idx),
                                                character: None,
                                                reason,
                                            });
                                        },
                                    }
                                },
                                LineContent::Directive(string) => {
                                    let node = Node::Directive { command: string, children: Vec::new() };
                                    out_stack.push((node, indent));
                                },
                                LineContent::Text(string) => {
                                    let node = Node::Text(string);
                                    if let Some(&mut (ref mut next_down, _)) = out_stack.last_mut() {
//                                        println!("!push text to parent {:?}", next_down.name);
                                        if !next_down.append_child(node.clone()) {
                                            return Err(ParseError {
                                                line_number: line_idx,
                                                context: produce_context(line_idx),
                                                character: None,
                                                reason: ErrorReason::IllegalNesting(format!("parent -> {:?} child -> {:?}", next_down, node)),
                                            });
                                        }
                                    } else {
//                                        println!("!push text to root");
                                        out_nodes.push(node);
                                    }
                                },
                            }
                        },
                    }
                },
                IResult::Error(err) => {
                    return Err(ParseError {
                        line_number: line_idx,
                        context: produce_context(line_idx),
                        character: None,
                        reason:ErrorReason::Parse(format!("{:?}", err)),
                    });
                },
                IResult::Incomplete(needed) => {
                    return Err(ParseError {
                        line_number: line_idx,
                        context: produce_context(line_idx),
                        character: None,
                        reason:ErrorReason::Parse(format!("{:?}", needed)),
                    });
                },
            }
        }
    }

    // push remainder on
    while let Some((node, _)) = out_stack.pop() {
        if let Some(&mut (ref mut next_down, _)) = out_stack.last_mut() {
//            println!("!push ele {:?} to next down {:?}", ele.name, next_down.name);
            if !next_down.append_child(node.clone()) {
                return Err(ParseError {
                    line_number: 0,
                    context: vec![],
                    character: None,
                    reason: ErrorReason::IllegalNesting(format!("parent -> {:?} child -> {:?}", next_down, node)),
                });
            }
        } else {
//            println!("!push ele to root {:?}", ele.name);
            out_nodes.push(node);
        }
    }

    Ok(out_nodes)
}