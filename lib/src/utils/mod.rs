mod stringify;
pub use stringify::StringifyVisitor;

mod graphviz;
pub use graphviz::GraphvizExporter;

mod hasher;
pub use hasher::{AstHasher, Crc32Hasher};

use crate::ast::Statement;
use std::fs::OpenOptions;
use std::process::Command;

pub fn write_ast_to_file(statement: &Statement, name: &str) {
    let dot_file_name = format!("{}.dot", name);

    // graphviz
    // dump dot file
    {
        let mut out = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&dot_file_name)
            .unwrap();

        let mut graphviz = GraphvizExporter::new(&mut out);
        graphviz.plot_statement(statement);
    }

    // convert to svg
    {
        Command::new("dot")
            .args(["-Tsvg", &dot_file_name, "-o", &format!("{}.svg", name)])
            .status()
            .expect("write ast to file failed.");
    }
}
