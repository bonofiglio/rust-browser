use super::dom::{ElementAttributes, ElementChildren, ElementNode, Node, TextNode};

const GREATER_THAN: u8 = 0x003E;
const LESS_THAN: u8 = 0x003C;
const WHITESPACE: u8 = 0x0020;
const SLASH: u8 = 0x002F;
const QUOTE: u8 = 0x0022;

#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken(UnexpectedTokenError),
    PrematureEndOfFile(PrematureEndOfFileError),
    Generic(GenericError),
    InvalidIdentifier(InvalidIdentifierError),
    InvalidAttributeValue(InvalidAttributeValueError),
}

#[derive(Debug)]
pub struct InvalidAttributeValueError {
    pub value: String,
}

impl InvalidAttributeValueError {
    pub fn new(value: &str) -> InvalidAttributeValueError {
        InvalidAttributeValueError {
            value: InvalidAttributeValueError::build_error_message(value),
        }
    }

    fn build_error_message(value: &str) -> String {
        format!("Invalid attribute value \"{}\"", value)
    }
}

#[derive(Debug)]
pub struct InvalidIdentifierError {
    pub identifier: String,
}

impl InvalidIdentifierError {
    pub fn new(identifier: &str) -> InvalidIdentifierError {
        InvalidIdentifierError {
            identifier: InvalidIdentifierError::build_error_message(identifier),
        }
    }

    fn build_error_message(identifier: &str) -> String {
        format!("Invalid identifier \"{}\"", identifier)
    }
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

    fn validate_identifier(tag_name: &str) -> bool {
        for char in tag_name.chars() {
            if !char.is_alphanumeric() {
                return false;
            }
        }

        return true;
    }

    fn validate_attribute_value(value: &str) -> bool {
        let chars = value.as_bytes();

        if chars[0] != QUOTE || chars[value.len() - 1] != QUOTE {
            return false;
        }

        true
    }

    fn strip_attribute_value_quotes(value: &str) -> String {
        let chars = value.as_bytes();

        String::from_utf8(chars[1..chars.len() - 1].to_vec()).unwrap()
    }

    fn parse_attribute_section(section: &str) -> Result<(String, String), ParserError> {
        match section.split_once("=") {
            Some((key, value)) => {
                if !Parser::validate_identifier(key) {
                    return Err(ParserError::InvalidIdentifier(InvalidIdentifierError::new(
                        key,
                    )));
                }

                if !Parser::validate_attribute_value(value) {
                    return Err(ParserError::InvalidAttributeValue(
                        InvalidAttributeValueError::new(value),
                    ));
                }

                return Ok((key.to_owned(), Parser::strip_attribute_value_quotes(value)));
            }
            None => {
                return Ok((section.to_owned(), "".to_owned()));
            }
        }
    }

    fn parse_attributes(attributes_string: &str) -> Result<ElementAttributes, ParserError> {
        let mut parsed_attributes = ElementAttributes::new();
        let attribute_sections = attributes_string.split(" ");

        for section in attribute_sections {
            let (key, value) = Parser::parse_attribute_section(section)?;

            if parsed_attributes.contains_key(&key) {
                continue;
            }

            parsed_attributes.insert(key.to_owned(), value.to_owned());
        }

        Ok(parsed_attributes)
    }

    fn get_tag_data(&mut self) -> Result<(String, ElementAttributes), ParserError> {
        let mut tag = String::new();

        while !self.eof() && self.current_char() != GREATER_THAN {
            let current_char = self.current_char();

            tag.push(current_char as char);
            self.position += 1;
        }

        // Reached eof before closing tag
        if self.current_char() != GREATER_THAN {
            return Err(ParserError::PrematureEndOfFile(
                PrematureEndOfFileError::new(self.position),
            ));
        }

        self.position += 1;

        let split_tag = tag.split_once(" ");

        match split_tag {
            Some((tag_name, attributes_string)) => {
                if !Parser::validate_identifier(&tag_name) {
                    return Err(ParserError::InvalidIdentifier(InvalidIdentifierError::new(
                        &tag,
                    )));
                }

                let attributes = Parser::parse_attributes(attributes_string)?;

                return Ok((tag_name.to_owned(), attributes));
            }
            None => {
                if !Parser::validate_identifier(&tag) {
                    return Err(ParserError::InvalidIdentifier(InvalidIdentifierError::new(
                        &tag,
                    )));
                }
                return Ok((tag, ElementAttributes::new()));
            }
        }
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

                    let (tag_name, attributes) = self.get_tag_data()?;

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
                        attributes,
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
        let (tag_name, attributes) = self.get_tag_data()?;

        let mut root_node = ElementNode {
            tag_name,
            attributes,
            children: ElementChildren::new(),
        };

        let children = self.get_element_content(&root_node)?;

        root_node.children = children;

        Ok(Node::Element(root_node))
    }
}
