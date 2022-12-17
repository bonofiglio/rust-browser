use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Node {
    Element(ElementNode),
    Text(TextNode),
}

#[derive(Clone, Debug)]
pub struct TextNode {
    pub content: String,
}

#[derive(Clone, Debug)]
pub struct ElementNode {
    pub tag_name: String,
    pub attributes: ElementAttributes,
    pub children: ElementChildren,
}

pub type ElementChildren = Vec<Node>;
pub type ElementAttributes = HashMap<String, String>;
