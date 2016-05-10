use variable::{VarBindingList};
use std::cell::RefCell;

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
    // 运行时参数
    rt_args: RefCell<Option<VarBindingList>>,
}

impl Stmt {
    pub fn new(kind: StmtKind, content: &str) -> Stmt {
        Stmt::new_with_args(kind, content, VarBindingList::new())
    }

    pub fn new_with_args(kind: StmtKind, content: &str, args: VarBindingList) -> Stmt {
        Stmt {
            kind: kind,
            content: content.to_string(),
            args: args,
            note: None,
            rt_args: RefCell::new(None),
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
    
    pub fn rtargs_init(&self, args: &VarBindingList) {
        let mut rtargs = self.rt_args.borrow_mut();
        assert!(rtargs.is_none(), "please call rtargs_init() **before** any other rtargs_*() call");
        let mut bindings = VarBindingList::new();
        bindings.add_more(args);
        *rtargs = Some(bindings);
    }
    
    pub fn rtargs_set(&self, name: &str, value: &str) {
        let mut rtargs = self.rt_args.borrow_mut();
        assert!(rtargs.is_some(), "please call rtargs_init() **first**");
        rtargs.as_mut().map(|bindings| bindings.set_binding(name, value));
    }
    
    pub fn rtargs_eval(&self, expr: &str, upvars1: Option<&VarBindingList>,
                                          upvars2: Option<&VarBindingList>) -> Option<String> {
        let rtargs = self.rt_args.borrow();
        rtargs.as_ref().map_or(None, |bindings| {
            bindings.eval(expr, upvars1, upvars2)
        })
    }

    pub fn rtargs_eval_var(&self, expr: &str, upvars1: Option<&VarBindingList>,
                                              upvars2: Option<&VarBindingList>) -> Option<String> {
        let rtargs = self.rt_args.borrow();
        rtargs.as_ref().map_or(None, |bindings| {
            bindings.eval_var(expr, upvars1, upvars2)
        })
    }
    
    pub fn rtargs_clean(&self) {
        *self.rt_args.borrow_mut() = None;
    }
}

#[cfg(test)]
mod tests {
    use statement::Stmt;
    use variable::VarBindingList;
    
    #[test]
    fn test_rt_args() {
        let mut init_args = VarBindingList::new();
        init_args.set_binding("z", "200");
        
        let stmt = Stmt::new_loop(6);
        stmt.rtargs_init(&init_args);
        let a = stmt.rtargs_eval_var("a", None, None);
        assert_eq!(a, None);
        stmt.rtargs_set("a", "123");
        let a = stmt.rtargs_eval_var("a", None, None);
        assert_eq!(a, Some("123".to_string()));
        stmt.rtargs_set("b", "var:a");
        let b = stmt.rtargs_eval_var("b", None, None);
        assert_eq!(b, Some("123".to_string()));
        let b = stmt.rtargs_eval("var:b", None, None);
        assert_eq!(b, Some("123".to_string()));
        let c = stmt.rtargs_eval_var("c", None, None);
        assert_eq!(c, None);
        let z = stmt.rtargs_eval_var("z", None, None);
        assert_eq!(z, Some("200".to_string())); // init args works
        
        stmt.rtargs_clean();
        let a = stmt.rtargs_eval_var("a", None, None);
        assert_eq!(a, None);
        let b = stmt.rtargs_eval_var("b", None, None);
        assert_eq!(b, None);
    }
}
