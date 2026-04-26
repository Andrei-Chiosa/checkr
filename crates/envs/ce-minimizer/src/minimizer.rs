use tapi::kind::Name;

use super::{DFA, Edge, NamedDFA, Node};
use std::collections::{BTreeSet, BTreeMap, VecDeque};

#[derive(PartialEq)]
enum Equivalence {
    Equivalent,
    Distinguishable,
}

type EquivTable = Vec<Vec<bool>>;

#[derive(
    Debug, Clone, PartialEq, tapi::Tapi, serde::Serialize, serde::Deserialize, thiserror::Error,
)]
pub enum MinimizationError {
    #[error("transition not found")]
    IncompleteInput,
}

impl NamedDFA {
    pub fn minimize(&mut self) -> Result<NamedDFA, MinimizationError> {
        // Remove unreachable states first
        self.remove_unreachable_states();

        let table = self.build_equivalence_table()?;

        //partition the states into blocks of mutually equivalent states
        let n = self.dfa.state_count;
        let mut class = vec![usize::MAX; n]; //class[state] get the equivalence class
        let mut num_classes = 0;

        for p in 0..n {
            if class[p] != usize::MAX {
                continue;
            }

            class[p] = num_classes;
            for q in (p + 1)..n {
                if self.equivalent(p, q, &table) == Equivalence::Equivalent {
                    class[q] = num_classes;
                }
            }
            num_classes += 1;
        }

        // representative[class_id] = the original state that represents it
        let mut representative = vec![usize::MAX; num_classes];
        for p in 0..n {
            let c = class[p];
            if representative[c] == usize::MAX {
                representative[c] = p; // first state seen = representative
            }
        }

        // new edges
        let mut new_edges: Vec<Edge> = Vec::new();
        for c in 0..num_classes {
            let rep = representative[c];
            for edge in &self.dfa.edges {
                if edge.from == rep {
                    new_edges.push(Edge {
                        from: c,
                        symbol: edge.symbol,
                        to: class[edge.to],
                    });
                }
            }
        }

        let new_accepting: Vec<Node> = (0..num_classes)
            .filter(|&c| self.dfa.accepting.contains(&representative[c]))
            .collect();

        let new_initial = class[self.dfa.initial];

        let new_names: Vec<String> = (0..num_classes)
            .map(|c| {
                let members: Vec<&str> = (0..n)
                    .filter(|&p| class[p] == c)
                    .map(|p| self.names[p].as_str())
                    .collect();
                members.join(",")
            })
            .collect();

        Ok(NamedDFA {
            dfa: DFA {
                state_count: num_classes,
                edges: new_edges,
                initial: new_initial,
                accepting: new_accepting,
                alphabet: self.dfa.alphabet.clone(),
            },
            names: new_names,
        })
    }

    fn remove_unreachable_states(&mut self) {
        //Perform a BFS
        let mut visited: BTreeSet<Node> = BTreeSet::new();
        let mut queue: VecDeque<Node> = VecDeque::new();

        queue.push_back(self.dfa.initial);

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            for edge in &self.dfa.edges {
                if edge.from == current {
                    queue.push_back(edge.to);
                }
            }
        }

        self.dfa.edges.retain(|e| visited.contains(&e.from));

        self.dfa.accepting.retain(|s| visited.contains(s));

        //remap unreachable states
        let mut remap: BTreeMap<Node, Node> = BTreeMap::new();
        for (new_id, old_id) in visited.iter().enumerate() {
            remap.insert(*old_id, new_id);
        }

        let mut new_names = vec![String::new(); visited.len()];
        for (old_id, name) in self.names.iter().enumerate() {
            if let Some(&new_id) = remap.get(&old_id) {
                new_names[new_id] = name.clone();
            }
        }
        self.names = new_names;

        for edge in &mut self.dfa.edges {
            edge.from = remap[&edge.from];
            edge.to = remap[&edge.to];
        }

        self.dfa.accepting = self.dfa.accepting.iter().map(|s| remap[s]).collect();

        self.dfa.initial = remap[&self.dfa.initial];

        self.dfa.state_count = visited.len();
    }

    fn build_equivalence_table(&self) -> Result<EquivTable, MinimizationError> {
        let n = self.dfa.state_count;

        // All pairs assumed equivalent (false), (true) pair is distinguishable
        let mut table = vec![vec![false; n]; n];

        // Base case
        for p in 0..n {
            for q in 0..n {
                let p_acc = self.dfa.accepting.contains(&p);
                let q_acc = self.dfa.accepting.contains(&q);
                if p_acc != q_acc {
                    table[p][q] = true;
                    table[q][p] = true;
                }
            }
        }

        let mut changed = true;
        while changed {
            changed = false;
            for p in 0..n {
                for q in 0..n {
                    if table[p][q] {
                        continue;
                    }
                    for symbol in &self.dfa.alphabet {
                        let dp = self
                            .dfa
                            .delta(p, *symbol)
                            .ok_or(MinimizationError::IncompleteInput)?;
                        let dq = self
                            .dfa
                            .delta(q, *symbol)
                            .ok_or(MinimizationError::IncompleteInput)?;
                        if table[dp][dq] {
                            table[p][q] = true;
                            table[q][p] = true;
                            changed = true;
                            break;
                        }
                    }
                }
            }
        }

        Ok(table)
    }

    fn equivalent(&self, q0: Node, q1: Node, table: &EquivTable) -> Equivalence {
        if q0 >= self.dfa.state_count || q1 >= self.dfa.state_count {
            panic!("States not present in DFA");
        }
        if table[q0][q1] {
            Equivalence::Distinguishable
        } else {
            Equivalence::Equivalent
        }
    }
}

#[cfg(test)]
mod tests {
    use std::vec;
    use super::*;

    fn dfa1() -> NamedDFA {
        NamedDFA {
            dfa: DFA {
                state_count: 4,
                initial: 0,
                accepting: vec![0, 1],
                alphabet: vec!['1', '2'],
                edges: vec![
                    Edge { from: 0, symbol: '2', to: 1 },
                    Edge { from: 1, symbol: '2', to: 0 },
                    Edge { from: 0, symbol: '1', to: 2 },
                    Edge { from: 1, symbol: '1', to: 2 },
                    Edge { from: 2, symbol: '1', to: 0 },
                    Edge { from: 2, symbol: '2', to: 2 },
                    Edge { from: 3, symbol: '1', to: 2 },
                    Edge { from: 3, symbol: '2', to: 2 },
                ],
            },
            names: vec!["q0".to_string(), "q1".to_string(), "q2".to_string(), "q3".to_string()],
        }
    }

    fn dfa2_already_minimized() -> NamedDFA {
        NamedDFA {
            dfa: DFA {
                state_count: 3,
                initial: 0,
                accepting: vec![2],
                alphabet: vec!['0', '1', '3'],
                edges: vec![
                    Edge { from: 0, symbol: '3', to: 0 },
                    Edge { from: 0, symbol: '0', to: 1 },
                    Edge { from: 0, symbol: '1', to: 1 },
                    Edge { from: 1, symbol: '1', to: 0 },
                    Edge { from: 1, symbol: '3', to: 0 },
                    Edge { from: 1, symbol: '0', to: 2 },
                    Edge { from: 2, symbol: '3', to: 1 },
                    Edge { from: 2, symbol: '0', to: 2 },
                    Edge { from: 2, symbol: '1', to: 2 },
                ],
            },
            names: vec!["q0".to_string(), "q1".to_string(), "q2".to_string()],
        }
    }

    fn dfa3_all_states_equivalent() -> NamedDFA {
        NamedDFA {
            dfa: DFA {
                state_count: 3,
                initial: 0,
                accepting: vec![0, 1, 2], 
                alphabet: vec!['1', '2'],
                edges: vec![
                    Edge { from: 0, symbol: '1', to: 1 },
                    Edge { from: 0, symbol: '2', to: 2 },
                    Edge { from: 1, symbol: '1', to: 0 },
                    Edge { from: 1, symbol: '2', to: 2 },
                    Edge { from: 2, symbol: '1', to: 0 },
                    Edge { from: 2, symbol: '2', to: 1 },
                ],
            },
            names: vec!["q0".to_string(), "q1".to_string(), "q2".to_string()],
        }
    }

    fn dfa4() -> NamedDFA {
        NamedDFA {
            dfa: DFA {
                state_count: 4,
                initial: 0,
                accepting: vec![1],
                alphabet: vec!['1', '2'],
                edges: vec![
                    Edge { from: 0, symbol: '1', to: 1 },
                    Edge { from: 0, symbol: '2', to: 2 }, // trap A
                    Edge { from: 1, symbol: '1', to: 1 },
                    Edge { from: 1, symbol: '2', to: 3 }, // trap B
                    Edge { from: 2, symbol: '1', to: 2 },
                    Edge { from: 2, symbol: '2', to: 2 },
                    Edge { from: 3, symbol: '1', to: 3 },
                    Edge { from: 3, symbol: '2', to: 3 },
                ],
            },
            names: vec!["q0".to_string(), "q1".to_string(), "q2".to_string(), "q3".to_string()],
        }
    }

    fn dfa_incomplete() -> NamedDFA {
        NamedDFA {
            dfa: DFA {
                state_count: 3,
                initial: 0,
                accepting: vec![2],
                alphabet: vec!['a', 'b'],
                edges: vec![
                    Edge { from: 0, symbol: 'a', to: 1 },
                    Edge { from: 0, symbol: 'b', to: 0 },
                    Edge { from: 1, symbol: 'a', to: 2 },
                    Edge { from: 2, symbol: 'a', to: 2 },
                    Edge { from: 2, symbol: 'b', to: 2 },
                ],
            },
            names: vec!["q0".to_string(), "q1".to_string(), "q2".to_string()],
        }
    }

    #[test]
    fn dfa1_test() {
        let dfa = dfa1().minimize().unwrap();
        
        assert_eq!(dfa.dfa.state_count, 2);
        assert_eq!(dfa.dfa.initial, 0);
        assert_eq!(dfa.dfa.accepting, vec![0]);
        assert_eq!(dfa.dfa.alphabet, vec!['1','2']);
        assert_eq!(dfa.dfa.edges, 
            vec![
                Edge { from: 0, symbol: '2', to: 0 },
                Edge { from: 0, symbol: '1', to: 1 },
                Edge { from: 1, symbol: '1', to: 0 },
                Edge { from: 1, symbol: '2', to: 1 }
            ]
        );       
    }

    #[test]
    fn dfa2_test() {
        let dfa = dfa2_already_minimized().minimize().unwrap();
        
        assert_eq!(dfa.dfa.state_count, 3);
        assert_eq!(dfa.dfa.initial, 0);
        assert_eq!(dfa.dfa.accepting, vec![2]);
        assert_eq!(dfa.dfa.alphabet, vec!['0','1','3']);
        assert_eq!(dfa.dfa.edges, 
            vec![
                Edge { from: 0, symbol: '3', to: 0 },
                Edge { from: 0, symbol: '0', to: 1 },
                Edge { from: 0, symbol: '1', to: 1 },
                Edge { from: 1, symbol: '1', to: 0 },
                Edge { from: 1, symbol: '3', to: 0 },
                Edge { from: 1, symbol: '0', to: 2 },
                Edge { from: 2, symbol: '3', to: 1 },
                Edge { from: 2, symbol: '0', to: 2 },
                Edge { from: 2, symbol: '1', to: 2 },
            ]
        );       
    }

    #[test]
    fn dfa3_test() {
        let dfa = dfa3_all_states_equivalent().minimize().unwrap();
        
        assert_eq!(dfa.dfa.state_count, 1);
        assert_eq!(dfa.dfa.initial, 0);
        assert_eq!(dfa.dfa.accepting, vec![0]);
        assert_eq!(dfa.dfa.alphabet, vec!['1','2']);
        assert_eq!(dfa.dfa.edges, 
            vec![
                Edge { from: 0, symbol: '1', to: 0 },
                Edge { from: 0, symbol: '2', to: 0 },
            ]
        );       
    }

    #[test]
    fn dfa4_test() {
        let dfa = dfa4().minimize().unwrap();
        println!("{:#?}", dfa);

        assert_eq!(dfa.dfa.state_count, 3);
        assert_eq!(dfa.dfa.initial, 0);
        assert_eq!(dfa.dfa.accepting, vec![1]);
        assert_eq!(dfa.dfa.alphabet, vec!['1','2']);
        assert_eq!(dfa.dfa.edges, 
            vec![
                Edge { from: 0, symbol: '1', to: 1 },
                Edge { from: 0, symbol: '2', to: 2 },
                Edge { from: 1, symbol: '1', to: 1 },
                Edge { from: 1, symbol: '2', to: 2 },
                Edge { from: 2, symbol: '1', to: 2 },
                Edge { from: 2, symbol: '2', to: 2 },
            ]
        );       
    }

    #[test]
    fn dfa_incomplete_test() {
        let result = dfa_incomplete().minimize();
        assert!(matches!(result, Err(MinimizationError::IncompleteInput)));
    }
}