use rust_fsm::*;

// init
// method
// blank
// url
// blank
// version
// cr
// lf
// header_field
// colon
// header_value
// body
// end
// failed

state_machine! {
    derive(Debug)
    pub RequestParser(Closed)

    Closed(Unsuccessful) => Open [SetupTimer],
    Open(TimerTriggered) => HalfOpen,
    HalfOpen => {
        Successful => Closed,
        Unsuccessful => Open [SetupTimer],
    }
}

fn foo() {
    let m: StateMachine<RequestParser> = StateMachine::new();
}
