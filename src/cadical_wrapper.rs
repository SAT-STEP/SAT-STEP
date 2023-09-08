use cadical::Callbacks;

pub struct CadicalCallbackWrapper {}

impl CadicalCallbackWrapper {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callbacks for CadicalCallbackWrapper {
    // when `solve` is called
    fn started(&mut self) {
        println!("Cadical started SAT solving!");
    }

    // called by the solver to check if it should terminate
    fn terminate(&mut self) -> bool {
        false
    }

    // Returns the maximum length of clauses to be passed to `learn`. This
    // methods will be called only once when `set_callbacks` is called.
    fn max_length(&self) -> i32 {
        i32::max_value()
    }

    // called by the solver when a new derived clause is learnt
    fn learn(&mut self, _clause: &[i32]) {
        // println!("Learnt: {:?}", clause);
    }
}
