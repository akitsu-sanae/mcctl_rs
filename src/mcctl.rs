use crate::ddsv::Lts;

pub type Prop = String;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Formula {
    False,
    True,
    Prop(Prop),
    Not(Box<Formula>),
    And(Box<Formula>, Box<Formula>),
    Or(Box<Formula>, Box<Formula>),
    Impl(Box<Formula>, Box<Formula>),
    EX(Box<Formula>),
    EU(Box<Formula>, Box<Formula>),
    EG(Box<Formula>),
}

impl Formula {
    fn unfold(self) -> BiMap<usize, Formula> {
        fn unfold_impl(f: Formula, mut acc: BiMap<usize, Formula>) -> BiMap<usize, Formula> {
            use Formula::*;
            match f {
                False => {
                    acc.insert(acc.len(), False);
                    acc
                }
                True => {
                    acc.insert(acc.len(), True);
                    acc
                }
                Prop(p) => {
                    acc.insert(acc.len(), Prop(p));
                    acc
                }
                Not(box f) => {
                    let mut acc = unfold_impl(f.clone(), acc);
                    acc.insert(acc.len(), Not(Box::new(f)));
                    acc
                }
                And(box f1, box f2) => {
                    let acc = unfold_impl(f1.clone(), acc);
                    let mut acc = unfold_impl(f2.clone(), acc);
                    acc.insert(acc.len(), And(Box::new(f1), Box::new(f2)));
                    acc
                }
                Or(box f1, box f2) => {
                    let acc = unfold_impl(f1.clone(), acc);
                    let mut acc = unfold_impl(f2.clone(), acc);
                    acc.insert(acc.len(), Or(Box::new(f1), Box::new(f2)));
                    acc
                }
                EX(box f) => {
                    let mut acc = unfold_impl(f.clone(), acc);
                    acc.insert(acc.len(), EX(Box::new(f)));
                    acc
                }
                _ => unimplemented!(),
            }
        }
        let mut acc = BiMap::new();
        acc.insert(0, Formula::True);
        unfold_impl(self, BiMap::new())
    }
}

use std::fmt;
impl fmt::Display for Formula {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use Formula::*;
        match self {
            False => write!(fmt, "false"),
            True => write!(fmt, "true"),
            Prop(ref p) => write!(fmt, "{}", p),
            Not(ref f) => write!(fmt, "(not {})", f),
            And(ref lhs, ref rhs) => write!(fmt, "(and {} {})", lhs, rhs),
            Or(ref lhs, ref rhs) => write!(fmt, "(or {} {})", lhs, rhs),
            Impl(ref lhs, ref rhs) => write!(fmt, "(impl {} {})", lhs, rhs),
            EX(ref f) => write!(fmt, "(EX {})", f),
            EU(ref f, ref g) => write!(fmt, "(EU {} {})", f, g),
            EG(ref f) => write!(fmt, "(EG {})", f),
        }
    }
}

use bimap::BiMap;
use std::fmt::Debug;
pub fn mark<T: Debug>(
    lts: &mut Lts<T>,
    prop_valuate: fn(&Prop, &T) -> bool,
    spec: Formula,
) -> BiMap<usize, Formula> {
    let subformulas = spec.unfold();
    // println!("subformulas: {:?}", subformulas);

    for i in 0..subformulas.len() {
        /*
        println!("-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-");
        println!(
            "subformula at {} ({:?})",
            i,
            subformulas.get_by_left(&i).unwrap()
        ); */
        mark_impl(lts, prop_valuate, i, &subformulas);
        // println!("lts: {:?}", lts);
    }
    subformulas
}

fn mark_impl<T: Debug>(
    lts: &mut Lts<T>,
    prop_valuate: fn(&Prop, &T) -> bool,
    i: usize,
    subformulas: &BiMap<usize, Formula>,
) {
    use Formula::*;
    match subformulas.get_by_left(&i).unwrap() {
        False => (),
        True => {
            for (i, state_ex) in lts.iter_mut() {
                state_ex.mark(*i);
            }
        }
        Prop(ref p) => {
            for (_, state_ex) in lts.iter_mut() {
                if prop_valuate(p, &state_ex.state.vars) {
                    state_ex.mark(i);
                }
            }
        }
        Not(ref f) => {
            let f_index = subformulas.get_by_right(f).unwrap();
            for (_, state_ex) in lts.iter_mut() {
                if !state_ex.is_marked(*f_index) {
                    state_ex.mark(i)
                }
            }
        }
        And(box ref f1, box ref f2) => {
            let f1_index = subformulas.get_by_right(f1).unwrap();
            let f2_index = subformulas.get_by_right(f2).unwrap();
            for (_, state_ex) in lts.iter_mut() {
                if state_ex.is_marked(*f1_index) && state_ex.is_marked(*f2_index) {
                    state_ex.mark(i)
                }
            }
        }
        Or(box ref f1, box ref f2) => {
            let f1_index = subformulas.get_by_right(f1).unwrap();
            let f2_index = subformulas.get_by_right(f2).unwrap();
            for (_, state_ex) in lts.iter_mut() {
                if state_ex.is_marked(*f1_index) || state_ex.is_marked(*f2_index) {
                    state_ex.mark(i)
                }
            }
        }
        EX(box ref f) => {
            let f_index = subformulas.get_by_right(f).unwrap();
            let need_update_ids = {
                let mut state_ids = vec![];
                for (current_id, state_ex) in lts.iter() {
                    for (_, succ_id) in state_ex.transs.iter() {
                        if lts.get(&succ_id).unwrap().is_marked(*f_index) {
                            state_ids.push(*current_id);
                            break;
                        }
                    }
                }
                state_ids
            };
            for state_id in need_update_ids {
                lts.get_mut(&state_id).unwrap().mark(i)
            }
        }
        _ => unimplemented!(),
    }
}
