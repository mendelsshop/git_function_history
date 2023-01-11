use umpl;

use std::{error::Error, fmt};

use crate::impl_function_trait;

use super::FunctionTrait;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UMPLFunction {
    pub(crate) lines: (usize, usize),
    pub(crate) name: String,
    pub(crate) body: String,
}

impl fmt::Display for UMPLFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl FunctionTrait for UMPLFunction {
    impl_function_trait!(UMPLFunction);

    fn get_total_lines(&self) -> (usize, usize) {
        todo!()
    }

    fn get_tops(&self) -> Vec<(String, usize)> {
        todo!()
    }

    fn get_bottoms(&self) -> Vec<(String, usize)> {
        todo!()
    }
}

pub(crate) fn find_function_in_file(
    file_contents: &str,
    name: &str,
) -> Result<Vec<UMPLFunction>, Box<dyn Error>> {
    // parse the file contents
    let lexed = umpl::lexer::Lexer::new(file_contents.to_string());
    let tokens = lexed.scan_tokens();
    let mut parsed = umpl::parser::Parser::new(tokens);
    let ast = parsed.parse();
    for node in ast {}
    Err("")?
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UMPLFilter {}

impl UMPLFilter {
    pub fn matches(&self, function: &UMPLFunction) -> bool {
        false
    }
}
