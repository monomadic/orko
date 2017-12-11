
use {Node, TemplateContext};
use std::io::{self, Write};

use escape::*;

#[derive(Debug)]
pub enum WriteError<DE> {
    DirectiveError(DE),
    IO(io::Error),
}

impl<DE> From<io::Error> for WriteError<DE> {
    fn from(err: io::Error) -> Self {
        WriteError::IO(err)
    }
}


pub trait DirectiveHandler {
    type DirectiveError;
    fn handle<W>(&mut self, context:&TemplateContext, command: &str, children: &[Node], base_indent: usize, indent_size: usize, writer: &mut W) -> Result<(), Self::DirectiveError> where W : Write;
}

pub fn write_out<W, DH>(nodes:&[Node], context:&TemplateContext, writer:&mut W, base_indent: usize, indent_size: usize, directive_handler:&mut DH) -> Result<(), WriteError<DH::DirectiveError>>
    where W : Write, DH: DirectiveHandler {
    for node in nodes {
        if node.should_indent() {
            for _ in 0..base_indent {
                writer.write(b" ")?;
            }
        }

        match node {
            &Node::Doctype(ref doctype) => {
                let out = format!("<!DOCTYPE {}>\n", doctype);
                writer.write(out.as_bytes())?;
                writer.write(b"\n")?;
            }
            &Node::Directive { ref command, ref children } => {
                // println!("handle directive -> {:?} children {:?}", command, children);
                directive_handler.handle(context, command, children, base_indent, indent_size, writer).map_err(WriteError::DirectiveError)?;
            }
            &Node::Text(ref text) => {
//                let out = escape_html(text).expect("escaped text");
//                writer.write(out.as_bytes())?;
                writer.write(text.as_bytes())?;
                if indent_size > 0 {
                    writer.write(b"\n")?;
                }
            },
            &Node::RawText(ref raw_text) => {
                writer.write(raw_text.as_bytes())?;
                writer.write(b"\n")?;
            },
            &Node::Element(ref element) => {
                let destroy_whitespace = element.name == "a";
                let seperate_close_tag = element.children.len() > 0 || element.name == "script" || element.name == "a";
                let trailing_slash : &str = if !seperate_close_tag { " /" } else { "" };

//                println!("ele -> {:?} Close tag -> {:?} trailing slash -> {:?}", element, seperate_close_tag, trailing_slash);

                let open_tag : String = if element.attributes.is_empty() {
                    format!("<{}{}>", element.name, trailing_slash)
                } else {
                    let attributes : Vec<String> = element.attributes.iter().map(|&(ref k, ref v)|
                        format!("{}=\"{}\"", k, escape_default(v))
                    ).collect();
                    format!("<{} {}{}>", element.name, attributes.join(" "), trailing_slash)
                };
                writer.write(open_tag.as_bytes())?;
                if indent_size > 0 && !destroy_whitespace {
                    writer.write(b"\n")?;
                }
                if seperate_close_tag {
                    if destroy_whitespace {
                        write_out(element.children.as_slice(), context, writer, 0, 0, directive_handler)?;
                    } else {
                        write_out(element.children.as_slice(), context, writer, base_indent + indent_size, indent_size, directive_handler)?;
                    }

                    let closing_tag : String = format!("</{}>", element.name);
                    if !destroy_whitespace {
                        for _ in 0..base_indent {
                            writer.write(b" ")?;
                        }
                    }
                    writer.write(closing_tag.as_bytes())?;
                    if indent_size > 0 {
                        writer.write(b"\n")?;
                    }

                }

            },
        }
    }

    Ok(())
}
