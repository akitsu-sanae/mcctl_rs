use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Location(pub String);

impl Location {
    pub fn new(s: &str) -> Self {
        Location(s.to_string())
    }
}

impl fmt::Display for Location {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label(pub String);

impl Label {
    pub fn new(s: &str) -> Self {
        Label(s.to_string())
    }
}

impl fmt::Display for Label {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

pub type Guard<T> = fn(&T) -> bool;

pub type Action<T> = fn(T) -> T;

pub struct Trans<T> {
    pub label: Label,
    pub dst: Location,
    pub guard: Guard<T>,
    pub action: Action<T>,
}

pub struct ExecUnit<T> {
    pub src: Location,
    pub transs: Vec<Trans<T>>,
}

pub type Process<T> = Vec<ExecUnit<T>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State<T> {
    pub vars: T,
    pub locations: Vec<Location>,
}

struct Mark(pub usize);

impl Mark {
    fn empty() -> Self {
        Mark(0)
    }
}

impl fmt::Debug for Mark {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#b}", self.0)
    }
}

#[derive(Debug)]
pub struct StateEx<T> {
    pub state: State<T>,
    pub transs: Vec<(Label, usize)>,
    mark: Mark,
}

impl<T> StateEx<T> {
    pub fn is_marked(&self, index: usize) -> bool {
        self.mark.0 & (1 << index) != 0
    }
    pub fn mark(&mut self, index: usize) {
        self.mark.0 |= 1 << index;
    }
    pub fn unmark(&mut self, index: usize) {
        self.mark.0 &= !(1 << index);
    }
}

pub struct Lts<T>(pub HashMap<usize, StateEx<T>>);

impl<T> Lts<T> {
    pub fn new() -> Self {
        Lts(HashMap::new())
    }
    pub fn update_mark(&mut self, pred: impl Fn(&StateEx<T>) -> bool, i: usize) {
        for (_, state_ex) in self.0.iter_mut() {
            if pred(state_ex) {
                state_ex.mark(i)
            }
        }
    }
    pub fn find_states(&self, pred: impl Fn(usize, &StateEx<T>) -> bool) -> Vec<usize> {
        let mut result = vec![];
        for (state_id, state_ex) in self.0.iter() {
            if pred(*state_id, state_ex) {
                result.push(*state_id);
            }
        }
        result
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
    transs: &Vec<Trans<T>>,
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
// TODO: don't use String as error type!
pub fn concurrent_composition<T: Clone + Hash + Eq>(
    vars: T,
    processes: Vec<Process<T>>,
) -> Result<Lts<T>, String> {
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

fn bfs<T: Clone + Hash + Eq>(
    init: State<T>,
    next_func: impl Fn(&State<T>) -> Vec<(Label, State<T>)>,
) -> Lts<T> {
    let mut lts = Lts::new();
    lts.0.insert(
        0,
        StateEx {
            state: init.clone(),
            transs: vec![],
            mark: Mark::empty(),
        },
    );
    let mut state_dict: HashMap<State<T>, usize> = HashMap::new();
    state_dict.insert(init.clone(), 0);
    let mut queue = VecDeque::new();
    queue.push_back((0, init));

    loop {
        if let Some((state_id, state)) = queue.pop_front() {
            let nexts = next_func(&state);
            let mut transs = Vec::with_capacity(nexts.len());
            for (label, next_state) in nexts {
                let next_id = if let Some(id) = state_dict.get(&next_state) {
                    *id // already exists
                } else {
                    let id = state_dict.len();
                    state_dict.insert(next_state.clone(), id);
                    queue.push_back((id, next_state.clone()));
                    id
                };
                transs.push((label, next_id));
            }
            lts.0.insert(
                state_id,
                StateEx {
                    state: state,
                    transs: transs,
                    mark: Mark::empty(),
                },
            );
        } else {
            break;
        }
    }
    lts
}
