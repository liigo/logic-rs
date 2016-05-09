use std::collections::HashMap;
use std::collections::hash_map::Entry;
use utils::split_lr;

/// 变量定义（声明）
#[derive(Default, Debug)]
pub struct VarDef {
    pub name: String,
    /// i8,u8,i16,u16,i32,u32,f64,f64,str,hex
    pub typ:  String,
    /// 'a..z' or 'a...z'
    pub range: String,
    /// default value if not binded
    pub default: String,

    // todo:
    // pub note: String,
    // pub props: String,
}

impl VarDef {
    pub fn new<S: Into<String>>(name: S, typ: S) -> VarDef {
        VarDef {
            name: name.into(),
            typ: typ.into(),
            .. Default::default()
        }
    }
}

// 变量定义列表
#[derive(Debug)]
pub struct VarDefList {
   pub defs: Vec<VarDef>, // 保持定义的顺序始终不变
}

impl VarDefList {
    pub fn new() -> VarDefList {
        VarDefList {
            defs: Vec::with_capacity(3)
        }
    }

    pub fn add(&mut self, def: VarDef) {
        self.defs.push(def);
    }

    pub fn add_more(&mut self, defs: VarDefList) {
        for def in defs.defs {
            self.add(def);
        }
    }

    pub fn find(&self, name: &str) -> Option<&VarDef> {
        for def in &self.defs {
            if def.name == name {
                return Some(def);
            }
        }
        None
    }
}

// 变量值绑定
#[derive(Debug, Clone)]
pub struct VarBinding {
    name:  String,
    value: String,
}

// 变量值绑定列表
#[derive(Debug)]
pub struct VarBindingList {
    pub bindings: HashMap<String, VarBinding>,
}

impl VarBinding {
    pub fn new<S: Into<String>>(name: S, value: S) -> VarBinding {
        VarBinding {
            name: name.into(),
            value: value.into(),
        }
    }
}

impl VarBindingList {
    pub fn new() -> VarBindingList {
        VarBindingList {
            bindings: HashMap::with_capacity(8),
        }
    }

    pub fn add(&mut self, binding: VarBinding) {
        self.bindings.entry(binding.name.clone()).or_insert(binding);
    }

    pub fn add_more(&mut self, bindings: &VarBindingList) {
        for (_, binding) in &bindings.bindings {
            self.add(binding.clone());
        }
    }

    pub fn raw_value_of(&self, name: &str) -> Option<&str> {
        self.bindings.get(name).map(|binding| &binding.value[..])
    }

    pub fn contains(&self, name: &str) -> bool {
        self.bindings.contains_key(name)
    }

    /// Add new binding, update old binding, or remove old binding if value is empty('').
    /// 通常value文本应当带前缀(prefix:)如：'str:hello', 'int:123', 'var:name', 'hex:FF 1A 00'...
    /// 如果不带前缀的话可能会导致歧义，如'str:var:xxx'去掉前缀'str:'后含义迥异。
    pub fn set_binding(&mut self, name: &str, value: &str) {
        match self.bindings.entry(name.to_string()) {
            Entry::Vacant(entry) => {
                // add new binding
                entry.insert(VarBinding::new(name, value));
            }
            Entry::Occupied(mut entry) => {
                // update or remove old binding
                if value != "" {
                    // update old binding
                    entry.insert(VarBinding::new(name, value));
                } else {
                    // remove old binding
                    entry.remove();
                }
            }
        }
    }

    pub fn remove_binding(&mut self, name: &str) {
        self.bindings.remove(name);
    }
    
    /// 对指定名称的变量求值
    pub fn eval_var(&self, name: &str, upvars1: Option<&VarBindingList>,
                                       upvars2: Option<&VarBindingList>) -> Option<String> {
        let value = self.raw_value_of(name)
                        .or_else(|| { upvars1.map_or(None, |v| v.raw_value_of(name)) })
                        .or_else(|| { upvars2.map_or(None, |v| v.raw_value_of(name)) });
        value.map_or(None, |value_str| {
            let (l,r) = split_lr(value_str, ":");
            if l == "var" {
                self.eval_var(r, upvars1, upvars2)
            } else {
                Some(value_str.to_string())
            }
        })
    }
    
    /// 对指定名称的变量求值，但仅返回不带前缀的文本部分（如"int:123"返回"123"）
    pub fn eval_var_str(&self, name: &str, upvars1: Option<&VarBindingList>,
                                           upvars2: Option<&VarBindingList>) -> Option<String> {
        self.eval_var(name, upvars1, upvars2).map_or(None, |value_str| {
            let (l,r) = split_lr(&value_str, ":");
            assert!(l != "var");
            Some(r.to_string())  
        })
    }
    
    // 对表达式求值。如果expr是"var:name"形式的变量，返回该变量的值，否则返回expr本身。
    pub fn eval(&self, expr: &str, upvars1: Option<&VarBindingList>,
                                   upvars2: Option<&VarBindingList>) -> Option<String> {
        let (l,r) = split_lr(&expr, ":");
        if l == "var" {
            self.eval_var(r, upvars1, upvars2)
        } else {
            Some(expr.to_string())
        }
    }
    
    // 返回值的文本部分
    pub fn eval_str(&self, expr: &str, upvars1: Option<&VarBindingList>,
                                       upvars2: Option<&VarBindingList>) -> Option<String> {
        self.eval(expr, upvars1, upvars2).map_or(None, |value_str| {
            let (l,r) = split_lr(&value_str, ":");
            assert!(l != "var");
            Some(r.to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{VarDef, VarDefList, VarBinding, VarBindingList};
    #[test]
    fn test_vars() {
        let mut vars = VarDefList::new();
        vars.add(VarDef::new("a", "i32"));
        vars.add(VarDef::new("b", "u8"));
        assert!(vars.find("a").unwrap().name == "a");
        assert!(vars.find("a").unwrap().typ  == "i32");
        assert!(vars.find("b").unwrap().name == "b");
        assert!(vars.find("b").unwrap().typ  == "u8");

        let mut vars2 = VarDefList::new();
        vars2.add_more(vars);
        assert!(vars2.find("a").unwrap().name == "a");
        assert!(vars2.find("a").unwrap().typ  == "i32");
        assert!(vars2.find("b").unwrap().name == "b");
        assert!(vars2.find("b").unwrap().typ  == "u8");
    }

    #[test]
    fn test_bindings() {
        let binding = VarBinding::new("a", "123");
        assert!(binding.name  == "a");
        assert!(binding.value == "123");

        let mut bindings = VarBindingList::new();
        bindings.add(VarBinding::new("b", "hello"));
        bindings.add(binding);
        assert!(bindings.contains("a"));
        assert!(bindings.contains("b"));
        assert!(bindings.contains("c") == false);
        assert!(bindings.contains("A") == false); // case sensitive
        bindings.set_binding("c", "liigo"); // add new binding
        assert!(bindings.raw_value_of("a") == Some("123"));
        assert!(bindings.raw_value_of("b") == Some("hello"));
        assert!(bindings.raw_value_of("c") == Some("liigo"));
        bindings.set_binding("c", "6"); // update old binding
        assert!(bindings.raw_value_of("c") == Some("6"));
        bindings.set_binding("c", ""); // remove old binding
        assert!(bindings.contains("c") == false);
        assert!(bindings.raw_value_of("c") == None);
    }
    
    #[test]
    fn test_eval() {
        let args = {
            let mut args = VarBindingList::new();
            args.set_binding("a", "0");
            args.set_binding("s", "var:c");
            args
        };
        let locals = {
            let mut locals = VarBindingList::new();
            locals.set_binding("a", "1");
            locals.set_binding("b", "var:a");
            locals.set_binding("c", "var:g1");
            locals.set_binding("d", "var:g2");
            locals.set_binding("e", "int:123");
            locals
        };
        let globals = {
            let mut globals = VarBindingList::new();
            globals.set_binding("g1", "100");
            globals.set_binding("g2", "var:g3");
            globals.set_binding("g3", "var:a"); // ref to locals var "a"
            globals.set_binding("g4", "var:c");
            globals.set_binding("g5", "var:x");
            globals
        };
        
        // evaluates without upvars
        assert_eq!(locals.eval_var("a", None, None), Some("1".to_string())); // a = "1"
        assert_eq!(locals.eval_var("b", None, None), Some("1".to_string())); // b -> a -> "1"
        assert_eq!(locals.eval_var("c", None, None), None); // no upvars, so no "g1" is defined
        assert_eq!(locals.eval_var("x", None, None), None); // no "x"
        assert_eq!(locals.eval_var("e", None, None), Some("int:123".to_string())); // e = "int:123"
        assert_eq!(globals.eval_var("g1", None, None), Some("100".to_string())); // g1 = "100"
        assert_eq!(globals.eval_var("gx", None, None), None); // no "gx"
        assert_eq!(globals.eval_var("g3", None, None), None); // no "c"
        
        // evaluates with upvars1
        assert_eq!(locals.eval_var("c", Some(&globals), None), Some("100".to_string())); // c -> g1 -> "100"
        assert_eq!(locals.eval_var("d", Some(&globals), None), Some("1".to_string())); // d -> g2 -> g3 -> a -> "1"
        assert_eq!(locals.eval_var("x",  Some(&globals), None), None); // no "x"
        assert_eq!(locals.eval_var("g1", Some(&globals), None), Some("100".to_string())); // g1 = "100"
        assert_eq!(locals.eval_var("g2", Some(&globals), None), Some("1".to_string())); // g2 -> g3 -> a -> "1"
        assert_eq!(locals.eval_var("g4", Some(&globals), None), Some("100".to_string())); // g4 -> c -> g1 -> "100"
        assert_eq!(locals.eval_var("g5", Some(&globals), None), None); // no "x"
        assert_eq!(locals.eval_var("gx", Some(&globals), None), None); // no "gx"
        
        // evaluates with upvars1 and upvars2
        assert_eq!(args.eval_var("a", Some(&locals), Some(&globals)), Some("0".to_string())); // args.a shadows locals.a
        assert_eq!(args.eval_var("g1", Some(&locals), Some(&globals)), Some("100".to_string()));
        assert_eq!(args.eval_var("d", Some(&locals), Some(&globals)), Some("0".to_string())); // args.a shadows locals.a
        assert_eq!(args.eval_var("g4", Some(&locals), Some(&globals)), Some("100".to_string()));
        
        
        // eval_var_str()
        assert_eq!(locals.eval_var_str("e", None, None), Some("123".to_string()));
        assert_eq!(locals.eval_var_str("d", Some(&globals), None), Some("1".to_string()));
        assert_eq!(locals.eval_var_str("x", Some(&globals), None), None);
        
        // eval() and eval_str()
        assert_eq!(locals.eval("var:e", None, None), Some("int:123".to_string()));
        assert_eq!(locals.eval_str("var:e", None, None), Some("123".to_string()));
        assert_eq!(locals.eval("var:d", Some(&globals), None), Some("1".to_string()));
        assert_eq!(locals.eval_str("var:d", Some(&globals), None), Some("1".to_string()));
        assert_eq!(locals.eval("var:x", Some(&globals), None), None);
        assert_eq!(locals.eval_str("var:x", Some(&globals), None), None);
        assert_eq!(locals.eval("int:123", Some(&globals), None), Some("int:123".to_string()));
        assert_eq!(locals.eval_str("int:123", Some(&globals), None), Some("123".to_string()));
    }
}
