use std::path::Path;

pub fn compile(source_file: &Path) {
    let source_code = std::fs::read_to_string(source_file).unwrap();

    let language = tree_sitter_gneiss::language();
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(language).unwrap();
    let tree = parser.parse(&source_code, None).unwrap();

    let source_code = source_code.into();
    let ast = crate::ast::File::parse(&tree, &source_code);
    eprintln!("{ast:#?}");

    todo!();
}
