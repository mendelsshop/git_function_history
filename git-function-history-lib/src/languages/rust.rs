use std::{collections::HashMap, fmt};

use enum_stuff::enumstuff;
use ra_ap_syntax::{
    ast::{self, Fn, HasDocComments, HasGenericParams, HasName},
    AstNode, SourceFile, SyntaxKind,
};

use crate::{impl_function_trait, UnwrapToError};

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
    pub(crate) function: Vec<RustParentFunction>,
    /// The line number the function starts and ends on
    pub(crate) lines: (usize, usize),
    /// The lifetime of the function
    pub(crate) lifetime: Vec<String>,
    /// The generic types of the function
    /// also includes lifetimes
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
    pub fn get_parent_function(&self) -> Vec<RustParentFunction> {
        self.function.clone()
    }

    /// get the block of the function
    pub fn get_block(&self) -> Option<Block> {
        self.block.clone()
    }
}

impl fmt::Display for RustFunction {
    /// don't use this for anything other than debugging the output is not guaranteed to be in the right order
    /// use `fmt::Display` for `RustFile` instead
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
            Some(block) => write!(f, "\n...\n{}", block.bottom)?,
        };
        Ok(())
    }
}

/// This is used for the functions that are being looked up themeselves but store an outer function that may aontains a function that is being looked up.
#[derive(Debug, Clone)]
pub struct RustParentFunction {
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
    /// also includes lifetimes
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

impl RustParentFunction {
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
                .map(|(k, v)| format!("{k}: {v}"))
                .collect::<Vec<String>>()
                .join(","),
        );
        map.insert("lifetime generics".to_string(), self.lifetime.join(","));
        map.insert("attributes".to_string(), self.attributes.join(","));
        map.insert("doc comments".to_string(), self.doc_comments.join(","));
        self.return_type.as_ref().map_or((), |return_type| {
            map.insert("return type".to_string(), return_type.clone());
        });
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
    /// also includes lifetimes
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
#[derive(Debug, PartialEq, Eq, Copy, Clone, enumstuff)]
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

// TODO: split this function into smaller functions
pub(crate) fn find_function_in_file(
    file_contents: &str,
    name: &str,
) -> Result<Vec<RustFunction>, String> {
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
        let stuff = get_stuff(f, file_contents, &map).unwrap_to_error("could not get endline")?;
        let generics = get_genrerics_and_lifetime(f);
        let mut parent = f.syntax().parent();
        let mut parent_fn: Vec<RustParentFunction> = Vec::new();
        let mut parent_block = None;
        while let Some(p) = parent.into_iter().next() {
            if p.kind() == SyntaxKind::SOURCE_FILE {
                break;
            }
            ast::Fn::cast(p.clone()).map_or_else(
                || {
                    if let Some(block) = ast::Impl::cast(p.clone()) {
                        let attr = get_doc_comments_and_attrs(&block);
                        let Some(stuff) = get_stuff(&block, file_contents, &map) else {
                            return;
                        };
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
                        let Some(stuff) = get_stuff(&block, file_contents, &map) else {
                            return;
                        };
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
                        if let Some(stuff) = stuff {
                            parent_block = Some(Block {
                                name: None,
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
                    }
                },
                |function: ast::Fn| {
                    let Some(stuff) = get_stuff(&function, file_contents, &map) else {
                        return;
                    };
                    let generics = get_genrerics_and_lifetime(&function);
                    let attr = get_doc_comments_and_attrs(&function);
                    let name = match function.name() {
                        Some(name) => name.to_string(),
                        None => return,
                    };
                    parent_fn.push(RustParentFunction {
                        name,
                        lifetime: generics.1,
                        generics: generics.0,
                        top: stuff.1 .0,
                        bottom: stuff.1 .1,
                        lines: (stuff.0 .0, stuff.0 .1),
                        return_type: get_ret_type(&function),
                        arguments: f.param_list().map_or_else(HashMap::new, |args| {
                            args.params()
                                .filter_map(|arg| {
                                    arg.to_string()
                                        .rsplit_once(": ")
                                        .map(|x| (x.0.to_string(), x.1.to_string()))
                                })
                                .collect::<HashMap<String, String>>()
                        }),
                        attributes: attr.1,
                        doc_comments: attr.0,
                    });
                },
            );
            parent = p.parent();
        }
        let attr = get_doc_comments_and_attrs(f);
        let start_line = stuff.0 .0;
        let start_idx = match map
            .get(&(start_line - 1))
            .unwrap_to_error("could not get start index for function based off line number")?
        {
            0 => 0,
            x => *x + 1,
        };
        let contents: String = file_contents
            .get(start_idx..f.syntax().text_range().end().into())
            .unwrap_to_error("could not function text based off of start and stop indexes")?
            .to_string();
        let body = super::make_lined(&contents, start_line);
        let function = RustFunction {
            name: match f.name() {
                Some(name) => name.to_string(),
                None => continue,
            },
            body,
            block: parent_block,
            function: parent_fn,
            return_type: get_ret_type(f),
            arguments: f.param_list().map_or_else(HashMap::new, |args| {
                args.params()
                    .filter_map(|arg| {
                        arg.to_string()
                            .rsplit_once(": ")
                            .map(|x| (x.0.to_string(), x.1.to_string()))
                    })
                    .collect::<HashMap<String, String>>()
            }),
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
        .filter(|function| function.name().map_or(false, |n| n.text() == name))
        .for_each(|function| functions.push(function));
}
#[inline]
fn get_stuff<T: AstNode>(
    block: &T,
    file: &str,
    map: &HashMap<usize, &usize>,
) -> Option<((usize, usize), (String, String))> {
    let start = block.syntax().text_range().start();
    let end: usize = block.syntax().text_range().end().into();
    // get the start and end lines
    let mut found_start_brace = 0;
    let index = super::turn_into_index(file).ok()?;
    let end_line = super::get_from_index(&index, end - 1)?;
    let start_line = super::get_from_index(&index, start.into())?;
    for (i, line) in file.chars().enumerate() {
        if line == '{' && found_start_brace == 0 && usize::from(start) < i {
            found_start_brace = i;
            break;
        }
    }
    if found_start_brace == 0 {
        found_start_brace = usize::from(start);
    }
    let start_idx = map.get(&(start_line - 1))?;
    let start_lines = start_line;
    let mut content: String = file.get((**start_idx)..=found_start_brace)?.to_string();
    if content.get(..1)? == "\n" {
        content = content.get(1..)?.to_string();
    }
    Some((
        (start_line, end_line),
        (
            super::make_lined(&content, start_lines),
            super::make_lined(file.lines().nth(end_line - 1).unwrap_or("}"), end_line),
        ),
    ))
}
#[inline]
fn get_genrerics_and_lifetime<T: HasGenericParams>(block: &T) -> (Vec<String>, Vec<String>) {
    // TODO: map trait bounds from where clauses to the generics so it will return a (HashMap<String, Vec<String>>, HashMap<String, Vec<String>>)
    // the key of each hashmap will be the name of the generic/lifetime and the values will be the trait bounds
    // and also use type_or_const_params
    block.generic_param_list().map_or_else(
        || (vec![], vec![]),
        |gt| {
            (
                gt.type_or_const_params()
                    .map(|gt| match gt {
                        ast::TypeOrConstParam::Type(ty) => ty.to_string(),
                        ast::TypeOrConstParam::Const(c) => c.to_string(),
                    })
                    .collect::<Vec<String>>(),
                gt.lifetime_params()
                    .map(|lt| lt.to_string())
                    .collect::<Vec<String>>(),
            )
        },
    )
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

fn get_ret_type(fns: &Fn) -> Option<String> {
    fns.ret_type()
        .and_then(|ret| ret.ty().map(|ty| ty.to_string()))
}
#[derive(Debug, Clone, PartialEq, Eq, enumstuff)]

/// filters for rust functions
pub enum RustFilter {
    /// when you want to filter by function that are in a specific block (impl, trait, extern)
    InBlock(BlockType),
    /// when you want filter by a function that has a parent function of a specific name
    #[enumstuff(skip)]
    HasParentFunction(String),
    /// when you want to filter by a function that has a has a specific return type
    #[enumstuff(skip)]
    HasReturnType(String),
    /// when you want to filter by a function that has a specific parameter type
    #[enumstuff(skip)]
    HasParameterType(String),
    /// when you want to filter by a function that has a specific parameter name
    #[enumstuff(skip)]
    HasParameterName(String),
    /// when you want to filter by a function that has a specific lifetime
    #[enumstuff(skip)]
    HasLifetime(String),
    /// when you want to filter by a function that has a specific generic with name
    #[enumstuff(skip)]
    HasGeneric(String),
    /// when you want to filter by a function that has a specific attribute
    #[enumstuff(skip)]
    HasAttribute(String),
    /// when you want to filter by a function that has or contains a specific doc comment
    #[enumstuff(skip)]
    HasDocComment(String),
    /// when you want to filter by a function that's block has a specific attribute
    #[enumstuff(skip)]
    BlockHasAttribute(String),
    /// when you want to filter by a function that's block has a specific doc comment
    #[enumstuff(skip)]
    BlockHasDocComment(String),
    /// when you want to filter by a function that's block has a specific lifetime
    #[enumstuff(skip)]
    BlockHasLifetime(String),
    /// when you want to filter by a function that's block has a specific generic with name
    #[enumstuff(skip)]
    BlockHasGeneric(String),
    /// when you want to filter by a function that's parent function has a specific attribute
    #[enumstuff(skip)]
    ParentFunctionHasAttribute(String),
    /// when you want to filter by a function that's parent function has a specific doc comment
    #[enumstuff(skip)]
    ParentFunctionHasDocComment(String),
    /// when you want to filter by a function that's parent function has a specific lifetime
    #[enumstuff(skip)]
    ParentFunctionHasLifetime(String),
    /// when you want to filter by a function that's parent function has a specific generic with name
    #[enumstuff(skip)]
    ParentFunctionHasGeneric(String),
    /// when you want to filter by a function that's parent function has a specific return type
    #[enumstuff(skip)]
    ParentFunctionHasReturnType(String),
    /// when you want to filter by a function that's parent function has a specific parameter type
    #[enumstuff(skip)]
    ParentFunctionHasParameterType(String),
    /// when you want to filter by a function that's parent function has a specific parameter name
    #[enumstuff(skip)]
    ParentFunctionHasParameterName(String),
}

impl RustFilter {
    /// checks if a function matches the filter
    pub fn matches(&self, function: &RustFunction) -> bool {
        match self {
            Self::InBlock(block_type) => function
                .block
                .as_ref()
                .map_or(false, |block| block.block_type == *block_type),
            Self::HasParentFunction(parent) => function.function.iter().any(|f| f.name == *parent),
            Self::HasReturnType(return_type) => {
                function.return_type == Some(return_type.to_string())
            }
            Self::HasParameterType(parameter_type) => {
                function.arguments.values().any(|x| x == parameter_type)
            }
            Self::HasParameterName(parameter_name) => {
                function.arguments.keys().any(|x| x == parameter_name)
            }
            Self::HasLifetime(lifetime) => function.lifetime.contains(lifetime),
            Self::HasGeneric(generic) => function.generics.contains(generic),
            Self::HasAttribute(attribute) => function.attributes.contains(attribute),
            Self::HasDocComment(comment) => {
                function
                    .doc_comments
                    .iter()
                    .filter(|doc| comment.contains(*doc))
                    .count()
                    > 0
            }
            Self::BlockHasAttribute(attribute) => function
                .block
                .as_ref()
                .map_or(false, |block| block.attributes.contains(attribute)),
            Self::BlockHasDocComment(comment) => function.block.as_ref().map_or(false, |block| {
                block
                    .doc_comments
                    .iter()
                    .filter(|doc| comment.contains(*doc))
                    .count()
                    > 0
            }),
            Self::BlockHasLifetime(lifetime) => function
                .block
                .as_ref()
                .map_or(false, |block| block.lifetime.contains(lifetime)),
            Self::BlockHasGeneric(generic) => function
                .block
                .as_ref()
                .map_or(false, |block| block.generics.contains(generic)),
            Self::ParentFunctionHasAttribute(attribute) => function
                .function
                .iter()
                .any(|f| f.attributes.contains(attribute)),
            Self::ParentFunctionHasDocComment(comment) => function.function.iter().any(|f| {
                f.doc_comments
                    .iter()
                    .filter(|doc| comment.contains(*doc))
                    .count()
                    > 0
            }),
            Self::ParentFunctionHasLifetime(lifetime) => function
                .function
                .iter()
                .any(|f| f.lifetime.contains(lifetime)),
            Self::ParentFunctionHasGeneric(generic) => function
                .function
                .iter()
                .any(|f| f.generics.contains(generic)),
            Self::ParentFunctionHasReturnType(return_type) => function
                .function
                .iter()
                .any(|f| f.return_type == Some(return_type.to_string())),
            Self::ParentFunctionHasParameterType(parameter_type) => function
                .function
                .iter()
                .any(|f| f.arguments.values().any(|x| x == parameter_type)),
            Self::ParentFunctionHasParameterName(parameter_name) => function
                .function
                .iter()
                .any(|f| f.arguments.keys().any(|x| x == parameter_name)),
        }
    }
}

impl FunctionTrait for RustFunction {
    fn get_tops(&self) -> Vec<(String, usize)> {
        let mut tops = Vec::new();
        self.block.as_ref().map_or((), |block| {
            tops.push((block.top.clone(), block.lines.0));
        });
        for parent in &self.function {
            tops.push((parent.top.clone(), parent.lines.0));
        }
        tops
    }

    fn get_bottoms(&self) -> Vec<(String, usize)> {
        let mut bottoms = Vec::new();
        self.block.as_ref().map_or((), |block| {
            bottoms.push((block.bottom.clone(), block.lines.1));
        });
        for parent in &self.function {
            bottoms.push((parent.bottom.clone(), parent.lines.1));
        }
        bottoms
    }

    fn get_total_lines(&self) -> (usize, usize) {
        self.block.as_ref().map_or_else(
            || {
                let mut start = self.lines.0;
                let mut end = self.lines.1;
                for parent in &self.function {
                    if parent.lines.0 < start {
                        start = parent.lines.0;
                        end = parent.lines.1;
                    }
                }
                (start, end)
            },
            |block| (block.lines.0, block.lines.1),
        )
    }

    impl_function_trait!(RustFunction);
}
