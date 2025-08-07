use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct FnInfo {
    pub line_at_call: usize,
    pub callees: Vec<(String, usize)>, // (callee_name, line_number)
}

pub fn find_roots(hm: &HashMap<String, FnInfo>) -> Vec<String> {
    let all_fns: HashSet<&String> = hm.keys().collect();
    let mut called_fns = HashSet::new();

    for info in hm.values() {
        for (callee, _) in &info.callees {
            called_fns.insert(callee);
        }
    }

    all_fns
        .difference(&called_fns)
        .map(|s| (*s).clone())
        .collect()
}

pub fn print_tree(
    name: &str,
    hm: &HashMap<String, FnInfo>,
    prefix: String,
    is_last: bool,
    visited: &mut HashSet<String>,
) {
    if !visited.insert(name.to_string()) {
        return;
    }

    let connector = if is_last { "└── " } else { "├── " };
    let fn_info = &hm[name];

    println!("{}{}{} (line {})", prefix, connector, name, fn_info.line_at_call);

    let new_prefix = if is_last {
        format!("{}    ", prefix)
    } else {
        format!("{}│   ", prefix)
    };

    let callees = &fn_info.callees;
    let len = callees.len();
    for (i, (callee, _)) in callees.iter().enumerate() {
        let is_last_callee = i == len - 1;
        print_tree(callee, hm, new_prefix.clone(), is_last_callee, visited);
    }
}



