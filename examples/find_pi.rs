use std::time::Instant;

use evofunc::{Instruction, Program, MCTS};

fn main() {
    let iset = vec![
        Instruction::Const(0),
        Instruction::Const(1),
        Instruction::Const(2),
        Instruction::Add,
        Instruction::Sub,
        Instruction::Mul,
        Instruction::Div,
        Instruction::Exp,
        Instruction::Log,
    ];

    const CONSTS: [f32; 3] = [0.0, 1.0, 2.0];

    let distance_to_pi = move |prog: &Program| {
        prog.evaluate_to_result_and_remaining_stack(&CONSTS, &[])
            .map(|(r, rest)| -(r - std::f32::consts::PI).abs())
    };

    let mut mcts = MCTS::with_max_program_length(&iset, 64, distance_to_pi);
    mcts.exploration_chance = 0.5;//1.0 / (2f32.sqrt());
    let mut current_high_score = mcts.high_score();
    eprintln!("start score {}", current_high_score.map(|v| v.to_string()).unwrap_or("/".to_string()));
    let start = Instant::now();
    let mut now = Instant::now();
    for i in 0..200_000_000 {
        if i % 1_000_000 == 0 && i != 0{
            let total_elapsed = start.elapsed().as_secs_f32();
            let total_minutes = (total_elapsed.floor() / 60.0).floor();
            let total_seconds = total_elapsed - (total_minutes * 60.0);
            let elapsed = now.elapsed().as_secs_f32();
            let nodes = mcts.node_count();
            let memory = mcts.node_memory_upper_bound() / 1024 / 1024 / 1024;
            eprint!("\r Explored {} million nodes in {}m{}s ({:.3}s/million), using {:.3}GB RAM\t\t\t", nodes / 1_000_000, total_minutes, total_seconds.ceil(), elapsed, memory);
            now = Instant::now();
        }
        let more_exploration = mcts.search_one();
        let high_score = mcts.high_score();
        if high_score > current_high_score {
            current_high_score = high_score;
            //let best_node_prog = mcts.best_node_program();
            let prog = mcts.make_best_program();
            let result = prog.evaluate_to_result(&CONSTS, &[]);
            let unformatted = mcts.make_best_program().render();
            let prog = mcts.make_best_program().render_pretty(&CONSTS)/*.map(|p| {
                let pycmd = format!("from sympy.parsing.sympy_parser import parse_expr; from sympy import E, simplify; print(simplify(parse_expr(\"{p}\", local_dict={{\"e\":E}}, evaluate=False)))");
                let out = std::process::Command::new("python").arg("-c").arg(pycmd).output();
                match out {
                    Err(e) => {
                        format!("Unoptimized: {}", p)
                    }
                    Ok(out)=> {
                        if out.stderr.len() > 0 {
                            eprintln!("Error formatting: {}", String::from_utf8_lossy(&out.stderr))
                        }
                        String::from_utf8_lossy(&out.stdout).trim().to_string()
                    },
                }                
            })*/;
            eprintln!(
                "\nNew highscore {} with program {} [{:?}] => {:?} ",
                high_score.map(|v| v.to_string()).unwrap_or("/".to_string()),
                prog.unwrap_or_default(),
                unformatted,
                result,
            );
        }
        if !more_exploration {
            break;
        }
    }
    eprintln!(
        "\nBest program has highscore {} with program {:?}",
        current_high_score.map(|v| v.to_string()).unwrap_or("/".to_string()),
        mcts.make_best_program().render()
    );

    //std::fs::write("mcts.dot", mcts.write_dot());
}
