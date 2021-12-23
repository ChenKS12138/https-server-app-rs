use crate::app;

pub fn boost() {
    let matches = clap::App::new("http-server-app")
        .about("https server app")
        .arg(
            clap::Arg::with_name("cert")
                .short("c")
                .long("cert")
                .help("server tls/ssl certificate")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("key")
                .short("k")
                .long("key")
                .help("server tls/ssl key")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("bind")
                .short("b")
                .long("bind")
                .help("bind address")
                .default_value("0.0.0.0:443")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("root")
                .short("r")
                .long("root")
                .help("root directory")
                .default_value(".")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("no-gui")
                .short("n")
                .long("no-gui")
                .help("disable gui")
                .takes_value(false),
        )
        .get_matches();
    app::run(
        matches.value_of("cert"),
        matches.value_of("key"),
        matches.value_of("bind"),
        matches.value_of("root"),
        !matches.is_present("no-gui"),
    );
}
