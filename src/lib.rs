use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use sv_parser::{parse_sv_str, unwrap_node, Define, DefineText, Locate, RefNode};
use verilog_filelist_parser::Filelist;

pub type InstPath = String;

pub struct InstListAnalyzer {
    filelist: Filelist,
    top: Option<String>,
    pub instlist: Vec<InstPath>,
    instance_tree: Rc<RefCell<InstanceNode>>,
    sv_buffer: String,
}

#[derive(Default, Clone)]
struct InstanceNode {
    identifier: String,                                // instance identifier
    child: HashMap<String, Rc<RefCell<InstanceNode>>>, // instance identifier to instance mapping
    parent: Option<Rc<RefCell<InstanceNode>>>,         //
}

fn get_identifier(node: RefNode) -> Option<Locate> {
    // unwrap_node! can take multiple types
    match unwrap_node!(node, SimpleIdentifier, EscapedIdentifier, Keyword) {
        Some(RefNode::SimpleIdentifier(x)) => Some(x.nodes.0),
        Some(RefNode::EscapedIdentifier(x)) => Some(x.nodes.0),
        Some(RefNode::Keyword(x)) => Some(x.nodes.0),
        _ => None,
    }
}

impl InstanceNode {
    fn traversal(&self, instlist: &mut Vec<InstPath>) {
        if self.child.is_empty() {
            instlist.push(self.reverse_traversal())
        } else {
            for v in self.child.values() {
                v.borrow().traversal(instlist);
            }
        }
    }

    fn reverse_traversal(&self) -> InstPath {
        let mut top_down_path = InstPath::new();
        let mut parent = self.parent.clone();
        let mut path = vec![self.identifier.clone()];
        while let Some(p) = parent {
            path.push(p.borrow().identifier.clone());
            parent = p.borrow().parent.clone();
        }
        for p in path.iter().rev() {
            top_down_path.push_str(&format!("{}/", p));
        }
        top_down_path
    }
    fn deep_copy(&self) -> Self {
        let mut new_map: HashMap<String, Rc<RefCell<InstanceNode>>> = HashMap::new();
        if !self.child.is_empty() {
            for (ck, cv) in &self.child {
                let child_copy = cv.borrow().deep_copy();
                new_map.insert(ck.to_string(), Rc::new(RefCell::new(child_copy)));
            }
        } else {
            return self.clone();
        }
        Self {
            identifier: self.identifier.clone(),
            parent: self.parent.clone(),
            child: new_map,
        }
    }
    fn change_child_parent(&mut self, parent: Rc<RefCell<InstanceNode>>) {
        for cv in self.child.values() {
            cv.borrow_mut().parent = Some(parent.clone());
        }
    }
}

impl InstListAnalyzer {
    pub fn new<S: ToString>(top_name: S) -> Self {
        Self {
            instance_tree: Rc::new(RefCell::new(InstanceNode {
                identifier: top_name.to_string(),
                ..InstanceNode::default()
            })),
            top: Some(top_name.to_string()),
            filelist: Filelist::new(),
            instlist: vec![],
            sv_buffer: String::new(),
        }
    }

    pub fn parse_from_filelist<P: AsRef<Path>>(&mut self, path: P) {
        self.filelist = verilog_filelist_parser::parse_file(path)
            .expect("invalid verilog-2001 verilog filelist format");
        for fp in &self.filelist.files {
            let mut f = std::fs::File::open(fp).expect(&format!("no such file as {:?}", fp));
            let _ = f.read_to_string(&mut self.sv_buffer);
            self.sv_buffer.push_str("\n\n");
        }
    }

    pub fn generate_instlist(&mut self) {
        self.instance_tree.borrow().traversal(&mut self.instlist);
    }

    pub fn list_result(&self) {
        for p in &self.instlist {
            println!("{:?}", p)
        }
    }

    pub fn analyze_filelist(&mut self) -> bool {
        // if let Some(_top_module_path) = &self.top {
        let mut other_paths = vec![];
        let mut defines = HashMap::new();
        other_paths.extend(&self.filelist.files);
        other_paths.extend(&self.filelist.incdirs);
        for (k, v) in &self.filelist.defines {
            defines.insert(
                k.clone(),
                v.clone().map(|x| Define {
                    identifier: k.to_string(),
                    text: Some(DefineText {
                        text: x,
                        origin: None,
                    }),
                    arguments: vec![],
                }),
            );
        }
        let result = parse_sv_str(
            &self.sv_buffer,
            PathBuf::from(""),
            &defines,
            &other_paths,
            false,
            false,
        );
        assert!(result.is_ok());
        if let Ok((syntax_tree, _)) = result {
            let mut current_module = String::new();
            let mut buffered_nodes: HashMap<String, Rc<RefCell<InstanceNode>>> = HashMap::new();

            let mut current_node = Rc::new(RefCell::new(InstanceNode::default()));
            for node in &syntax_tree {
                match node {
                    RefNode::ModuleInstantiation(idnty) => {
                        let id = unwrap_node!(idnty, InstanceIdentifier).unwrap();
                        let id = get_identifier(id).unwrap();
                        let inst_name = syntax_tree.get_str(&id).unwrap();
                        let new_instnode = InstanceNode::default();
                        let child_node = Rc::new(RefCell::new(new_instnode));
                        child_node.borrow_mut().identifier = inst_name.to_string();
                        child_node.borrow_mut().parent = Some(current_node.clone());

                        let id = unwrap_node!(idnty, ModuleIdentifier).unwrap();
                        let id = get_identifier(id).unwrap();
                        let module_id = syntax_tree.get_str(&id).unwrap();
                        if let Some(v) = buffered_nodes.get(module_id) {
                            let v_has_parent = v.borrow().parent.is_some();
                            if v_has_parent {
                                let copy_of_v = Rc::new(RefCell::new(v.borrow().deep_copy()));
                                copy_of_v.borrow_mut().identifier = inst_name.to_string();
                                copy_of_v.borrow_mut().parent = Some(current_node.clone());
                                copy_of_v.borrow_mut().change_child_parent(child_node);
                                current_node
                                    .borrow_mut()
                                    .child
                                    .insert(inst_name.to_string(), copy_of_v.clone());
                            } else {
                                v.borrow_mut().identifier = inst_name.to_string();
                                v.borrow_mut().parent = Some(current_node.clone());
                                child_node.borrow_mut().child = v.borrow().child.clone();
                                current_node
                                    .borrow_mut()
                                    .child
                                    .insert(inst_name.to_string(), child_node.clone());
                            }
                        } else {
                            panic!("unrecognized module {}", module_id);
                        }
                    }
                    RefNode::ModuleDeclaration(module) => {
                        let id = unwrap_node!(module, ModuleIdentifier).unwrap();
                        let id = get_identifier(id).unwrap();
                        let module_id = syntax_tree.get_str(&id).unwrap();
                        current_node = Rc::new(RefCell::new(InstanceNode::default()));
                        if let Some(top) = &self.top {
                            if top == module_id {
                                current_node.borrow_mut().identifier = module_id.to_string();
                            }
                        }
                        current_module = module_id.to_string();
                    }
                    RefNode::Keyword(kid) => {
                        let id = unwrap_node!(kid, Keyword).unwrap();
                        let id = get_identifier(id).unwrap();
                        let kwd = syntax_tree.get_str(&id).unwrap();
                        if kwd == "endmodule" {
                            if let Some(top) = &self.top {
                                if *top == current_module {
                                    self.instance_tree = current_node;
                                    break;
                                } else {
                                    buffered_nodes
                                        .entry(current_module.clone())
                                        .or_insert_with(|| current_node.clone());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            true
        } else {
            println!("top module parse failed");
            false
        }
    }
}
