use crate::utils::visibility_zone::VisibilityZone;

#[derive(Default, Debug)]
pub struct Scopes {
    stack: Vec<Scope>,
    pub functions: Vec<Function>,
}

#[derive(Default, Debug)]
pub struct Scope {
    is_root: bool,
    start_line: usize,
    open_blocks: usize,
}

#[derive(Debug)]
pub struct Function {
    pub zone: VisibilityZone,
    pub signature: String,
}

impl Scopes {
    pub fn new() -> Self {
        let mut scopes = Scopes::default();
        scopes.enter_scope(0); // root scope
        scopes
    }

    pub fn enter_scope(&mut self, line_index: usize) {
        let mut scope = Scope {
            start_line: line_index,
            open_blocks: 0,
            is_root: self.stack.is_empty(),
        };
        if !self.stack.is_empty() {
            scope.inherit(self.get_current_scope())
        }
        self.stack.push(scope);
    }

    pub fn exit_scope(&mut self, line_index: usize) {
        let start_line = self.get_current_scope().start_line;
        let end_line = line_index;

        for function in self.functions.iter_mut().rev() {
            if function.zone.start == start_line && function.zone.end == 0 {
                function.zone.end = end_line;
            }
        }
        self.stack.pop();
    }

    pub fn get_current_scope(&mut self) -> &mut Scope {
        self.stack.last_mut().unwrap()
    }

    pub fn add_function(&mut self, signature: String) {
        let start_line = self.get_current_scope().start_line;
        self.functions.push(Function {
            // name,
            signature,
            zone: VisibilityZone {
                start: start_line,
                end: 0,
            },
        });
    }
}

impl Scope {
    fn inherit(&mut self, other: &Scope) {}

    pub fn add_block(&mut self) {
        self.open_blocks += 1;
    }

    pub fn close_block(&mut self) {
        self.open_blocks -= 1;
    }

    pub fn is_in_block(&self) -> bool {
        self.open_blocks > 0
    }

    pub fn is_root(&self) -> bool {
        self.is_root
    }

    pub fn start_line(&self) -> usize {
        self.start_line
    }
}
