use engine::{Engine, Context};
use statement::{Stmt, StmtKind};
use variable::{VarDef, VarDefList, VarBinding, VarBindingList};
use std::collections::HashMap;
use utils::split_lr;

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

    pub fn exec(&self, args: &VarBindingList, context: &mut Context, engine: &Engine) -> Result<(),String> {
        let mut eip = 0; // 指向将要执行（或正在执行）的语句
        let mut result: Result<(),String> = Ok(());
        // 初始化函数局部变量（复制函数参数作为局部变量）
        let mut locals = VarBindingList::new();
        locals.add_more(args);
        // 清除返回值
        context.globals.remove_binding("$return");

        // 下面一个大的循环依次执行每一条语句
        loop {
            if eip < 0 || eip >= self.stmts.len() {
                break;
            }
            let stmt = &self.stmts[eip];
            match stmt.kind {
                // 调用指令（由用户定义的指令）
                StmtKind::CallIns => {
                    result = engine.exec_ins(&stmt.content, &stmt.args, context);
                }
                // 调用函数（由用户定义的函数）
                StmtKind::CallFn => {
                    result = engine.exec_fn(&stmt.content, &stmt.args, context);
                }
                // 开始循环
                StmtKind::Loop => {

                }
                // 结束循环
                StmtKind::EndLoop => {

                }
                // 返回
                StmtKind::Return => {
                    // TODO: evaluates stmt.content
                    context.globals.set_binding("$return", &stmt.content);
                    break;
                }
                // 定义变量/绑定变量/变量运算
                StmtKind::SetVar => {

                }
                // 定义局部变量并赋值
                StmtKind::SetLocal => {
                    locals.set_binding("", "");
                }
                // 定义全局变量并赋值
                StmtKind::SetGlobal => {
                    let (name, value) = split_lr(&stmt.content, "=");
                    if name != "" {
                        // TODO: evaluates name and value
                        context.globals.set_binding(name, value);
                    } else {
                        context.log_error(format!("Invalid set global: {}", &stmt.content).as_str());
                    }

                }
            }
            eip += 1; // we'll execute next statement later
        } // end of loop

        result
    }

}
