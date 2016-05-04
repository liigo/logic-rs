use engine::Engine;
use statement::{Stmt, StmtKind};
use variable::{VarDef, VarDefList, VarBinding, VarBindingList};
use std::collections::HashMap;

pub struct FnDef {
    pub name: String,
    pub args: VarDefList,
    pub stmts: Vec<Stmt>,

    // privates
    loop_table: Option<HashMap<u32, u32>>,
}

impl FnDef {
    pub fn new(name: &str) -> FnDef {
        FnDef {
            name: name.to_string(),
            args: VarDefList::new(),
            stmts: Vec::new(),
            loop_table: None,
        }
    }

    pub fn add_stmt(&mut self, stmt: Stmt) {
        self.stmts.push(stmt);
    }

    pub fn exec(&self, engine: &mut Engine, args: &VarBindingList) -> Result<String,String> {

        Ok("".to_string())
    }
}
