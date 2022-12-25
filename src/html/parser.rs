use super::dom::{ElementAttributes, ElementChildren, ElementNode, Node, TextNode};

const GREATER_THAN: u8 = 0x003E;
const LESS_THAN: u8 = 0x003C;
const WHITESPACE: u8 = 0x0020;
const SLASH: u8 = 0x002F;
const UPPER_CASE_A: u8 = 0x0041;
const UPPER_CASE_Z: u8 = 0x005A;
const LOWER_CASE_A: u8 = 0x0061;
const LOWER_CASE_Z: u8 = 0x007A;
const ZERO: u8 = 0x0030;
const NINE: u8 = 0x0039;

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

    fn next_char(&self) -> Result<u8, ParserError> {
        if self.position + 1 >= self.input.len() {
            return Err(ParserError::Generic(GenericError::new(
                self.position,
                "index out of bounds",
            )));
        }

        Ok(self.input[self.position + 1])
    }

    fn get_tag_name(&mut self) -> Result<String, ParserError> {
        let mut tag_name = Vec::<u8>::with_capacity(1);

        while !self.eof() && self.current_char() != GREATER_THAN {
            let current_char = self.current_char();

            match current_char {
                UPPER_CASE_A..=UPPER_CASE_Z | LOWER_CASE_A..=LOWER_CASE_Z | ZERO..=NINE => {
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

        while !self.eof() {
            match self.current_char() {
                LESS_THAN => {
                    let next_character = self.next_char()?;

                    if next_character == SLASH {
                        self.position += 1;
                    }
                    self.position += 1;

                    let tag_name = self.get_tag_name()?;

                    if next_character == SLASH {
                        if tag_name != root.tag_name {
                            return Err(ParserError::UnexpectedToken(UnexpectedTokenError::new(
                                &format!("</{}>", root.tag_name),
                                &(self.current_char() as char).to_string(),
                                self.position,
                            )));
                        } else {
                            return Ok(nodes);
                        }
                    }

                    let mut node = ElementNode {
                        tag_name,
                        // todo
                        attributes: ElementAttributes::new(),
                        children: ElementChildren::new(),
                    };

                    node.children = self.get_element_content(&node)?;

                    nodes.push(Node::Element(node));
                }
                WHITESPACE => self.skip_whitespaces(),
                // Text content
                _ => nodes.push(Node::Text(TextNode {
                    content: self.get_text_content()?.trim().to_owned(),
                })),
            }
        }

        Err(ParserError::PrematureEndOfFile(
            PrematureEndOfFileError::new(self.position),
        ))
    }

    pub fn parse(&mut self) -> Result<Node, ParserError> {
        let current_char = self.current_char();

        if current_char != LESS_THAN {
            return Err(ParserError::UnexpectedToken(UnexpectedTokenError::new(
                "<",
                &(current_char as char).to_string(),
                self.position,
            )));
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
