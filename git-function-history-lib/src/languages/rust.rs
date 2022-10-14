use std::{collections::HashMap, error::Error, fmt};

use ra_ap_syntax::{
    ast::{self, HasDocComments, HasGenericParams, HasName},
    AstNode, SourceFile, SyntaxKind,
};

use crate::impl_function_trait;

use super::FunctionTrait;

/// This holds the information about a single  function each commit will have multiple of these.
#[derive(Debug, Clone)]
pub struct RustFunction {
    pub(crate) name: String,
    /// The actual code of the function
    pub(crate) body: String,
    /// is the function in a block ie `impl` `trait` etc
    pub(crate) block: Option<Block>,
    /// optional parent functions
    pub(crate) function: Vec<FunctionBlock>,
    /// The line number the function starts and ends on
    pub(crate) lines: (usize, usize),
    /// The lifetime of the function
    pub(crate) lifetime: Vec<String>,
    /// The generic types of the function
    pub(crate) generics: Vec<String>,
    /// The arguments of the function
    pub(crate) arguments: HashMap<String, String>,
    /// The return type of the function
    pub(crate) return_type: Option<String>,
    /// The functions atrributes
    pub(crate) attributes: Vec<String>,
    /// the functions doc comments
    pub(crate) doc_comments: Vec<String>,
}

impl RustFunction {
    /// get the parent functions
    pub fn get_parent_function(&self) -> Vec<FunctionBlock> {
        self.function.clone()
    }

    /// get the block of the function
    pub fn get_block(&self) -> Option<Block> {
        self.block.clone()
    }
}

impl fmt::Display for RustFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.block {
            None => {}
            Some(block) => write!(f, "{}\n...\n", block.top)?,
        };
        for i in &self.function {
            write!(f, "{}\n...\n", i.top)?;
        }
        write!(f, "{}", self.body)?;
        for i in &self.function {
            write!(f, "\n...\n{}", i.bottom)?;
        }
        match &self.block {
            None => {}
            Some(block) => write!(f, "\n...{}", block.bottom)?,
        };
        Ok(())
    }
}

/// This is used for the functions that are being looked up themeselves but store an outer function that may aontains a function that is being looked up.
#[derive(Debug, Clone)]
pub struct FunctionBlock {
    /// The name of the function (parent function)
    pub(crate) name: String,
    /// what the signature of the function is
    pub(crate) top: String,
    /// what the last line of the function is
    pub(crate) bottom: String,
    /// The line number the function starts and ends on
    pub(crate) lines: (usize, usize),
    /// The lifetime of the function
    pub(crate) lifetime: Vec<String>,
    /// The generic types of the function
    pub(crate) generics: Vec<String>,
    /// The arguments of the function
    pub(crate) arguments: HashMap<String, String>,
    /// The return type of the function
    pub(crate) return_type: Option<String>,
    /// the function atrributes
    pub(crate) attributes: Vec<String>,
    /// the functions doc comments
    pub(crate) doc_comments: Vec<String>,
}

impl FunctionBlock {
    /// get the metadata for this block ie the name of the block, the type of block, the line number the block starts and ends
    pub fn get_metadata(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("name".to_string(), self.name.clone());
        map.insert("lines".to_string(), format!("{:?}", self.lines));
        map.insert("signature".to_string(), self.top.clone());
        map.insert("bottom".to_string(), self.bottom.clone());
        map.insert("generics".to_string(), self.generics.join(","));
        map.insert(
            "arguments".to_string(),
            self.arguments
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<String>>()
                .join(","),
        );
        map.insert("lifetime generics".to_string(), self.lifetime.join(","));
        map.insert("attributes".to_string(), self.attributes.join(","));
        map.insert("doc comments".to_string(), self.doc_comments.join(","));
        match &self.return_type {
            None => {}
            Some(return_type) => {
                map.insert("return type".to_string(), return_type.clone());
            }
        };
        map
    }
}

/// This holds information about when a function is in an impl/trait/extern block
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// The name of the block ie for `impl` it would be the type were impling for
    pub(crate) name: Option<String>,
    /// The signature of the block
    pub(crate) top: String,
    /// The last line of the block
    pub(crate) bottom: String,
    /// the type of block ie `impl` `trait` `extern`
    pub(crate) block_type: BlockType,
    /// The line number the function starts and ends on
    pub(crate) lines: (usize, usize),
    /// The lifetime of the function
    pub(crate) lifetime: Vec<String>,
    /// The generic types of the function
    pub(crate) generics: Vec<String>,
    /// The blocks atrributes
    pub(crate) attributes: Vec<String>,
    /// the blocks doc comments
    pub(crate) doc_comments: Vec<String>,
}

impl Block {
    /// get the metadata for this block ie the name of the block, the type of block, the line number the block starts and ends
    pub fn get_metadata(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Some(name) = &self.name {
            map.insert("name".to_string(), name.to_string());
        }
        map.insert("block".to_string(), format!("{}", self.block_type));
        map.insert("lines".to_string(), format!("{:?}", self.lines));
        map.insert("signature".to_string(), self.top.clone());
        map.insert("bottom".to_string(), self.bottom.clone());
        map.insert("generics".to_string(), self.generics.join(","));
        map.insert("lifetime generics".to_string(), self.lifetime.join(","));
        map.insert("attributes".to_string(), self.attributes.join(","));
        map.insert("doc comments".to_string(), self.doc_comments.join(","));
        map
    }
}

/// This enum is used when filtering commit history only for let say impl and not externs or traits
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum BlockType {
    /// This is for `impl` blocks
    Impl,
    /// This is for `trait` blocks
    Extern,
    /// This is for `extern` blocks
    Trait,
    /// This is for code that gets labeled as a block but `get_function_history` can't find a block type
    Unknown,
}

impl BlockType {
    /// This is used to get the name of the block type from a string
    pub fn from_string(s: &str) -> Self {
        match s {
            "impl" => Self::Impl,
            "extern" => Self::Extern,
            "trait" => Self::Trait,
            _ => Self::Unknown,
        }
    }
}

impl fmt::Display for BlockType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Impl => write!(f, "impl"),
            Self::Extern => write!(f, "extern"),
            Self::Trait => write!(f, "trait"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

#[allow(clippy::too_many_lines)]
// TODO: split this function into smaller functions
pub(crate) fn find_function_in_file(
    file_contents: &str,
    name: &str,
) -> Result<Vec<RustFunction>, Box<dyn Error>> {
    let mut functions = Vec::new();
    get_function_asts(name, file_contents, &mut functions);
    let mut starts = file_contents
        .match_indices('\n')
        .map(|x| x.0)
        .collect::<Vec<_>>();
    starts.push(0);
    starts.sort_unstable();
    let map = starts
        .iter()
        .enumerate()
        .collect::<HashMap<usize, &usize>>();
    let mut hist = Vec::new();
    for f in &functions {
        let stuff = get_stuff(f, file_contents, &map);
        let generics = get_genrerics_and_lifetime(f);
        let mut parent = f.syntax().parent();
        let mut parent_fn: Vec<FunctionBlock> = Vec::new();
        let mut parent_block = None;
        while let Some(p) = parent.into_iter().next() {
            if p.kind() == SyntaxKind::SOURCE_FILE {
                break;
            }
            ast::Fn::cast(p.clone()).map_or_else(
                || {
                    if let Some(block) = ast::Impl::cast(p.clone()) {
                        let attr = get_doc_comments_and_attrs(&block);
                        let stuff = get_stuff(&block, file_contents, &map);
                        let generics = get_genrerics_and_lifetime(&block);
                        parent_block = Some(Block {
                            name: block.self_ty().map(|ty| ty.to_string()),
                            lifetime: generics.1,
                            generics: generics.0,
                            top: stuff.1 .0,
                            bottom: stuff.1 .1,
                            block_type: BlockType::Impl,
                            lines: (stuff.0 .0, stuff.0 .1),
                            attributes: attr.1,
                            doc_comments: attr.0,
                        });
                    } else if let Some(block) = ast::Trait::cast(p.clone()) {
                        let attr = get_doc_comments_and_attrs(&block);
                        let stuff = get_stuff(&block, file_contents, &map);
                        let generics = get_genrerics_and_lifetime(&block);
                        parent_block = Some(Block {
                            name: block.name().map(|ty| ty.to_string()),
                            lifetime: generics.1,
                            generics: generics.0,
                            top: stuff.1 .0,
                            bottom: stuff.1 .1,
                            block_type: BlockType::Trait,
                            lines: (stuff.0 .0, stuff.0 .1),
                            attributes: attr.1,
                            doc_comments: attr.0,
                        });
                    } else if let Some(block) = ast::ExternBlock::cast(p.clone()) {
                        let attr = get_doc_comments_and_attrs(&block);
                        let stuff = get_stuff(&block, file_contents, &map);
                        parent_block = Some(Block {
                            name: block.abi().map(|ty| ty.to_string()),
                            lifetime: Vec::new(),
                            generics: Vec::new(),
                            top: stuff.1 .0,
                            bottom: stuff.1 .1,
                            block_type: BlockType::Extern,
                            lines: (stuff.0 .0, stuff.0 .1),
                            attributes: attr.1,
                            doc_comments: attr.0,
                        });
                    }
                },
                |function| {
                    let stuff = get_stuff(&function, file_contents, &map);
                    let generics = get_genrerics_and_lifetime(&function);
                    let attr = get_doc_comments_and_attrs(&function);
                    parent_fn.push(FunctionBlock {
                        name: function.name().unwrap().to_string(),
                        lifetime: generics.1,
                        generics: generics.0,
                        top: stuff.1 .0,
                        bottom: stuff.1 .1,
                        lines: (stuff.0 .0, stuff.0 .1),
                        return_type: function.ret_type().map(|ty| ty.to_string()),
                        arguments: match f.param_list() {
                            Some(args) => args
                                .params()
                                .filter_map(|arg| {
                                    arg.to_string()
                                        .rsplit_once(": ")
                                        .map(|x| (x.0.to_string(), x.1.to_string()))
                                })
                                .collect::<HashMap<String, String>>(),
                            None => HashMap::new(),
                        },
                        attributes: attr.1,
                        doc_comments: attr.0,
                    });
                },
            );
            parent = p.parent();
        }
        let attr = get_doc_comments_and_attrs(f);
        let mut start = stuff.0 .0;
        let bb = match map[&start] {
            0 => 0,
            x => x + 1,
        };
        let contents: String = file_contents[bb..f.syntax().text_range().end().into()]
            .to_string()
            .lines()
            .map(|l| {
                start += 1;
                format!("{}: {}\n", start, l,)
            })
            .collect();
        let body = contents.trim_end().to_string();
        let function = RustFunction {
            name: f.name().unwrap().to_string(),
            body,
            block: parent_block,
            function: parent_fn,
            return_type: f.ret_type().map(|ty| ty.to_string()),
            arguments: match f.param_list() {
                Some(args) => args
                    .params()
                    .filter_map(|arg| {
                        arg.to_string()
                            .rsplit_once(": ")
                            .map(|x| (x.0.to_string(), x.1.to_string()))
                    })
                    .collect::<HashMap<String, String>>(),
                None => HashMap::new(),
            },
            lifetime: generics.1,
            generics: generics.0,
            lines: (stuff.0 .0, stuff.0 .1),
            attributes: attr.1,
            doc_comments: attr.0,
        };
        hist.push(function);
    }
    if hist.is_empty() {
        Err("no function found")?;
    }
    Ok(hist)
}
#[inline]
fn get_function_asts(name: &str, file: &str, functions: &mut Vec<ast::Fn>) {
    let parsed_file = SourceFile::parse(file).tree();
    parsed_file
        .syntax()
        .descendants()
        .filter_map(ast::Fn::cast)
        .filter(|function| function.name().unwrap().text() == name)
        .for_each(|function| functions.push(function));
}
#[inline]
fn get_stuff<T: AstNode>(
    block: &T,
    file: &str,
    map: &HashMap<usize, &usize>,
) -> ((usize, usize), (String, String), (usize, usize)) {
    let start = block.syntax().text_range().start();
    let end = block.syntax().text_range().end();
    // get the start and end lines
    let mut found_start_brace = 0;
    let mut end_line = 0;
    let mut starts = 0;
    let mut start_line = 0;
    // TODO: combine these loops
    for (i, line) in file.chars().enumerate() {
        if line == '\n' {
            if usize::from(start) < i {
                starts = i;
                break;
            }
            start_line += 1;
        }
    }
    for (i, line) in file.chars().enumerate() {
        if line == '\n' {
            if usize::from(end) < i {
                break;
            }
            end_line += 1;
        }
        if line == '{' && found_start_brace == 0 && usize::from(start) < i {
            found_start_brace = i;
        }
    }
    if found_start_brace == 0 {
        found_start_brace = usize::from(start);
    }
    let start = map[&start_line];
    let mut start_lines = start_line;
    let mut content: String = file[(*start)..=found_start_brace].to_string();
    if &content[..1] == "\n" {
        content = content[1..].to_string();
    }
    (
        (start_line, end_line),
        (
            content
                .lines()
                .map(|l| {
                    start_lines += 1;
                    format!("{}: {}\n", start_lines, l,)
                })
                .collect::<String>()
                .trim_end()
                .to_string(),
            format!(
                "\n{}: {}",
                end_line,
                file.lines()
                    .nth(if end_line == file.lines().count() - 1 {
                        end_line
                    } else {
                        end_line - 1
                    })
                    .unwrap_or("")
            ),
        ),
        (starts, end_line),
    )
}
#[inline]
fn get_genrerics_and_lifetime<T: HasGenericParams>(block: &T) -> (Vec<String>, Vec<String>) {
    // TODO: map trait bounds from where clauses to the generics and also use type_or_const_params
    match block.generic_param_list() {
        None => (vec![], vec![]),
        Some(gt) => (
            gt.generic_params()
                .map(|gt| gt.to_string())
                .collect::<Vec<String>>(),
            gt.lifetime_params()
                .map(|lt| lt.to_string())
                .collect::<Vec<String>>(),
        ),
    }
}
#[inline]
fn get_doc_comments_and_attrs<T: HasDocComments>(block: &T) -> (Vec<String>, Vec<String>) {
    (
        block
            .doc_comments()
            .map(|c| c.to_string())
            .collect::<Vec<String>>(),
        block
            .attrs()
            .map(|c| c.to_string())
            .collect::<Vec<String>>(),
    )
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    /// when you want to filter by function that are in a specific block (impl, trait, extern)
    FunctionInBlock(BlockType),
    /// when you want filter by a function that has a parent function of a specific name
    FunctionWithParent(String),
    /// when you want to filter by a function that has a has a specific return type
    FunctionWithReturnType(String),
    /// when you want to filter by a function that has a specific parameter type
    FunctionWithParameterType(String),
    /// when you want to filter by a function that has a specific parameter name
    FunctionWithParameterName(String),
    /// when you want to filter by a function that has a specific lifetime
    FunctionWithLifetime(String),
    /// when you want to filter by a function that has a specific generic with name
    FunctionWithGeneric(String),
    /// when you want to filter by a function that has a specific attribute
    FunctionWithAttribute(String),
}

impl Filter {
    pub fn matches(&self, function: &RustFunction) -> bool {
        match self {
            Self::FunctionInBlock(block_type) => function
                .block
                .as_ref()
                .map_or(false, |block| block.block_type == *block_type),
            Self::FunctionWithParent(parent) => function.function.iter().any(|f| f.name == *parent),
            Self::FunctionWithReturnType(return_type) => {
                function.return_type == Some(return_type.to_string())
            }
            Self::FunctionWithParameterType(parameter_type) => {
                function.arguments.values().any(|x| x == parameter_type)
            }
            Self::FunctionWithParameterName(parameter_name) => {
                function.arguments.keys().any(|x| x == parameter_name)
            }
            Self::FunctionWithLifetime(lifetime) => function.lifetime.contains(lifetime),
            Self::FunctionWithGeneric(generic) => function.generics.contains(generic),
            Self::FunctionWithAttribute(attribute) => function.attributes.contains(attribute),
        }
    }
}

impl FunctionTrait for RustFunction {
    fn get_tops(&self) -> Vec<String> {
        let mut tops = Vec::new();
        match &self.block {
            Some(block) => {
                tops.push(block.top.clone());
            }
            None => {}
        }
        for parent in &self.function {
            tops.push(parent.top.clone());
        }
        tops
    }

    fn get_total_lines(&self) -> (usize, usize) {
        match &self.block {
            Some(block) => (block.lines.0, block.lines.1),
            None => {
                let mut start = self.lines.0;
                let mut end = self.lines.1;
                for parent in &self.function {
                    if parent.lines.0 < start {
                        start = parent.lines.0;
                        end = parent.lines.1;
                    }
                }
                (start, end)
            }
        }
    }

    fn get_bottoms(&self) -> Vec<String> {
        let mut bottoms = Vec::new();
        match &self.block {
            Some(block) => {
                bottoms.push(block.bottom.clone());
            }
            None => {}
        }
        for parent in &self.function {
            bottoms.push(parent.bottom.clone());
        }
        bottoms
    }

    impl_function_trait!(RustFunction);
}
