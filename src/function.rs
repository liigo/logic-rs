use engine::{Engine, Context};
use statement::{Stmt, StmtKind};
use variable::{VarDefList, VarBindingList};
use std::collections::HashMap;
use std::cell::RefCell;
use utils::split_lr;

pub struct FnDef {
    pub name: String,
    pub args: VarDefList,
    pub stmts: Vec<Stmt>,

    // privates
    loop_table: RefCell<Option<HashMap<u32, u32>>>,
}

impl FnDef {
    pub fn new(name: &str) -> FnDef {
        FnDef {
            name: name.to_string(),
            args: VarDefList::new(),
            stmts: Vec::new(),
            loop_table: RefCell::new(None),
        }
    }

    pub fn add_stmt(&mut self, stmt: Stmt) {
        self.stmts.push(stmt);
    }

    pub fn exec(&self, args: &VarBindingList, context: &mut Context, engine: &Engine) -> Result<(),String> {
        let mut eip: u32 = 0; // 指向将要执行（或正在执行）的语句
        let mut result: Result<(),String> = Ok(());
        // 初始化函数局部变量（复制函数参数作为局部变量）
        let mut locals = VarBindingList::new();
        locals.add_more(args);
        // 清除返回值
        context.globals.remove_binding("$return");

        // 下面一个大的循环依次执行每一条语句
        loop {
            if eip as usize >= self.stmts.len() {
                break;
            }
            let stmt = &self.stmts[eip as usize];
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
                    if self.loop_table.borrow().is_none() {
                        self.build_loop_table(context);
                    }
                    
                    let index: u32 = if ! stmt.rtargs_contains("$index") {
                        // init for loop
                        stmt.rtargs_init(&stmt.args);
                        stmt.rtargs_set("$index", "0");
                        0
                    } else {
                        stmt.rtargs_eval_var("$index", None, None)
                            .map_or(0, |s| s.parse().expect("ensure $index is valid integer"))
                    };
                    
                    let count: u32 = stmt.rtargs_eval_var("$count", None, None)
                                         .map_or(0, |s| s.parse().expect("ensure $cound is valid integer"));
                    
                    if index < count {
                        // increase $index, then run loop body
                        stmt.rtargs_set("$index", (index + 1).to_string().as_str());
                        // 从循环体内代码的角度看(其实看不到), $index从1开始递增
                        // 下一条语句就是循环体, 无需跳转
                    } else {
                        // ending loop, do some cleanup, and go to the statement **after** endloop
                        stmt.rtargs_clean(); // 清除循环状态，以备此后再次执行
                        eip = self.find_loop_pair(eip, context).expect("find loop end") + 1;
                        continue;
                    }
                }
                // 结束循环
                StmtKind::EndLoop => {
                    // go to loop begin unconditionally
                    eip = self.find_loop_pair(eip, context).expect("find loop begin");
                    continue;
                }
                // 返回
                StmtKind::Return => {
                    // TODO: evaluates stmt.content
                    context.globals.set_binding("$return", &stmt.content);
                    // TODO: 清理loop.stmt.rtargs，否则再次调用此函数时循环条件永远不成立
                    break;
                }
                // 定义变量/绑定变量/变量运算
                StmtKind::SetVar => {
                    self.do_set_var(stmt, context, &mut locals);
                }
                // 定义局部变量并赋值
                StmtKind::SetLocal => {
                    self.do_set_local(&stmt.content, &mut locals, context);
                }
                // 定义全局变量并赋值
                StmtKind::SetGlobal => {
                    self.do_set_global(&stmt.content, context);
                }
            }
            eip += 1; // we'll execute next statement later
        } // end of loop

        result
    }
    
    fn find_loop_pair(&self, eip: u32, context: &mut Context) -> Option<u32> {
        self.loop_table.borrow().as_ref().map_or(None, |map| {
            map.get(&eip).map(|i| *i)
        })
    }
    
    fn do_set_local(&self, expr: &str, locals: &mut VarBindingList, context: &mut Context) {
        let (name, value) = split_lr(expr, "=");
        if name != "" {
            locals.set_binding(name, value);
        } else {
            context.log_error(format!("Invalid set local: {}", expr).as_str());
        }
    }
    
    fn do_set_global(&self, expr: &str, context: &mut Context) {
        let (name, value) = split_lr(expr, "=");
        if name != "" {
            context.globals.set_binding(name, value);
        } else {
            context.log_error(format!("Invalid set global: {}", expr).as_str());
        }
    }
    
    // 处理变量定义、赋值和运算操作
    // 所需操作数和操作符来自Stmt.args参数，要求其中包含以下固定名称的值绑定('$varname','$op1','$operand1',...)
    // 表达式基本形式和参数：$varname $op1 $operand1 $op2 $operand2 (各参数的绑定值均来自args/locals/globals)
    // 其中$op1为赋值操作符: = := += -= *= /=
    // 其中$op2为运算操作符: + - * /
    // 其中$op2和$operand2可被省略
    // 示例：
    // x = a
    // x := a
    // x += a
    // x = a + b
    // x += a + b
    // 存储结果：
    // x = a  定义变量x并写入局部变量表locals
    // x := a 定义变量x并写入全局变量表globals
    // 使用其他赋值操作符（+= -= *= /=）对变量赋值的，要求该变量必须事先存在（即先用=或:=定义变量）
    fn do_set_var(&self, stmt: &Stmt, context: &mut Context, locals: &mut VarBindingList) {
        // we do need these in statement's args:
        // varname, op1, operand1, op2, operand2   (the last two are optional)
        let args = &stmt.args;
        let varname = args.eval_var("$varname", Some(locals), Some(&mut context.globals));
        let op1 = args.eval_var("$op1", Some(locals), Some(&mut context.globals));
        let operand1 = args.eval_var("$operand1", Some(locals), Some(&mut context.globals));
        if varname.is_none() || op1.is_none() || operand1.is_none() {
            context.log_error("Set var requires named args at least: varname, op1, operand1");
            return;
        }
        let name = varname.as_ref().map_or("", |s| &s);
        let op2 = args.eval_var("$op2", Some(locals), Some(&mut context.globals));
        let operand2 = args.eval_var("$operand2", Some(locals), Some(&mut context.globals));
        
        let newvalue: Option<String> = if op2.is_some() && operand2.is_some() {
            Some(self.do_x_op_y(op2.as_ref().map_or("", |s| &s),
                                operand1.as_ref().map_or("", |s| &s),
                                operand2.as_ref().map_or("", |s| &s),
                                &stmt, context, locals))
        } else {
            if op2.is_some() || operand2.is_some() {
                context.log_error("Both $op2 and $operand2 are requried");
            }
            operand1.map_or(None, |s| stmt.args.eval(&s, Some(locals), Some(&mut context.globals)))
        };
        let newvalue = newvalue.as_ref().map_or("", |s| &s);
        
        let op1 = op1.as_ref().map_or("", |s| &s);
        match op1 {
            ":=" => { // set new global
                context.globals.set_binding(name, newvalue);
            }
            "=" => { // set new local
                locals.set_binding(name, newvalue);
            }
            
            _ => {
                // TODO: += -= *= /=
                if op1.ends_with("=") {
                    let oldvalue = locals.eval_var(name, Some(&mut context.globals), None);
                    let newvalue = self.do_x_op_y(&op1[..op1.len()-1],
                                                  oldvalue.as_ref().map_or("", |s| &s),
                                                  newvalue, stmt, context, locals);
                    if locals.contains(name) {
                        locals.set_binding(name, &newvalue);
                    } else if context.globals.contains(name) {
                        context.globals.set_binding(name, &newvalue);
                    } else {
                        context.log_error(&format!("Assign to undefined var: {}", name));
                    }
                }
                 
            }
        }
    }
    
    // op: + - * /
    // 对x和y这两个值执行op运算
    fn do_x_op_y(&self, op: &str, x: &str, y: &str,
                 stmt: &Stmt, context: &mut Context, locals: &mut VarBindingList) -> String {
        let (xl, xr) = split_lr(x, ":");
        let (yl, yr) = split_lr(y, ":");
        match op {
            "+" => {
                if xl == "int" && yl == "int" {
                    let x: isize = xr.parse().unwrap_or(0);
                    let y: isize = yr.parse().unwrap_or(0);
                    return format!("int:{}", x + y);
                } else {
                    return format!("str:{}{}", xr, yr);
                }
            }
            "-" => {
                if xl == "int" && yl == "int" {
                    let x: isize = xr.parse().unwrap_or(0);
                    let y: isize = yr.parse().unwrap_or(0);
                    return format!("int:{}", x - y);
                } else {
                    return "".to_string();
                }
            }
            "*" => {
                if xl == "int" && yl == "int" {
                    let x: isize = xr.parse().unwrap_or(0);
                    let y: isize = yr.parse().unwrap_or(0);
                    return format!("int:{}", x * y);
                } else {
                    return "".to_string();
                }
            }
            "/" => {
                if xl == "float" && yl == "float" {
                    let x: f64 = xr.parse().unwrap_or(0.0);
                    let y: f64 = yr.parse().unwrap_or(0.0);
                    return format!("int:{}", x / y);
                } else {
                    return "".to_string();
                }
            }
            _ => {
                context.log_error(&format!("Unsupport set var op: {}", op));
                return "".to_string();
            }
        }
    }
    
    fn build_loop_table(&self, context: &mut Context) {
        if self.loop_table.borrow().is_some() {
            debug_assert!(false, "only build loop table once");
            return;
        }
        let mut loop_pairs: HashMap<u32, u32> = HashMap::new();
        let mut loop_stack: Vec<u32> = Vec::new();
        let mut index = 0u32;
        for stmt in &self.stmts {
            match stmt.kind {
                StmtKind::Loop => {
                    loop_stack.push(index);
                }
                StmtKind::EndLoop => {
                    if loop_stack.len() > 0 {
                        let begin = loop_stack.pop().expect("exist");
                        loop_pairs.insert(begin, index);
                        loop_pairs.insert(index, begin);
                    } else {
                        context.log_fatal(&format!("Unpaired endloop in function: {} line: {}", self.name, index));
                        break;
                    }
                }
                _ => { }
            }
            index += 1;
        }
        
        *self.loop_table.borrow_mut() = Some(loop_pairs);
    }

}
