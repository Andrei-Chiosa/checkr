use ce_core::rand;
use ce_core::rand::Rng;
use ce_minimizer::*;

fn main() {
    let mut rng = rand::rng();
    let state_count = if rng.random_bool(0.6) {rng.random_range(2..=4)} else { rng.random_range(5..=8)};
    let allow_nondeterminism = rng.random_bool(0.1);

    let dfa_input = generate_random_dfa(&mut rng, state_count, allow_nondeterminism);

    println!("{dfa_input}")
}