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
        write!(
            f,
            "{}",
            self.class
                .iter()
                .map(|c| format!("{}\n", c.top))
                .collect::<String>()
        )?;
        write!(f, "{}", self.body)?;
        write!(
            f,
            "{}",
            self.class
                .iter()
                .rev()
                .map(|c| format!("\n{}", c.bottom))
                .collect::<String>()
        )?;
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
    // println!("{:#?}", parsed);
    let mut functions = Vec::new();
    for unit in parsed {
        extract_methods_from_compilation_unit(&unit, name).map(|f| functions.extend(f))?;
    }
    Ok(functions)
}

fn extract_methods_from_compilation_unit(
    unit: &CompilationUnitItem<'_>,
    name: &str,
) -> Result<Vec<JavaFunction>, String> {
    // recursively search for items with type Method

    let mut methods = Vec::new();

    match unit {
        javaparser::parse::tree::CompilationUnitItem::Class(class) => {
            let mut class_methods = Vec::new();
            for item in &class.body.items {
                extract_methods_from_class_item(item, name).map(|f| class_methods.extend(f))?;
            }
            methods.extend(class_methods);
        }
        javaparser::parse::tree::CompilationUnitItem::Interface(interface) => {
            let mut interface_methods = Vec::new();
            for item in &interface.body.items {
                extract_methods_from_class_item(item, name).map(|f| interface_methods.extend(f))?;
            }
            methods.extend(interface_methods);
        }
        javaparser::parse::tree::CompilationUnitItem::Enum(enum_) => {
            let mut enum_methods = Vec::new();
            if let Some(enum_body) = &enum_.body_opt {
                for item in &enum_body.items {
                    extract_methods_from_class_item(item, name).map(|f| enum_methods.extend(f))?;
                }
            }
            methods.extend(enum_methods);
        }
        javaparser::parse::tree::CompilationUnitItem::Annotation(annotation) => {
            let mut annotation_methods = Vec::new();
            for item in &annotation.body.items {
                extract_methods_from_annotation_item(item, name)
                    .map(|f| annotation_methods.extend(f))?;
            }
            methods.extend(annotation_methods);
        }
    }
    Ok(methods)
}

fn extract_methods_from_class_item(
    item: &javaparser::parse::tree::ClassBodyItem<'_>,
    name: &str,
) -> Result<Vec<JavaFunction>, String> {
    let mut methods = Vec::new();
    match item {
        javaparser::parse::tree::ClassBodyItem::Method(method) => {
            // println!("{:#?}", method);
            if method.name.fragment == name {
                let args = vec![];
                // method
                // .parameters
                // .iter()
                // .map(|p| p.name.to_string())
                // .collect::<Vec<String>>();
                // let body = method.body.to_string();
                // let lines = (method.line, method.line + body.lines().count());
                let class = Vec::new();
                methods.push(JavaFunction::new(
                    "test".to_string(),
                    (0, 0),
                    args,
                    "test".to_string(),
                    class,
                ));
            }
        }
        javaparser::parse::tree::ClassBodyItem::Class(class) => {
            let mut class_methods = Vec::new();
            for item in &class.body.items {
                extract_methods_from_class_item(item, name).map(|f| class_methods.extend(f))?;
            }
            methods.extend(class_methods);
        }
        javaparser::parse::tree::ClassBodyItem::Interface(interface) => {
            let mut interface_methods = Vec::new();
            for item in &interface.body.items {
                extract_methods_from_class_item(item, name).map(|f| interface_methods.extend(f))?;
            }
            methods.extend(interface_methods);
        }
        javaparser::parse::tree::ClassBodyItem::Enum(enum_) => {
            let mut enum_methods = Vec::new();
            if let Some(enum_body) = &enum_.body_opt {
                for item in &enum_body.items {
                    extract_methods_from_class_item(item, name).map(|f| enum_methods.extend(f))?;
                }
            }
            methods.extend(enum_methods);
        }
        javaparser::parse::tree::ClassBodyItem::Annotation(annotation) => {
            let mut annotation_methods = Vec::new();
            for item in &annotation.body.items {
                extract_methods_from_annotation_item(item, name)
                    .map(|f| annotation_methods.extend(f))?;
            }
            methods.extend(annotation_methods);
        }
        javaparser::parse::tree::ClassBodyItem::Constructor(constructor) => {}
        javaparser::parse::tree::ClassBodyItem::FieldDeclarators(field_declarators) => {}
        javaparser::parse::tree::ClassBodyItem::StaticInitializer(static_initializer) => {}
    }
    Ok(methods)
}

fn extract_methods_from_annotation_item(
    item: &javaparser::parse::tree::AnnotationBodyItem<'_>,
    name: &str,
) -> Result<Vec<JavaFunction>, String> {
    let mut methods = Vec::new();
    match item {
        javaparser::parse::tree::AnnotationBodyItem::Annotation(annotation) => {
            let mut annotation_methods = Vec::new();
            for item in &annotation.body.items {
                extract_methods_from_annotation_item(item, name)
                    .map(|f| annotation_methods.extend(f))?;
            }
            methods.extend(annotation_methods);
        }
        javaparser::parse::tree::AnnotationBodyItem::FieldDeclarators(field_declarators) => {}
        javaparser::parse::tree::AnnotationBodyItem::Class(class) => {
            let mut class_methods = Vec::new();
            for item in &class.body.items {
                extract_methods_from_class_item(item, name).map(|f| class_methods.extend(f))?;
            }
            methods.extend(class_methods);
        }
        javaparser::parse::tree::AnnotationBodyItem::Interface(interface) => {
            let mut interface_methods = Vec::new();
            for item in &interface.body.items {
                extract_methods_from_class_item(item, name).map(|f| interface_methods.extend(f))?;
            }
            methods.extend(interface_methods);
        }
        javaparser::parse::tree::AnnotationBodyItem::Enum(enum_) => {
            let mut enum_methods = Vec::new();
            if let Some(enum_body) = &enum_.body_opt {
                for item in &enum_body.items {
                    extract_methods_from_class_item(item, name).map(|f| enum_methods.extend(f))?;
                }
            }
            methods.extend(enum_methods);
        }
        javaparser::parse::tree::AnnotationBodyItem::Param(param) => {}
    }
    Ok(methods)
}

#[cfg(test)]
mod java_test {
    use super::*;

    #[test]
    fn java() {
        let file_contents = r#"
        @Company
            public class Test {
                public static void main(String[] args) {
                    System.out.println("Hello, World");
                }
            }
        "#;
        let function = find_function_in_file(file_contents, "main").unwrap();
        println!("{:#?}", function);
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
        }"
            .to_string(),
            vec![java_class1, java_class2],
        );
        println!("{}", java_fn);
    }
}
