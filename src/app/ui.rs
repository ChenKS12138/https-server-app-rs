use crate::app::middleware;
use crate::infra::https::HttpsServer;

pub fn launch() {
    let entry = middleware::static_middleware(String::from("."));

    use fltk::{app, button::Button, frame::Frame, prelude::*, window::Window};
    let app = fltk::app::App::default().with_scheme(app::Scheme::Gtk);
    let mut started = false;
    let mut wind = Window::new(100, 100, 400, 300, "基于OpenSSL的安全Web服务器程序");
    let mut frame = Frame::new(0, 0, 400, 200, "stopped");
    let mut but = Button::new(160, 210, 80, 40, "Start Server");
    wind.end();
    wind.show();
    let mut server = HttpsServer::new(
        String::from("0.0.0.0:443"),
        String::from("/Users/cattchen/Codes/https-server-app/script/server_bundle.cert.pem"),
        String::from("/Users/cattchen/Codes/https-server-app/script/server.private.pem"),
    );
    but.set_callback(move |but| {
        if started {
            frame.set_label("stopped");
            but.set_label("Start Server");
            server.shutdown().unwrap();
        } else {
            frame.set_label("started");
            but.set_label("Stop Server");
            server.launch(entry.to_owned()).unwrap();
        }
        started = !started;
    }); // the closure capture is mutable borrow to our button
    app.run().unwrap();
}
