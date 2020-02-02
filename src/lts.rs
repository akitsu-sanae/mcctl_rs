use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

use crate::process::{self, ExecUnit, Label, Location, Process};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State<T> {
    pub vars: T,
    pub locations: Vec<Location>,
}

pub type StateId = usize;

#[derive(Debug)]
pub struct Trans<T> {
    pub state: State<T>,
    pub dst: Vec<(Label, StateId)>,
}

#[derive(Debug)]
pub struct Lts<T>(pub Vec<Trans<T>>);

impl<T: Clone + Hash + Eq> Lts<T> {
    pub fn new() -> Self {
        Lts(vec![])
    }

    pub fn find_states(&self, pred: impl Fn(usize, &Trans<T>) -> bool) -> Vec<usize> {
        let mut result = vec![];
        for (state_id, trans) in self.0.iter().enumerate() {
            if pred(state_id, trans) {
                result.push(state_id);
            }
        }
        result
    }

    // TODO: don't use String as error type!
    pub fn concurrent_composition(vars: T, processes: Vec<Process<T>>) -> Result<Lts<T>, String> {
        let s0 = State {
            vars: vars,
            locations: {
                let locations: Result<Vec<Location>, _> =
                    processes.iter().map(pick_init_location).collect();
                locations?
            },
        };
        let next_func = |state: &State<T>| -> Vec<(Label, State<T>)> {
            let mut next = vec![];
            for (i, process) in processes.iter().enumerate() {
                let location = state.locations[i].clone();
                let transs = &process
                    .iter()
                    .find(|exec: &&ExecUnit<T>| exec.src == location)
                    .unwrap()
                    .transs;
                calc_transitions_from(&mut next, &location, state, transs);
            }
            return next;
        };
        Ok(bfs(s0, next_func))
    }
}

fn pick_init_location<T>(p: &Process<T>) -> Result<Location, String> {
    match p.first() {
        Some(exec) => Ok(exec.src.clone()),
        None => Err(format!("wrong process: no transition")),
    }
}

fn calc_transitions_from<T: Clone>(
    next: &mut Vec<(Label, State<T>)>,
    current: &Location,
    state: &State<T>,
    transs: &Vec<process::Trans<T>>,
) {
    for trans in transs {
        if (trans.guard)(&state.vars) {
            let locations = state
                .locations
                .iter()
                .map(|l: &Location| {
                    if l == current {
                        trans.dst.clone()
                    } else {
                        l.clone()
                    }
                })
                .collect();
            let dst_state = State {
                vars: (trans.action)(state.vars.clone()),
                locations: locations,
            };
            next.push((trans.label.clone(), dst_state));
        }
    }
}

fn bfs<T: Clone + Hash + Eq>(
    init: State<T>,
    next_func: impl Fn(&State<T>) -> Vec<(Label, State<T>)>,
) -> Lts<T> {
    let mut lts = Lts::new();
    let mut state_dict = HashMap::new();
    state_dict.insert(init.clone(), 0);
    let mut queue = VecDeque::new();
    queue.push_back((0, init));

    loop {
        if let Some((state_id, state)) = queue.pop_front() {
            let nexts = next_func(&state);
            let mut dst = Vec::with_capacity(nexts.len());
            for (label, next_state) in nexts {
                let next_id = if let Some(id) = state_dict.get(&next_state) {
                    *id // already exists
                } else {
                    let id = state_dict.len();
                    state_dict.insert(next_state.clone(), id);
                    queue.push_back((id, next_state.clone()));
                    id
                };
                dst.push((label, next_id));
            }
            lts.0.insert(
                state_id,
                Trans {
                    state: state,
                    dst: dst,
                },
            );
        } else {
            break;
        }
    }
    lts
}
