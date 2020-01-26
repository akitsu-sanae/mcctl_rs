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

pub type Lts<T> = HashMap<usize, StateEx<T>>;

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
use std::fmt::Debug;
// TODO: don't use String as error type!
pub fn concurrent_composition<T: Clone + Debug + Hash + Eq>(
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
    let table = bfs(s0, next_func);
    Ok(conv_lts(table))
}

fn conv_lts<T: Clone + Hash + Eq>(
    table: HashMap<State<T>, (usize, Vec<(Label, State<T>)>)>,
) -> Lts<T> {
    let mut lts = Lts::new();
    for (state, (id, nexts)) in table.iter() {
        let nexts = nexts
            .iter()
            .map(|(label, dst)| {
                let (dst_id, _) = table.get(&dst).unwrap();
                (label.clone(), *dst_id)
            })
            .collect();
        lts.insert(
            *id,
            StateEx {
                state: state.clone(),
                transs: nexts,
                mark: Mark::empty(),
            },
        );
    }
    lts
}

fn bfs<T: Clone + Hash + Eq>(
    init: State<T>,
    next_func: impl Fn(&State<T>) -> Vec<(Label, State<T>)>,
) -> HashMap<State<T>, (usize, Vec<(Label, State<T>)>)> {
    let mut table = HashMap::new();
    table.insert(init.clone(), (0, vec![]));
    let mut queue = VecDeque::new();
    queue.push_back((init, 0));

    loop {
        if let Some((state, id)) = queue.pop_front() {
            let nexts = next_func(&state);
            table.insert(state, (id, nexts.clone()));
            for (_, dst) in nexts {
                if table.get(&dst).is_none() {
                    let id = table.len();
                    table.insert(dst.clone(), (id, vec![]));
                    queue.push_back((dst.clone(), id));
                }
            }
        } else {
            break;
        }
    }
    table
}
