use std::fmt;

use javaparser::parse::tree::{CompilationUnit, CompilationUnitItem};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavaFunction {
    pub name: String,
    pub lines: (usize, usize),
    pub args: Vec<String>,
    pub body: String,
    pub class: Vec<JavaBlock>,
}

impl JavaFunction {
    pub fn new(
        name: String,
        lines: (usize, usize),
        args: Vec<String>,
        body: String,
        class: Vec<JavaBlock>,
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
        // TODO: sort top and bottom by line number
        write!(f, "{}", self.class.iter().map(|c| format!("{}\n", c.top)).collect::<Vec<String>>().join(""))?;
        write!(f, "{}", self.body)?;
        write!(f, "{}", self.class.iter().rev().map(|c| format!("\n{}", c.bottom)).collect::<Vec<String>>().join(""))?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavaBlock {
    pub name: String,
    pub line: (usize, usize),
    pub top: String,
    pub bottom: String,
    pub type_: JavaBlockType,
}
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum JavaBlockType {
    Class,
    Enum,
    Interface,
}

pub(crate) fn find_function_in_file(
    file_contents: &str,
    name: &str,
) -> Result<Vec<JavaFunction>, String> {
    let file = javaparser::parse::apply(file_contents, "<stdin>").map_err(|_| "Parse error")?;
    let parsed = file.unit.clone().items;
    println!("{:#?}", parsed);
    let mut functions = Vec::new();
    for unit in parsed {
        extract_methods_from_compilation_unit(&unit, name).map(|f| functions.push(f))?;
    }
    Err("Not implemented".to_string())
}

fn extract_methods_from_compilation_unit(
    unit: &CompilationUnitItem<'_>,
    name: &str,
) -> Result<Vec<JavaFunction>, String> {
    let mut methods = Vec::new();

        match unit {
        javaparser::parse::tree::CompilationUnitItem::Class(_) => todo!(),
        javaparser::parse::tree::CompilationUnitItem::Interface(_) => todo!(),
        javaparser::parse::tree::CompilationUnitItem::Annotation(_) => todo!(),
        javaparser::parse::tree::CompilationUnitItem::Enum(_) => todo!(),
    }
    Ok(methods)
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

    #[test]
    fn java_fn_print() {
        let java_class1 = JavaBlock {
            name: "Test".to_string(),
            line: (1, 1),
            top: "public class Test {".to_string(),
            bottom: "}".to_string(),
            type_: JavaBlockType::Class,
        };
        let java_class2 = JavaBlock {
            name: "Test2".to_string(),
            line: (1, 1),
            top: "    public class Test2 {".to_string(),
            bottom: "    }".to_string(),
            type_: JavaBlockType::Class,
        };
        let java_fn = JavaFunction::new(
            "main".to_string(),
            (1, 1),
            vec![],
            "        public static void main(String[] args) {
            System.out.println(\"Hello, World\");
        }".to_string(),
            vec![java_class1, java_class2],
        );
        println!("{}", java_fn);
    }
}
