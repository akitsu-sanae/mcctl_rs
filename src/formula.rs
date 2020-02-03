use bimap::BiMap;

pub type Prop = String;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Formula {
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
    pub fn unfold(self) -> BiMap<usize, Formula> {
        fn unfold_impl(f: Formula, mut acc: BiMap<usize, Formula>) -> BiMap<usize, Formula> {
            use Formula::*;
            match f {
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
                Impl(box f1, box f2) => {
                    let acc = unfold_impl(f1.clone(), acc);
                    let mut acc = unfold_impl(f2.clone(), acc);
                    acc.insert(acc.len(), Impl(Box::new(f1), Box::new(f2)));
                    acc
                }
                EX(box f) => {
                    let mut acc = unfold_impl(f.clone(), acc);
                    acc.insert(acc.len(), EX(Box::new(f)));
                    acc
                }
                EU(box f1, box f2) => {
                    let acc = unfold_impl(f1.clone(), acc);
                    let mut acc = unfold_impl(f2.clone(), acc);
                    acc.insert(acc.len(), EU(Box::new(f1), Box::new(f2)));
                    acc
                }
                EG(box f) => {
                    let mut acc = unfold_impl(f.clone(), acc);
                    acc.insert(acc.len(), EG(Box::new(f)));
                    acc
                }
            }
        }
        unfold_impl(self, BiMap::new())
    }
}

use std::fmt;
impl fmt::Display for Formula {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use Formula::*;
        match self {
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
