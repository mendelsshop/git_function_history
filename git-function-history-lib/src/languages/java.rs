use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavaFunction {
    pub name: String,
    pub lines: (usize, usize),
    pub args: Vec<String>,
    pub body: String,
    pub class: JavaClass,
}

impl JavaFunction {
    pub fn new(
        name: String,
        lines: (usize, usize),
        args: Vec<String>,
        body: String,
        class: JavaClass,
    ) -> Self {
        Self {
            name,
            lines,
            args,
            body,
            class,
        }
    }
}

impl fmt::Display for JavaFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.class.top)?;
        write!(f, "{}", self.body)?;
        write!(f, "{}", self.class.bottom)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavaClass {
    pub name: String,
    pub line: (usize, usize),
    pub top: String,
    pub bottom: String,
}

pub(crate) fn find_function_in_file(
    file_contents: &str,
    name: &str,
) -> Result<JavaFunction, String> {
    let file = javaparser::parse::apply(file_contents, "<stdin>").map_err(|_| "Parse error")?;
    let parsed = file.unit.clone();
    println!("{:#?}", file);
    Err("Not implemented".to_string())
}

#[cfg(test)]
mod java_test {
    use super::*;

    #[test]
    fn java() {
        let file_contents = r#"
            public class Test {
                public static void main(String[] args) {
                    System.out.println("Hello, World");
                }
            }
        "#;
        let function = find_function_in_file(file_contents, "main").unwrap();

        }
    }