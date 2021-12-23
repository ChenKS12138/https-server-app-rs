use std::{cell::RefCell, rc::Rc};

use self::state::AppState;

mod middleware;
mod state;
mod ui;

pub fn run(
    cert: Option<&str>,
    key: Option<&str>,
    bind: Option<&str>,
    root: Option<&str>,
    enable_gui: bool,
) {
    let state = Rc::new(RefCell::new(AppState::new()));
    if enable_gui {
        ui::launch(state);
    } else {
        let mut state = state.borrow_mut();
        state.root_directory = root.and_then(|s| Some(String::from(s)));
        state.server.cert = cert.and_then(|s| Some(String::from(s)));
        state.server.key = key.and_then(|s| Some(String::from(s)));
        state.server.bind_addr = bind.and_then(|s| Some(String::from(s)));
        let root_directory =
            String::from(state.root_directory.clone().unwrap_or(String::from(".")));
        state
            .server
            .launch(middleware::static_middleware(root_directory))
            .unwrap();
        loop {}
    }
}
