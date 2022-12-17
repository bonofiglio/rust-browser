use html::parser::ParserError;

mod html;

fn main() -> Result<(), ParserError> {
    let mut parser = html::parser::Parser::new("<div>content</div>");

    let result = parser.parse()?;

    println!("{:?}", result);

    Ok(())
}
