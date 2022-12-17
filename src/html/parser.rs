use super::dom::{ElementAttributes, ElementChildren, ElementNode, Node, TextNode};

const GREATER_THAN: u8 = 0x003E;
const LESS_THAN: u8 = 0x003C;
const WHITESPACE: u8 = 0x0020;
const SLASH: u8 = 0x002F;

#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken(UnexpectedTokenError),
    PrematureEndOfFile(PrematureEndOfFileError),
    Generic(GenericError),
}

#[derive(Debug)]
pub struct GenericError {
    pub position: usize,
    pub message: String,
}

impl GenericError {
    pub fn new(position: usize, message: &str) -> GenericError {
        GenericError {
            message: format!("{} at {}", message, position),
            position,
        }
    }
}

#[derive(Debug)]
pub struct PrematureEndOfFileError {
    pub position: usize,
    pub message: String,
}

impl PrematureEndOfFileError {
    pub fn new(position: usize) -> PrematureEndOfFileError {
        PrematureEndOfFileError {
            position,
            message: "Premature end of file".to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct UnexpectedTokenError {
    pub position: usize,
    pub message: String,
}

impl UnexpectedTokenError {
    pub fn new(expected: &str, found: &str, position: usize) -> UnexpectedTokenError {
        UnexpectedTokenError {
            message: UnexpectedTokenError::build_error_message(expected, found, position),
            position,
        }
    }

    fn build_error_message(expected: &str, found: &str, position: usize) -> String {
        format!(
            "Expected \"{}\", found \"{}\" at {}",
            expected, found, position
        )
    }
}

struct CurrentElement {
    finished: bool,
    element: ElementNode,
}

impl CurrentElement {
    fn create_empty_element() -> ElementNode {
        ElementNode {
            tag_name: "".to_owned(),
            attributes: ElementAttributes::new(),
            children: ElementChildren::new(),
        }
    }

    pub fn new() -> CurrentElement {
        CurrentElement {
            finished: false,
            element: CurrentElement::create_empty_element(),
        }
    }

    pub fn reset(&mut self) {
        self.element = CurrentElement::create_empty_element();
        self.finished = false;
    }
}

pub struct Parser {
    input: Vec<u8>,
    position: usize,
}

impl Parser {
    pub fn new(input: &str) -> Parser {
        Parser {
            input: input.as_bytes().into(),
            position: 0,
        }
    }

    fn current_char(&self) -> u8 {
        self.input[self.position]
    }

    fn next_char(&self) -> u8 {
        self.input[self.position + 1]
    }

    fn get_tag_name(&mut self) -> Result<String, ParserError> {
        let mut tag_name = Vec::<u8>::with_capacity(1);

        while !self.eof() && self.current_char() != GREATER_THAN {
            let current_char = self.current_char();

            match current_char {
                0x0041..=0x005A | 0x0061..=0x007A | 0x0030..=0x0039 => {
                    tag_name.push(current_char);
                    self.position += 1;
                }
                _ => {
                    return Err(ParserError::UnexpectedToken(UnexpectedTokenError::new(
                        "tag name",
                        &(current_char as char).to_string(),
                        self.position,
                    )))
                }
            }
        }

        // Reached eof before closing tag
        if self.current_char() != GREATER_THAN {
            return Err(ParserError::PrematureEndOfFile(
                PrematureEndOfFileError::new(self.position),
            ));
        }

        self.position += 1;

        Ok(String::from_utf8(tag_name).unwrap())
    }

    fn eof(&self) -> bool {
        self.position >= self.input.len()
    }

    fn skip_whitespaces(&mut self) {
        if self.eof() {
            return;
        }

        if self.current_char() == WHITESPACE {
            self.position += 1;
            self.skip_whitespaces();
        }
    }

    fn get_text_content(&mut self) -> Result<String, ParserError> {
        let mut content = Vec::<u8>::new();

        while !self.eof() && self.current_char() != LESS_THAN {
            let current_char = self.current_char();

            if self.current_char() == GREATER_THAN {
                return Err(ParserError::UnexpectedToken(UnexpectedTokenError::new(
                    "text content",
                    &(self.current_char() as char).to_string(),
                    self.position,
                )));
            }

            content.push(current_char);

            self.position += 1;
        }

        Ok(String::from_utf8(content).unwrap())
    }

    fn get_element_content(&mut self, root: &ElementNode) -> Result<Vec<Node>, ParserError> {
        let mut nodes = Vec::<Node>::new();
        let mut open_tags = 0;

        while !self.eof() {
            match self.current_char() {
                LESS_THAN => {
                    let next_character = self.next_char();

                    if next_character == SLASH {
                        self.position += 2;
                    }

                    let tag_name = self.get_tag_name()?;

                    if next_character == SLASH {
                        if open_tags == 0 {
                            if tag_name != root.tag_name {
                                return Err(ParserError::UnexpectedToken(
                                    UnexpectedTokenError::new(
                                        &format!("</{}>", root.tag_name),
                                        &(self.current_char() as char).to_string(),
                                        self.position,
                                    ),
                                ));
                            } else {
                                return Ok(nodes);
                            }
                        }

                        open_tags -= 1;
                    } else {
                        open_tags += 1;
                    }

                    nodes.push(Node::Element(ElementNode {
                        tag_name,
                        // todo
                        attributes: ElementAttributes::new(),
                        children: ElementChildren::new(),
                    }));
                }
                WHITESPACE => self.skip_whitespaces(),
                // Text content
                _ => {
                    let last_node = nodes.last_mut();

                    match last_node {
                        Some(last_node) => match last_node {
                            Node::Text(_) => {
                                return Err(ParserError::Generic(GenericError::new(
                                    self.position,
                                    "Text nodes cannot have children",
                                )))
                            }
                            Node::Element(last_node) => {
                                last_node.children.push(Node::Text(TextNode {
                                    content: self.get_text_content()?.trim().to_owned(),
                                }))
                            }
                        },
                        None => nodes.push(Node::Text(TextNode {
                            content: self.get_text_content()?.trim().to_owned(),
                        })),
                    }
                }
            }
        }

        Err(ParserError::PrematureEndOfFile(
            PrematureEndOfFileError::new(self.position),
        ))
    }

    pub fn parse(&mut self) -> Result<Node, ParserError> {
        let current_char = self.current_char();

        if current_char != LESS_THAN {
            return Ok(Node::Text(TextNode {
                content: self.get_text_content()?,
            }));
        }

        self.position += 1;
        let tag_name = self.get_tag_name()?;

        let mut root_node = ElementNode {
            tag_name,
            attributes: ElementAttributes::new(),
            children: ElementChildren::new(),
        };

        let children = self.get_element_content(&root_node)?;

        root_node.children = children;

        Ok(Node::Element(root_node))
    }
}
