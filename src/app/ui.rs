use std::cell::RefCell;
use std::rc::Rc;

use fltk::dialog::{self, NativeFileChooser};

use crate::app::middleware;
use crate::app::state::AppState;

pub fn launch(state: Rc<RefCell<AppState>>) {
    use fltk::{app, button::Button, frame::Frame, prelude::*, window::Window};
    let app = fltk::app::App::default().with_scheme(app::Scheme::Gtk);
    let mut wind = Window::new(100, 100, 400, 300, "基于OpenSSL的安全Web服务器程序");

    let _ = Frame::new(100, 50, 30, 30, "Cert:");
    let mut but_cert = Button::new(140, 50, 150, 30, "Click To Chose");
    let cloned_state = state.clone();
    but_cert.set_callback(move |but| {
        let mut chooser_cert = NativeFileChooser::new(dialog::FileDialogType::BrowseFile);
        chooser_cert.show();
        but.set_label(
            chooser_cert
                .filename()
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or("Click To Chose"),
        );
        cloned_state.borrow_mut().server.cert = chooser_cert
            .filename()
            .as_os_str()
            .to_str()
            .and_then(|str| Some(str.to_string()));
    });

    let _ = Frame::new(80, 90, 30, 30, "Private Key:");
    let mut but_key = Button::new(140, 90, 150, 30, "Click To Chose");
    let cloned_state = state.clone();
    but_key.set_callback(move |but| {
        let mut chooser_cert = NativeFileChooser::new(dialog::FileDialogType::BrowseFile);
        chooser_cert.show();
        but.set_label(
            chooser_cert
                .filename()
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or("Click To Chose"),
        );

        cloned_state.borrow_mut().server.key = chooser_cert
            .filename()
            .as_os_str()
            .to_str()
            .and_then(|str| Some(str.to_string()));
    });

    let _ = Frame::new(60, 130, 30, 30, "Root Directory:");
    let mut but_root = Button::new(140, 130, 150, 30, "Click To Chose");
    let cloned_state = state.clone();
    but_root.set_callback(move |but| {
        let mut chooser_cert = NativeFileChooser::new(dialog::FileDialogType::BrowseDir);
        chooser_cert.show();
        but.set_label(
            chooser_cert
                .filename()
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or("Click To Chose"),
        );
        cloned_state.borrow_mut().root_directory = chooser_cert
            .filename()
            .as_os_str()
            .to_str()
            .and_then(|str| Some(str.to_string()));
    });

    let mut but_start = Button::new(120, 170, 150, 30, "Start");
    let cloned_state = state.clone();
    but_start.set_callback(move |but| {
        let mut state = cloned_state.borrow_mut();
        match state.server.status {
            crate::infra::https::HttpsServerStatus::Started => {
                state.server.shutdown().unwrap();
                but.set_label("Start");
            }
            crate::infra::https::HttpsServerStatus::Stopped => {
                if state.server.cert.is_none()
                    || state.server.key.is_none()
                    || state.root_directory.is_none()
                {
                    dialog::alert_default("Arguments Not Ready");
                    return;
                }
                state.server.bind_addr = Some(String::from("0.0.0.0:443"));
                let root_directory = state
                    .root_directory
                    .clone()
                    .unwrap_or(String::from("."))
                    .clone();
                state
                    .server
                    .launch(middleware::static_middleware(root_directory))
                    .unwrap();
                but.set_label("Stop");
            }
            _ => {
                dialog::alert_default("Server Busy");
            }
        }
    });

    wind.end();
    wind.show();
    app.run().unwrap();
}
