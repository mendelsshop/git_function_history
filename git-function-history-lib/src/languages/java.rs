use std::fmt;

use javaparser::{analyze::definition::MethodDef, parse::tree::CompilationUnitItem};

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
                .map(|c| format!("{}\n...\n", c.top))
                .collect::<String>()
        )?;
        write!(f, "{}", self.body)?;
        write!(
            f,
            "{}",
            self.class
                .iter()
                .rev()
                .map(|c| format!("\n...\n{}", c.bottom))
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
) -> Result<Vec<JavaFunction>, &str> {
    let file = javaparser::parse::apply(file_contents, "<stdin>").map_err(|_| "Parse error")?;
    let parsed = file.unit.clone().items;
    let mut functions = Vec::new();
    for unit in parsed {
        extract_methods_from_compilation_unit(&unit, name).map(|f| functions.extend(f))?;
    }
    Ok(functions)
}

fn extract_methods_from_compilation_unit(
    unit: &CompilationUnitItem<'_>,
    name: &str,
) -> Result<Vec<JavaFunction>, &str> {
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
) -> Result<Vec<JavaFunction>, &str> {
    let mut methods = Vec::new();
    match item {
        javaparser::parse::tree::ClassBodyItem::Method(method) => {
            println!("{:#?}", method);
            println!("Found method: {}", method.name.fragment);
            if method.name.fragment == name {
                // let methdef = javaparser::analyze::build::method::build(method);
                // let def = javaparser::extract::Definition::Method(methdef);
                // println!("{:#?}", methdef.span_opt.map(|s| s.fragment));
                // println!("{:#?}", methdef);
                let args = vec![];
                // method
                // .parameters
                // .iter()
                // .map(|p| p.name.to_string())
                // .collect::<Vec<String>>();
                // let body = method.body.to_string();
                // let lines = (method.line, method.line + body.lines().count());
                // println!("{:#?}", method);
                // method.
                // to find the the bottom of the class see if block_opt is some then find the span of the last block find the first } after that and then find the line number of that
                let mut top = 0;
                method.modifiers.iter().for_each(|m| {
                    //match the modifier to extract the line number
                    match m {
                        javaparser::parse::tree::Modifier::Keyword(k) => {
                            top = k.name.line;
                        }
                        javaparser::parse::tree::Modifier::Annotated(a) => {
                            match a {
                                javaparser::parse::tree::Annotated::Normal(n) => {
                                    n.params.iter().for_each(|p| {
                                        if p.name.line > top {
                                            top = p.name.line;
                                        }
                                    });
                                }
                                javaparser::parse::tree::Annotated::Marker(m) => {
                                    // top = m.name.line;
                                }
                                javaparser::parse::tree::Annotated::Single(s) => {
                                    // top = s.name.line;
                                }
                            }
                        }
                    }
                });
                if top == 0 {
                    top = match method.return_type.span_opt() {
                        Some(s) => s.line,
                        None => return Err("could not find top of method")?,
                    }
                }
                let mut bottom = 0;
                // to find the top of the class find the first { before the method and then find the line number of that first check modifiers if not use return type

                if let Some(b) = &method.block_opt {
                    // find the last block and then find the line number of the first } after that
                }
                // if there is no block then find the line number of

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
        javaparser::parse::tree::ClassBodyItem::Constructor(_)
        | javaparser::parse::tree::ClassBodyItem::FieldDeclarators(_)
        | javaparser::parse::tree::ClassBodyItem::StaticInitializer(_) => {}
    }
    Ok(methods)
}

fn extract_methods_from_annotation_item(
    item: &javaparser::parse::tree::AnnotationBodyItem<'_>,
    name: &str,
) -> Result<Vec<JavaFunction>, &str> {
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
        javaparser::parse::tree::AnnotationBodyItem::Param(_)
        | javaparser::parse::tree::AnnotationBodyItem::FieldDeclarators(_) => {}
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
                void main(String[] args) {
                    // System.out.println("Hello, World");
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
            top: "1: public class Test {".to_string(),
            bottom: "30: }".to_string(),
            type_: JavaBlockType::Class,
        };
        let java_class2 = JavaBlock {
            name: "Test2".to_string(),
            line: (1, 1),
            top: "3:    public class Test2 {".to_string(),
            bottom: "28:    }".to_string(),
            type_: JavaBlockType::Class,
        };
        let java_fn = JavaFunction::new(
            "main".to_string(),
            (1, 1),
            vec![],
            "5:        public static void main(String[] args) {
7:            System.out.println(\"Hello, World\");
8:        }"
                .to_string(),
            vec![java_class1, java_class2],
        );
        println!("{}", java_fn);
    }
}
