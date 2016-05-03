use variable::{VarBinding, VarBindingList};

// not use currently
trait Exec {
    fn exec(args: &VarBindingList, locals: &mut VarBindingList, globals: &mut VarBindingList);
}

// 语句类型
pub enum StmtKind {
    CallInst,    // 调用指令（由用户定义的指令）
    CallFn,      // 调用函数（由用户定义的函数）
    Loop,        // 开始循环
    EndLoop,     // 结束循环
    Return,      // 返回
    SetVar,      // 定义变量/绑定变量/变量运算
    SetLocal,    // 定义局部变量并赋值
    SetGlobal,   // 定义全局变量并赋值
}

// 表示函数内的任意一条可执行语句
// 这个结构可以方便的存入数据库
pub struct Stmt {
    pub kind: StmtKind,
    pub content: String,
    pub args: VarBindingList,
}

impl Stmt {
    pub fn new(kind: StmtKind, content: &str) -> Stmt {
        Stmt {
            kind: kind,
            content: content.to_string(),
            args: VarBindingList::new(),
        }
    }

    pub fn new_with_args(kind: StmtKind, content: &str, args: VarBindingList) -> Stmt {
        Stmt {
            kind: kind,
            content: content.to_string(),
            args: args,
        }
    }

    pub fn new_call_inst(name: &str, args: VarBindingList) -> Stmt {
        Stmt::new_with_args(StmtKind::CallInst, name, args)
    }

    pub fn new_call_fn(name: &str, args: VarBindingList) -> Stmt {
        Stmt::new_with_args(StmtKind::CallFn, name, args)
    }

    pub fn new_loop(count: u32) -> Stmt {
        let mut stmt = Stmt::new(StmtKind::Loop, "");
        stmt.args.set_binding("count", &count.to_string());
        stmt
    }

    pub fn new_end_loop() -> Stmt {
        Stmt::new(StmtKind::EndLoop, "")
    }

    pub fn new_return(expr: &str) -> Stmt {
        Stmt::new(StmtKind::Return, expr)
    }

    pub fn new_set_var(varname: &str, op1: &str, operand1: &str) -> Stmt {
        Stmt::new_set_var_ex(varname, op1, operand1, "", "")
    }

    pub fn new_set_var_ex(varname: &str, op1: &str, operand1: &str, op2: &str, operand2: &str) -> Stmt {
        let mut stmt = Stmt::new(StmtKind::SetVar, "");
        stmt.args.set_binding("varname", varname);
        stmt.args.set_binding("op1", op1);
        stmt.args.set_binding("operand1", operand1);
        stmt.args.set_binding("op2", op2);
        stmt.args.set_binding("operand2", operand2);
        stmt
    }

    // expr: "name=value"
    pub fn new_set_local(expr: &str) -> Stmt {
        Stmt::new(StmtKind::SetLocal, expr)
    }

    // expr: "name=value"
    pub fn new_set_global(expr: &str) -> Stmt {
        Stmt::new(StmtKind::SetGlobal, expr)
    }
}
