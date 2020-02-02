use crate::{
    formula::{Formula, Prop},
    lts::{Lts, StateId, Trans},
};
use std::collections::VecDeque;

#[derive(Clone, Copy)]
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
    pub marks: Vec<Mark>,
}

use bimap::BiMap;
use std::hash::Hash;
pub fn make_marks<T: Clone + Hash + Eq>(
    lts: &mut Lts<T>,
    prop_valuate: fn(&Prop, &T) -> bool,
    spec: Formula,
) -> Marks {
    let subformulas = spec.unfold();
    let mut marks: Vec<Mark> = vec![Mark::empty(); lts.0.len()];

    for i in 0..subformulas.len() {
        mark_impl(&mut marks, lts, prop_valuate, i, &subformulas);
    }
    Marks {
        subformulas: subformulas,
        marks: marks,
    }
}

fn mark_impl<T: Clone + Hash + Eq>(
    marks: &mut Vec<Mark>,
    lts: &mut Lts<T>,
    prop_valuate: fn(&Prop, &T) -> bool,
    i: usize,
    subformulas: &BiMap<usize, Formula>,
) {
    use Formula::*;
    match subformulas.get_by_left(&i).unwrap() {
        Prop(ref p) => update_mark(marks, lts, i, |_, trans| prop_valuate(p, &trans.state.vars)),
        Not(ref f) => {
            let f_index = subformulas.get_by_right(f).unwrap();
            let need_update_ids =
                lts.find_states(|state_id, _| !marks[state_id].is_marked(*f_index));
            for state_id in need_update_ids {
                marks[state_id].mark(i)
            }
        }
        And(box ref f1, box ref f2) => {
            let f1_index = subformulas.get_by_right(f1).unwrap();
            let f2_index = subformulas.get_by_right(f2).unwrap();
            let need_update_ids = lts.find_states(|state_id, _| {
                marks[state_id].is_marked(*f1_index) && marks[state_id].is_marked(*f2_index)
            });
            for state_id in need_update_ids {
                marks[state_id].mark(i)
            }
        }
        Or(box ref f1, box ref f2) => {
            let f1_index = subformulas.get_by_right(f1).unwrap();
            let f2_index = subformulas.get_by_right(f2).unwrap();
            let need_update_ids = lts.find_states(|state_id, _| {
                marks[state_id].is_marked(*f1_index) || marks[state_id].is_marked(*f2_index)
            });
            for state_id in need_update_ids.iter() {
                marks[*state_id].mark(i)
            }
        }
        EX(box ref f) => {
            let f_index = subformulas.get_by_right(f).unwrap();
            let need_update_ids = lts.find_states(|_, trans| {
                trans
                    .dst
                    .iter()
                    .any(|(_, succ_id)| marks[*succ_id].is_marked(*f_index))
            });
            for state_id in need_update_ids {
                marks[state_id].mark(i)
            }
        }
        EU(box ref f1, box ref f2) => {
            let f1_index = subformulas.get_by_right(f1).unwrap();
            let f2_index = subformulas.get_by_right(f2).unwrap();

            let mut need_update_ids =
                lts.find_states(|state_id, _| marks[state_id].is_marked(*f2_index));
            let mut queue = VecDeque::from(need_update_ids.clone());
            loop {
                if let Some(eu_id) = queue.pop_front() {
                    let mut founds = lts.find_states(|state_id, trans| {
                        trans.dst.iter().any(|(_, x)| *x == eu_id)
                            && marks[state_id].is_marked(*f1_index)
                            && !need_update_ids.iter().any(|x| *x == state_id) // not already exists
                    });
                    queue.append(&mut VecDeque::from(founds.clone()));
                    need_update_ids.append(&mut founds);
                } else {
                    break;
                }
            }
            for state_id in need_update_ids {
                marks[state_id].mark(i)
            }
        }
        EG(box f) => {
            let f_index = subformulas.get_by_right(f).unwrap();
            let mut need_update_ids =
                lts.find_states(|state_id, _| marks[state_id].is_marked(*f_index));

            // calc gfp
            loop {
                let unmark_ids: Vec<_> = need_update_ids
                    .clone()
                    .into_iter()
                    .filter(|state_id| {
                        let trans: &Trans<T> = lts.0.get(*state_id).unwrap();
                        !trans
                            .dst
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
                marks[state_id].mark(i)
            }
        }
        _ => unimplemented!(),
    }
}

fn update_mark<T>(
    marks: &mut Vec<Mark>,
    lts: &Lts<T>,
    i: usize,
    pred: impl Fn(StateId, &Trans<T>) -> bool,
) {
    for (state_id, trans) in lts.0.iter().enumerate() {
        if pred(state_id, trans) {
            marks[state_id].mark(i);
        }
    }
}
