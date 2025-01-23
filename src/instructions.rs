
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Instruction{
    Add,
    Sub,
    Mul,
    Div,
    Exp,
    Log,
    Const(u8),
    Var(u8)
}

pub const STACKSIZE: usize = 128;

#[derive(Clone)]
pub struct Program{
    pub instructions: Vec<Instruction>
}

impl Program{
    pub fn new() -> Self{
        Self{instructions: Vec::new()}
    }

    pub fn push_inst(&mut self, inst: Instruction){
        self.instructions.push(inst);
    }

    pub fn remove_last_inst(&mut self){
        self.instructions.pop();
    }

    pub fn len(&self) -> usize{
        self.instructions.len()
    }

    pub fn truncate_to_len(&mut self, len: usize){
        self.instructions.truncate(len);
    }

    pub fn clear(&mut self){
        self.instructions.clear();
    }
    
    pub fn create(insts: &[Instruction]) -> Self{
        Self{instructions: insts.to_vec()}
    }

    pub fn evaluate_to_result(&self, consts: &[f32], vars: &[f32]) -> Option<f32>{
        self.run(consts, vars).and_then(|mut s| s.pop())
    }

    pub fn evaluate_to_result_and_remaining_stack(&self, consts: &[f32], vars: &[f32]) -> Option<(f32, usize)>{
        self.run(consts, vars).and_then(|mut s| Some((s.pop()?, s.len())))
    }

    pub fn run(&self, consts: &[f32], vars: &[f32]) -> Option<Stack>{
        let mut stack = Stack::new();
        for inst in self.instructions.iter(){
            match inst{
                Instruction::Add => {
                    let b = stack.pop()?;
                    let a = stack.pop()?;
                    stack.push(a + b);
                },
                Instruction::Sub => {
                    let b = stack.pop()?;
                    let a = stack.pop()?;
                    stack.push(a - b);
                },
                Instruction::Mul => {
                    let b = stack.pop()?;
                    let a = stack.pop()?;
                    stack.push(a * b);
                },
                Instruction::Div => {
                    let b = stack.pop()?;
                    let a = stack.pop()?;
                    if b == 0.0{
                        stack.push(1.0);
                    }else{
                        stack.push(a / b);
                    }
                },
                Instruction::Exp => {
                    let v = stack.pop()?;
                    stack.push(v.exp());
                },
                Instruction::Log => {
                    let v = stack.pop()?.min(f32::MIN_POSITIVE);
                    stack.push(v.ln());
                },
                Instruction::Const(idx) => {
                    let v = consts[*idx as usize];
                    stack.push(v);
                },
                Instruction::Var(idx) => {
                    let v = vars[*idx as usize];
                    stack.push(v);
                },
            }
        }
        Some(stack)
    }

    pub fn render(&self) -> String{
        self.instructions.iter().map(|inst| format!("{:?}", inst)).collect::<Vec<_>>().join(" ")
    }

    pub fn render_pretty(&self, consts: &[f32]) -> Option<String>{
        let mut stack = Vec::new();
        enum Node{
            Leaf(Instruction),
            Node(Instruction, Vec<Node>)
        }
        impl Node{
            pub fn to_string(&self, consts: &[f32]) -> String{
                match self{
                    Node::Leaf(instruction) => match instruction{
                        Instruction::Const(ci) => format!("{}", consts[*ci as usize]),
                        Instruction::Var(vi) => format!("v_{vi}"),
                        _ => format!("{:?}", instruction)
                    },
                    Node::Node(i, c) => {
                        match i{
                            Instruction::Add => format!("({} + {})", c[0].to_string(consts), c[1].to_string(consts)),
                            Instruction::Sub => format!("({} - {})", c[0].to_string(consts), c[1].to_string(consts)),
                            Instruction::Mul => format!("({} * {})", c[0].to_string(consts), c[1].to_string(consts)),
                            Instruction::Div => format!("({} / {})", c[0].to_string(consts), c[1].to_string(consts)),
                            Instruction::Exp => format!("(e**{})", c[0].to_string(consts)),
                            Instruction::Log => format!("(ln({}))", c[0].to_string(consts)),
                            Instruction::Const(ci) => format!("{}", consts[*ci as usize]),
                            Instruction::Var(vi) => format!("v_{vi}"),
                        }
                    },
                }
            }
        }
        for inst in self.instructions.iter().copied(){
            match inst{
                Instruction::Add => {
                    let a = stack.pop()?;
                    let b = stack.pop()?;
                    stack.push(Node::Node(inst, vec![a,b]));
                },
                Instruction::Sub => {
                    let a = stack.pop()?;
                    let b = stack.pop()?;
                    stack.push(Node::Node(inst, vec![a,b]));
                },
                Instruction::Mul => {
                    let a = stack.pop()?;
                    let b = stack.pop()?;
                    stack.push(Node::Node(inst, vec![a,b]));
                },
                Instruction::Div => {
                    let a = stack.pop()?;
                    let b = stack.pop()?;
                    stack.push(Node::Node(inst, vec![a,b]));
                },
                Instruction::Exp => {
                    let a = stack.pop()?;
                    stack.push(Node::Node(inst, vec![a]));
                },
                Instruction::Log => {
                    let a = stack.pop()?;
                    stack.push(Node::Node(inst, vec![a]));
                },
                Instruction::Const(_) | Instruction::Var(_) => {
                    stack.push(Node::Leaf(inst))
                }
            }
        }

        stack.pop().map(|n| n.to_string(consts))
    }
}

pub struct Stack{
    values: [f32; STACKSIZE],
    sp: usize
}

impl Stack{
    pub fn new() -> Self{
        Stack { values: [0f32; STACKSIZE], sp: 0 }
    }

    pub fn len(&self) -> usize{
        self.sp
    }

    pub fn push(&mut self, value: f32){
        self.values[self.sp] = value;
        self.sp = (self.sp + 1).min(STACKSIZE - 1);
    }

    pub fn pop(&mut self) -> Option<f32>{
        if self.sp == 0{
            None
        }else{
            self.sp = self.sp.saturating_sub(1);
            Some(self.values[self.sp])
        }
    }
}