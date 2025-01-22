use core::f32;

use crate::{arena::{Ap, Arena}, instructions::{Instruction, Program}};

#[derive(Clone)]
pub struct ProgramNode{
    instruction: Instruction,
    /// Self score
    self_score: f32,
    /// Score of children
    child_score: f32,
    children: Vec<Ap<ProgramNode>>,
    parent: Option<Ap<ProgramNode>>,
    done: bool
}

impl ProgramNode{
    pub fn new(inst: Instruction, score: f32) -> Self{
        Self { instruction: inst, self_score: score, child_score: std::f32::NEG_INFINITY, children: Vec::new(), parent: None, done: false}
    }
}


pub struct MCTS{
    iset: Vec<Instruction>,
    root_node: Ap<ProgramNode>,
    pub exploration_chance: f32,
    pub max_program_length: usize,
    arena: Arena<ProgramNode>,
    evaluation_func: Box<dyn Fn(&Program) -> f32>,
    best_node: Ap<ProgramNode>
}

impl MCTS{
    pub fn new(instruction_set: &[Instruction],evaluate: impl Fn(&Program) -> f32 + 'static) -> Self{
        let max_program_length = 64;
        Self::with_max_program_length(instruction_set, max_program_length, evaluate)
    }

    pub fn with_max_program_length(instruction_set: &[Instruction], capacity: usize, evaluate: impl Fn(&Program) -> f32 + 'static) -> Self{
        let mut arena = Arena::with_capacity(instruction_set.len() * capacity);
        let root_node = arena.allocate(ProgramNode::new(instruction_set[0], f32::NEG_INFINITY));
        Self { iset: instruction_set.to_vec(), root_node, exploration_chance: 0.05, max_program_length: capacity, arena, evaluation_func: Box::new(evaluate), best_node: root_node }
    }

    pub fn high_score(&self) -> f32{
        self.best_node.get(&self.arena).self_score
    }

    pub fn make_best_program(&self) -> Program{
        self.make_program(&self.best_node)
    }

    pub fn make_program(&self, node: &Ap<ProgramNode>) -> Program{
        let mut instructions = Vec::new();
        let mut current_node = *node;
        while current_node != self.root_node{
            instructions.push(current_node.get(&self.arena).instruction);
            if let Some(parent) = current_node.get(&self.arena).parent{
                current_node = parent;
            }else{
                eprintln!("Node has no parent!");
                break;
            }
        }
        //We walked from back to front, so we need to reverse
        instructions.reverse();
        Program::create(&instructions)
    }

    fn recalculate_score_recursive(&mut self, node: &Ap<ProgramNode>){
        let child_count = node.get(&self.arena).children.iter().map(|c| c.get(&self.arena).self_score).filter(|s| s.is_finite()).count().min(1);
        // Mean
        //let child_score = node.get(&self.arena).children.iter().map(|c| c.get(&self.arena).self_score).filter(|s| s.is_finite()).sum::<f32>() / child_count as f32;
        // Max 
        let child_score = node.get(&self.arena).children.iter().map(|c| c.get(&self.arena).self_score).filter(|s| s.is_finite()).max_by(f32::total_cmp).unwrap_or(f32::NEG_INFINITY);
        node.get_mut(&mut self.arena).child_score = child_score;
        let done = node.get(&self.arena).children.iter().filter(|c| c.get(&self.arena).done).count();
        if done == self.iset.len(){
            node.get_mut(&mut self.arena).done = true;
        }
        if let Some(parent) = node.get(&self.arena).parent{
            self.recalculate_score_recursive(&parent);
        }
    }

    fn create_new_child(&mut self, node: &Ap<ProgramNode>, program: &mut Program) -> bool{
        //If the node already has all possible children, don't bother
        if node.get(&self.arena).children.len() == self.iset.len(){
            eprintln!("Node has all children");
            return false;
        }
        //Choose a new instruction that we haven't had yet
        let mut new_inst = fastrand::choice(self.iset.iter()).unwrap();
        //If we had that instruction already, take the first of the instruction set that isn't used
        if node.get(&self.arena).children.iter().any(|c| c.get(&self.arena).instruction == *new_inst){
            let mut allowed_instructions = self.iset.iter().filter(|new_inst| {
                node.get(&self.arena).children.iter().any(|c| c.get(&self.arena).instruction == **new_inst)
            });
            //If there are no unused instructions, return false
            let Some(actual_inst) = allowed_instructions.next() else {
                eprintln!("No more allowed instructions");
                return false
            };
            new_inst = actual_inst;
        }
        //Add the new instruction to the program
        program.push_inst(*new_inst);
        // Simulation step
        let score = (self.evaluation_func)(&program);
        //Insert into tree
        let mut new_node = ProgramNode::new(*new_inst, score);
        new_node.parent = Some(*node);
        let new_node_ap = self.arena.allocate(new_node);
        node.get_mut(&mut self.arena).children.push(new_node_ap);
        if score > self.high_score(){
            self.best_node = new_node_ap;
        }
        // Backpropagation step
        self.recalculate_score_recursive(node);
        true
    }

    fn search_step(&mut self, current_depth: usize, node: &Ap<ProgramNode>, program: &mut Program) -> bool{
        if current_depth == self.max_program_length{
            return false;
        }
        //Check if we are at the end of a branch
        if current_depth == self.max_program_length - 1{
            if node.get(&self.arena).children.len() == self.iset.len(){
                node.get_mut(&mut self.arena).done = true;
                return false;
            }
        }
        //Check if this node is already fully explored
        if node.get(&self.arena).done{
            eprintln!("Node already done");
            return false;
        }
        //Expansion step
        if (node.get(&self.arena).children.is_empty() || fastrand::f32() < self.exploration_chance) && (node.get(&self.arena).children.len() < self.iset.len()){
            self.create_new_child(node, program)
        }else{
            //Choice step
            let nodes = &node.get(&self.arena).children;
            //Choice happens as follows:
            //Each nodes score is softmaxed
            //We then add up the scores and if the random falls below the cumulative score for the current node, we choose it
            let max_score = nodes.iter().map(|n| n.get(&self.arena).self_score).max_by(|a,b| a.total_cmp(b)).unwrap_or_default();
            let numerator = nodes.iter().map(|n| (n.get(&self.arena).self_score - max_score).exp()).sum::<f32>();
            let random = fastrand::f32();
            let mut cumulative_score = 0.0;
            let mut chosen_index = nodes.len() - 1;
            let mut chosen_node = nodes[chosen_index];
            for (i, node) in nodes.iter().enumerate(){
                let node_score_softmax = (node.get(&self.arena).self_score - max_score).exp() / numerator;
                cumulative_score += node_score_softmax;
                if random < cumulative_score{
                    chosen_node = *node;
                    chosen_index = i;
                    break;
                }
            };
            if *node != self.root_node{
                program.push_inst(node.get(&self.arena).instruction);
            }
            let current_program_length = program.len();
            let total_node_count = nodes.len();
            if self.search_step(current_depth + 1, &chosen_node, program){
                return true;
            }else{
                for i in 0 .. total_node_count{
                    if i != chosen_index{
                        program.truncate_to_len(current_program_length);
                        let node = node.get(&self.arena).children[i].clone();
                        if self.search_step(current_depth + 1, &node, program){
                            return true;
                        }
                    }
                }
            }
            //If the search steps on all nodes have failed, extent
            program.truncate_to_len(current_program_length);
            self.create_new_child(node, program)
        }
    }

    pub fn search_one(&mut self) -> bool{
        let current_node = self.root_node;
        let mut program = Program::new();
        self.search_step(0, &current_node, &mut program)
    }

    pub fn write_dot(&self) -> String{
        use std::fmt::Write;
        let mut dotstr = String::from("digraph{");
        let mut nodes = vec![self.root_node];
        let get_node_name = |pointer: Ap<ProgramNode>|  if pointer == self.root_node {"root".to_string()} else { format!("n{}", pointer.internal_index())};
        while let Some(pointer) = nodes.pop(){
            let node = pointer.get(&self.arena);
            let program = self.make_program(&pointer);
            let result = program.run(&[0.0, 1.0, 2.0], &[]);
            let _ = writeln!(&mut dotstr, "{} [label=\"{:?}\nself={}\nchild={}\ndone={}\nprog=[{}]\nresult={:?}\"]", get_node_name(pointer), node.instruction, node.self_score, node.child_score, node.done, program.render(), result);
            if let Some(parent) = node.parent{
                let _ = writeln!(&mut dotstr, "{} -> {}", get_node_name(parent), get_node_name(pointer));
            }
            for c in node.children.iter().copied(){
                nodes.push(c);
            }
        }
        dotstr.push_str("\n}\n");
        dotstr
    }

}

#[cfg(test)]
mod tests{
    fn softmax(v: &[f32]) -> Vec<f32>{
        let max_score = v.iter().max_by(|a,b| a.total_cmp(b)).copied().unwrap_or_default();
        let numerator = v.iter().map(|n| (n - max_score).exp()).sum::<f32>();
        v.iter().map(|v| (*v - max_score).exp() - numerator).collect()
    }
    
    #[test]
    fn test_softmax(){
        let close = |a: f32, b: f32| (a - b) < 0.01;
        let input: [f32; 4] = [-2.0, 1.0, 0.1, 0.0];
        let expected: [f32; 4] = [0.02729201, 0.5481746 , 0.22287118, 0.20166218];
        assert!(softmax(&input).iter().zip(expected).all(|(c, e)| close(*c, e)));
    }
}
