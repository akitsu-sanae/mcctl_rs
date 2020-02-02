use crate::{
    formula::{Formula, Prop},
    lts::{Lts, StateEx, StateId},
};
use std::collections::{HashMap, VecDeque};

pub struct Mark(pub usize);

use std::fmt;
impl fmt::Debug for Mark {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#b}", self.0)
    }
}

impl Mark {
    pub fn empty() -> Mark {
        Mark(0)
    }
    pub fn is_marked(&self, index: usize) -> bool {
        self.0 & (1 << index) != 0
    }
    pub fn mark(&mut self, index: usize) {
        self.0 |= 1 << index;
    }
    pub fn unmark(&mut self, index: usize) {
        self.0 &= !(1 << index);
    }
}

#[derive(Debug)]
pub struct Marks {
    pub subformulas: BiMap<usize, Formula>,
    pub marks: HashMap<StateId, Mark>,
}

use bimap::BiMap;
use std::fmt::Debug;
use std::hash::Hash;
pub fn make_marks<T: Debug + Clone + Hash + Eq>(
    lts: &mut Lts<T>,
    prop_valuate: fn(&Prop, &T) -> bool,
    spec: Formula,
) -> Marks {
    let subformulas = spec.unfold();
    let mut marks: HashMap<StateId, Mark> = lts
        .0
        .iter()
        .map(|(state_id, _)| (*state_id, Mark::empty()))
        .collect();

    for i in 0..subformulas.len() {
        mark_impl(&mut marks, lts, prop_valuate, i, &subformulas);
    }
    Marks {
        subformulas: subformulas,
        marks: marks,
    }
}

fn mark_impl<T: Clone + Hash + Eq>(
    marks: &mut HashMap<StateId, Mark>,
    lts: &mut Lts<T>,
    prop_valuate: fn(&Prop, &T) -> bool,
    i: usize,
    subformulas: &BiMap<usize, Formula>,
) {
    use Formula::*;
    match subformulas.get_by_left(&i).unwrap() {
        Prop(ref p) => update_mark(marks, lts, i, |_, state_ex| {
            prop_valuate(p, &state_ex.state.vars)
        }),
        Not(ref f) => {
            let f_index = subformulas.get_by_right(f).unwrap();
            let need_update_ids =
                lts.find_states(|state_id, _| !marks.get(&state_id).unwrap().is_marked(*f_index));
            for state_id in need_update_ids {
                marks.get_mut(&state_id).unwrap().mark(i)
            }
        }
        And(box ref f1, box ref f2) => {
            let f1_index = subformulas.get_by_right(f1).unwrap();
            let f2_index = subformulas.get_by_right(f2).unwrap();
            let need_update_ids = lts.find_states(|state_id, _| {
                let mark = marks.get(&state_id).unwrap();
                mark.is_marked(*f1_index) && mark.is_marked(*f2_index)
            });
            for state_id in need_update_ids {
                marks.get_mut(&state_id).unwrap().mark(i)
            }
        }
        Or(box ref f1, box ref f2) => {
            let f1_index = subformulas.get_by_right(f1).unwrap();
            let f2_index = subformulas.get_by_right(f2).unwrap();
            let need_update_ids = lts.find_states(|state_id, _| {
                let mark = marks.get(&state_id).unwrap();
                mark.is_marked(*f1_index) || mark.is_marked(*f2_index)
            });
            for state_id in need_update_ids.iter() {
                marks.get_mut(&state_id).unwrap().mark(i)
            }
        }
        EX(box ref f) => {
            let f_index = subformulas.get_by_right(f).unwrap();
            let need_update_ids = lts.find_states(|_, state_ex| {
                state_ex
                    .transs
                    .iter()
                    .any(|(_, succ_id)| marks.get(&succ_id).unwrap().is_marked(*f_index))
            });
            for state_id in need_update_ids {
                marks.get_mut(&state_id).unwrap().mark(i)
            }
        }
        EU(box ref f1, box ref f2) => {
            let f1_index = subformulas.get_by_right(f1).unwrap();
            let f2_index = subformulas.get_by_right(f2).unwrap();

            let mut need_update_ids =
                lts.find_states(|state_id, _| marks.get(&state_id).unwrap().is_marked(*f2_index));
            let mut queue = VecDeque::from(need_update_ids.clone());
            loop {
                if let Some(eu_id) = queue.pop_front() {
                    let mut founds = lts.find_states(|state_id, state_ex| {
                        state_ex.transs.iter().any(|(_, x)| *x == eu_id)
                            && marks.get(&state_id).unwrap().is_marked(*f1_index)
                            && !need_update_ids.iter().any(|x| *x == state_id) // not already exists
                    });
                    queue.append(&mut VecDeque::from(founds.clone()));
                    need_update_ids.append(&mut founds);
                } else {
                    break;
                }
            }
            for state_id in need_update_ids {
                marks.get_mut(&state_id).unwrap().mark(i)
            }
        }
        EG(box f) => {
            let f_index = subformulas.get_by_right(f).unwrap();
            let mut need_update_ids = Vec::from(
                lts.find_states(|state_id, _| marks.get(&state_id).unwrap().is_marked(*f_index)),
            );

            // calc gfp
            loop {
                let unmark_ids: Vec<_> = need_update_ids
                    .clone()
                    .into_iter()
                    .filter(|state_id| {
                        let state_ex = lts.0.get(&state_id).unwrap();
                        !state_ex
                            .transs
                            .iter()
                            .any(|(_, next_id)| need_update_ids.contains(next_id))
                    })
                    .collect();
                if unmark_ids.is_empty() {
                    // already at fixed point
                    break;
                }
                for unmark_id in unmark_ids {
                    need_update_ids.retain(|x| *x != unmark_id);
                }
            }

            for state_id in need_update_ids {
                marks.get_mut(&state_id).unwrap().mark(i)
            }
        }
        _ => unimplemented!(),
    }
}

fn update_mark<T>(
    marks: &mut HashMap<StateId, Mark>,
    lts: &Lts<T>,
    i: usize,
    pred: impl Fn(StateId, &StateEx<T>) -> bool,
) {
    for (state_id, state_ex) in lts.0.iter() {
        if pred(*state_id, state_ex) {
            marks.get_mut(state_id).unwrap().mark(i);
        }
    }
}
