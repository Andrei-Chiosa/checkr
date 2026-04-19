mod dfa;
mod dfa_gen;
mod minimizer;

use crate::rand::seq::IndexedRandom;
use ce_core::{Env, EnvError, Generate, ValidationResult, define_env, rand};
use serde::{Deserialize, Serialize};

use dfa::*;
use dfa_gen::*;
use minimizer::*;

define_env!(MinimizerEnv);

#[derive(tapi::Tapi, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Input {
    dfa: String,
}

#[derive(tapi::Tapi, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Output {
    //dfa: String,
    dot: String,
    minimized_dot: String,
    //errors: Vec<SemanticErrorDFA>
    deterministic: bool,
}

impl Env for MinimizerEnv {
    type Input = Input;

    type Output = Output;

    type Meta = ();

    type Annotation = ();

    fn run(input: &Self::Input) -> ce_core::Result<Self::Output> {
        let test_output = parse_dfa(&input.dfa).map_err(
            ce_core::EnvError::invalid_input_for_program("failed to parse DFA"),
        )?;

        let mut named_dfa = NamedDFA::build(test_output).map_err(
            ce_core::EnvError::invalid_input_for_program("failed to parse DFA"),
        )?;

        let dot = named_dfa.to_dot();

        let deterministic = named_dfa.dfa.check_determinism();

        let mut minimized_dot = "".to_string();

        if deterministic {
            let minimized_dfa =
                named_dfa
                    .minimize()
                    .map_err(ce_core::EnvError::invalid_input_for_program(
                        "failed to minimize dfa",
                    ))?;
            minimized_dot = minimized_dfa.to_dot();
        }

        Ok(Output {
            dot,
            minimized_dot,
            deterministic,
        })
    }

    fn validate(
        input: &Self::Input,
        output: &Self::Output,
    ) -> Result<(ValidationResult, ()), EnvError> {
        // run the reference implementation
        let mut expected = NamedDFA::build(parse_dfa(&input.dfa).map_err(
            ce_core::EnvError::invalid_input_for_program("failed to parse DFA"),
        )?)
        .map_err(ce_core::EnvError::invalid_input_for_program(
            "failed to parse DFA",
        ))?;

        let deterministic = expected.dfa.check_determinism();

        if deterministic {
            let expected =
                expected
                    .minimize()
                    .map_err(ce_core::EnvError::invalid_input_for_program(
                        "failed to minimize dfa",
                    ))?;

            //println!("expected state_count: {}", expected.dfa.state_count);

            let produced = DFA::from_dot(&output.minimized_dot).map_err(
                ce_core::EnvError::invalid_input_for_program("invalid dot output"),
            )?;

            if produced.dfa_isomorphic_to(&expected.dfa) {
                Ok((ValidationResult::Correct, ()))
            } else {
                Ok((
                    ValidationResult::Mismatch {
                        reason: "produced automaton is different from expected".into(),
                    },
                    (),
                ))
            }
        } else if deterministic == output.deterministic {
            Ok((ValidationResult::Correct, ()))
        } else if deterministic {
            Ok((
                ValidationResult::Mismatch {
                    reason: "DFA expected".into(),
                },
                (),
            ))
        } else {
            Ok((
                ValidationResult::Mismatch {
                    reason: "NFA expected".into(),
                },
                (),
            ))
        }
    }
}

impl Generate for Input {
    type Context = ();

    fn gn<R: rand::Rng>(cx: &mut Self::Context, rng: &mut R) -> Self {
        let state_count = if rng.random_bool(0.6) {
            rng.random_range(2..=4)
        } else {
            rng.random_range(5..=8)
        };
        let allow_nondeterminism = rng.random_bool(0.1);

        Self {
            dfa: generate_random_dfa(rng, state_count, allow_nondeterminism),
        }
    }
}
