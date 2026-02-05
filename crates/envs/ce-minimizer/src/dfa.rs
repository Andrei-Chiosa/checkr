use std::collections::HashMap;

use ce_core::EnvError;


#[derive(Debug)]
pub struct RawDfa {
    states: Vec<String>, 
    initial: Option<String>, 
    accepting: Vec<String>, 
    alphabet: Vec<String>, 
    transitions: Vec<(String, String, String)> //from, symbol, to
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken { line: usize, token: String },
    InvalidTransition { line: usize, text: String },
    MultipleInitial { line: usize },
    TokenOutsideSection { line: usize, token: String },
}

pub fn parse_dfa(input: &str) -> Result<RawDfa,EnvError> {    
    #[derive(PartialEq)]
    enum Section {
        Null,
        States, 
        Initial, 
        Accepting, 
        Alphabet, 
        Transitions
    }
    
    let mut dfa= RawDfa {
        states: Vec::new(),
        initial: None, 
        accepting: Vec::new(), 
        alphabet: Vec::new(), 
        transitions: Vec::new()
    };
    let mut current_section = Section::Null;

    for (_, raw_line) in input.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        match line {
            "#states" => current_section = Section::States,
            "#initial" => current_section = Section::Initial,
            "#accepting" => current_section = Section::Accepting,
            "#alphabet" => current_section = Section::Alphabet,
            "#transitions" => current_section = Section::Transitions,
            _ => {
                let line = line.to_string();
                match current_section {
                    Section::States => dfa.states.push(line),
                    Section::Initial => {
                        if dfa.initial.is_some() {
                            return Err(EnvError::InvalidInputForProgram { 
                                message: "one initial state allowed".to_string(), 
                                source: None
                            })
                        }
                        dfa.initial = Some(line.to_string())
                    }
                    Section::Accepting => dfa.accepting.push(line),
                    Section::Alphabet => dfa.alphabet.push(line),
                    Section::Transitions => {
                        
                        let (lhs, to) = 
                            match line.split_once('>') {
                                Some((lhs,to)) => (lhs, to),
                                None => {
                                    return Err(
                                        EnvError::InvalidInputForProgram { 
                                            message: "invalid transition present".to_string(), 
                                            source: None, 
                                        })
                                    }
                            };
                        
                        let (from, symbol) = 
                            match lhs.split_once(':') {
                                Some((from,symbol)) => (from, symbol),
                                None => {
                                    return Err(
                                        EnvError::InvalidInputForProgram { 
                                            message: "invalid transition present".to_string(), 
                                            source: None, 
                                        })
                                    }
                            };
                       
                        dfa
                            .transitions
                            .push((from.to_string(), symbol.to_string(), to.to_string()));
                    }
                    Section::Null => {
                       return Err(EnvError::InvalidInputForProgram { 
                                message: "uncharacterised token".to_string(), 
                                source: None
                            })
                    }
                    
                } 
            }
        }  
    }

    //syntax checking 
    if dfa.states.is_empty() {
        return Err(EnvError::InvalidInputForProgram { 
                message: "no states defined".into(),
                source: None
            });
    }
    if dfa.initial.is_none() {
        return Err(EnvError::InvalidInputForProgram { 
            message: "no initial state defined".into(),
            source: None
        });
    }
    for state in &dfa.accepting {
        if !dfa.states.contains(state) {
            return Err(EnvError::InvalidInputForProgram { 
                message: format!("accepting state '{}' is not a declared state", state),
                source: None
            });
        }
    }
    for (from, symbol, to) in &dfa.transitions {
        if !dfa.states.contains(from) {
            return Err(EnvError::InvalidInputForProgram { 
                message: format!("transition from unknown state '{}'", from),
                source: None
            });
        }
        if !dfa.states.contains(to) {
            return Err(EnvError::InvalidInputForProgram { 
                message: format!("transition to unknown state '{}'", to),
                source: None
            });
        }
        if !dfa.alphabet.contains(symbol) {
            return Err(EnvError::InvalidInputForProgram { 
                message: format!("transition uses unknown symbol '{}'", symbol),
                source: None
            });
        }
    }

    Ok(dfa)
}

//takes a semantically uncheked dfa
pub fn dfa_to_dot(dfa: &RawDfa) -> Result<String,EnvError> {
    let mut out = String::new();
    out.push_str("digraph DFA {\n");
    
    if dfa.states.is_empty() {
        return Err(EnvError::InvalidInputForProgram { 
            message: "no states available".to_string(), 
            source: None
        })
    }

    //gather states, no difference in starting and ending state
    for state in &dfa.states {
        let label = if Some(state) == dfa.initial.as_ref() {
            format!("{state}")
        } else if dfa.accepting.contains(state) {
            format!("{state}")
        } else {
            state.clone()
        };

        out.push_str(&format!(
            "  {state} [label=\"{label}\"];\n"
        ));
    }

    //group transitions
    let mut edges: HashMap<(&str, &str), Vec<&str>> = HashMap::new();

    if dfa.transitions.is_empty() {
         return Err(EnvError::InvalidInputForProgram { 
            message: "no transitions available".to_string(), 
            source: None
        })
    }

    for (from, symbol, to) in &dfa.transitions {
        edges 
            .entry((from.as_str(), to.as_str()))
            .or_default()
            .push(symbol.as_str());
    }
    
    //emit transitions
    for ((from,to), mut symbols) in edges {
        symbols.sort();
        let label = symbols.join(",");
        out.push_str(&format!(
            "  {from} -> {to} [label=\"{label}\"];\n"
        ));
    }

    out.push_str("}\n");
    Ok(out)
}