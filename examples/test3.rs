extern crate mcctl_rs;

use mcctl_rs::{
    formula::{Formula, Prop},
    lts::Lts,
    mark,
    process::{ExecUnit, Label, Location, Process, Trans},
    viz,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Vars {
    x: i32,
}

impl Vars {
    fn init() -> Self {
        Vars { x: 1 }
    }
}

use std::fmt;
impl fmt::Display for Vars {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "x={}", self.x)
    }
}

fn main() {
    let process_p: Process<Vars> = vec![ExecUnit {
        src: Location::new("S"),
        transs: vec![
            Trans {
                label: Label::new(""),
                dst: Location::new("S"),
                guard: |vars| vars.x < 16,
                action: |mut vars| {
                    vars.x *= 2;
                    vars
                },
            },
            Trans {
                label: Label::new(""),
                dst: Location::new("S"),
                guard: |vars| vars.x < 16,
                action: |mut vars| {
                    vars.x = vars.x * 2 + 1;
                    vars
                },
            },
        ],
    }];

    let processes = vec![process_p];
    let mut lts = Lts::concurrent_composition(Vars::init(), processes).unwrap();

    fn prop_valuate(prop: &Prop, vars: &Vars) -> bool {
        match prop.as_str() {
            "x=1 or x%2=0" => vars.x == 1 || vars.x % 2 == 0,
            "x>=16 and x%4=0" => vars.x >= 16 && vars.x % 4 == 0,
            _ => panic!(),
        }
    }

    let spec = Formula::EU(
        Box::new(Formula::Prop("x=1 or x%2=0".to_string())),
        Box::new(Formula::Prop("x>=16 and x%4=0".to_string())),
    );

    let marks = mark::make_marks(&mut lts, prop_valuate, spec);
    viz::lts("test3.dot", &lts, marks);
}
