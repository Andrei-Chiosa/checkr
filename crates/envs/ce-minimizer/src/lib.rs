mod dfa;
mod minimizer;
mod dfa_gen;

use ce_core::{Env, Generate, ValidationResult, define_env, rand, EnvError};
use serde::{Deserialize, Serialize};
use crate::rand::{seq::IndexedRandom};

use dfa::*;
use minimizer::*;
use dfa_gen::*;

define_env!(MinimizerEnv);

#[derive(tapi::Tapi, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Input {
    dfa: String
}

#[derive(tapi::Tapi, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Output {
    //dfa: String,
    dot: String, 
    minimized_dot: String,
    //errors: Vec<SemanticErrorDFA>
    deterministic: bool
}

impl Env for MinimizerEnv {
    type Input = Input;

    type Output = Output;

    type Meta = ();

    type Annotation = ();

    fn run(input: &Self::Input) -> ce_core::Result<Self::Output> {
        let test_output = parse_dfa(&input.dfa)
            .map_err(ce_core::EnvError::invalid_input_for_program("failed to parse DFA"))?;
        
        let mut named_dfa = NamedDFA::build(test_output)
            .map_err(ce_core::EnvError::invalid_input_for_program("failed to parse DFA"))?;

        let dot = named_dfa.to_dot();

        let deterministic = named_dfa.dfa.check_determinism();

        let mut minimized_dot = "".to_string();

        if deterministic {
            let minimized_dfa = named_dfa.minimize()
                .map_err(ce_core::EnvError::invalid_input_for_program("failed to minimize dfa"))?;
            minimized_dot = minimized_dfa.to_dot();
        }

        Ok( Output { dot, minimized_dot, deterministic})        
    }

    fn validate(input: &Self::Input, output: &Self::Output) -> Result<(ValidationResult, ()), EnvError> {
        //input is for reference implementation and output is for the student
        
        // run the reference implementation
        let expected = NamedDFA::build(
                parse_dfa(&input.dfa).map_err(ce_core::EnvError::invalid_input_for_program("failed to parse DFA"))?
            ).map_err(ce_core::EnvError::invalid_input_for_program("failed to parse DFA"))?
            .minimize().map_err(ce_core::EnvError::invalid_input_for_program("failed to minimize dfa"))?
            .to_dot();
        
        let produced = &output.minimized_dot;

        Ok((ValidationResult::Correct, ()))
    }
}

impl Generate for Input {
    type Context = ();

    fn gn<R: rand::Rng>(cx: &mut Self::Context, rng: &mut R) -> Self {
        let state_count = if rng.random_bool(0.6) {rng.random_range(2..=4)} else { rng.random_range(5..=8)};
        let allow_nondeterminism = rng.random_bool(0.1);

        Self { dfa: generate_random_dfa(rng, state_count, allow_nondeterminism) }
    }
}

pub fn normalize(dfa: &DFA) -> DFA {
    let mut mapping: Vec<Option<usize>> = vec![None; dfa.state_count];
    let mut queue = std::collections::VecDeque::new();
    let mut counter = 0;

    mapping[dfa.initial] = Some(counter);
    counter += 1;
    queue.push_back(dfa.initial);

    while let Some(state) = queue.pop_front() {
        let mut edges: Vec<&Edge> = dfa.edges.iter()
            .filter(|e| e.from == state)
            .collect();
        edges.sort_by_key(|e| e.symbol);

        for edge in edges {
            if mapping[edge.to].is_none() {
                mapping[edge.to] = Some(counter);
                counter += 1;
                queue.push_back(edge.to);
            }
        }
    }

    let mut new_edges: Vec<Edge> = dfa.edges.iter()
        .filter(|e| mapping[e.from].is_some() && mapping[e.to].is_some())
        .map(|e| Edge {
            from: mapping[e.from].unwrap(),
            symbol: e.symbol,
            to: mapping[e.to].unwrap(),
        })
        .collect();
    new_edges.sort_by_key(|e| (e.from, e.symbol));

    let mut new_accepting: Vec<Node> = dfa.accepting.iter()
        .filter_map(|&s| mapping[s])
        .collect();
    new_accepting.sort();

    let mut new_alphabet = dfa.alphabet.clone();
    new_alphabet.sort();

    DFA {
        state_count: counter,
        edges: new_edges,
        initial: 0,
        accepting: new_accepting,
        alphabet: new_alphabet,
    }
}

pub fn is_isomorphic(a: &DFA, b: &DFA) -> bool {
    normalize(a) == normalize(b)
}