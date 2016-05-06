use std::collections::HashMap;
use std::collections::hash_map::Entry;

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
            Entry::Vacant(mut entry) => {
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

    // TODO: EvalVar(), EvalExpr(), EvalVarAsStr
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
}
