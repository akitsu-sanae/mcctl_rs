extern crate mcctl_rs;

use mcctl_rs::ddsv::{self, ExecUnit, Label, Location, Process, Trans};
use mcctl_rs::mcctl::{self, Formula, Prop};
use mcctl_rs::viz;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Vars {
    x: i32,
}

impl Vars {
    fn init() -> Self {
        Vars { x: 1 }
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
    let mut lts = ddsv::concurrent_composition(Vars::init(), processes).unwrap();

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

    let fs = mcctl::mark(&mut lts, prop_valuate, spec);

    let show_vars = |vars: &Vars| format!("x={}", vars.x);

    viz::lts("test3.dot", show_vars, &lts, fs);
}
