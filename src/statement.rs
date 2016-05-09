use variable::{VarBindingList};

// not use currently
trait Exec {
    fn exec(args: &VarBindingList, locals: &mut VarBindingList, globals: &mut VarBindingList);
}

/// 语句类型
pub enum StmtKind {
    /// 调用指令（由用户定义的指令）；Stmt.content为指令名称，Stmt.args为调用参数。
    CallIns,
    /// 调用函数（由用户定义的函数）；Stmt.content为函数名称，Stmt.args为调用参数。
    CallFn,
    /// 开始循环
    Loop,
    /// 结束循环
    EndLoop,
    /// 结束函数执行并返回值；Stmt.content为返回值
    Return,
    /// 定义变量/绑定变量/变量运算；Stmt.content为"name=value"的表达式
    SetVar,
    /// 定义局部变量并赋值；Stmt.content为"name=value"的表达式
    SetLocal,
    /// 定义全局变量并赋值；Stmt.content为"name=value"的表达式
    SetGlobal,
}

/// 表示函数内的任意一条可执行语句
/// 这个结构可以方便的序列化至数据库或JSON
pub struct Stmt {
    /// 语句类型，详见StmtKind的说明
    pub kind: StmtKind,
    /// 其含义取决于指令类型，详见StmtKind的说明
    pub content: String,
    /// 语句参数
    pub args: VarBindingList,
    /// 注释
    pub note: Option<String>,
}

impl Stmt {
    pub fn new(kind: StmtKind, content: &str) -> Stmt {
        Stmt {
            kind: kind,
            content: content.to_string(),
            args: VarBindingList::new(),
            note: None,
        }
    }

    pub fn new_with_args(kind: StmtKind, content: &str, args: VarBindingList) -> Stmt {
        Stmt {
            kind: kind,
            content: content.to_string(),
            args: args,
            note: None,
        }
    }

    pub fn new_call_ins(name: &str, args: VarBindingList) -> Stmt {
        Stmt::new_with_args(StmtKind::CallIns, name, args)
    }

    pub fn new_call_fn(name: &str, args: VarBindingList) -> Stmt {
        Stmt::new_with_args(StmtKind::CallFn, name, args)
    }

    pub fn new_loop(count: u32) -> Stmt {
        let mut stmt = Stmt::new(StmtKind::Loop, "");
        stmt.args.set_binding("$count", &count.to_string());
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
        stmt.args.set_binding("$varname", varname);
        stmt.args.set_binding("$op1", op1);
        stmt.args.set_binding("$operand1", operand1);
        if op2.len() > 0 {
            stmt.args.set_binding("$op2", op2);
            if operand2.len() > 0 {
                stmt.args.set_binding("$operand2", operand2);
            }
        }
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
