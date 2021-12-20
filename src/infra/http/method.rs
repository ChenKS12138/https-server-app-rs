macro_rules! generate_methods {
    ($($name:ident,$value:expr)+) => {
        #[allow(dead_code)]
        pub enum Method {
            $($name,)+
        }

        #[allow(dead_code)]
        pub fn get_methods(method:&str) -> Option<Method> {
            match method {
                $($value => Some(Method::$name),)+
                _ => None
            }
        }
    };
}

generate_methods! {
    Get, "GET"
    Post, "POST"
    Put, "PUT"
    Delete, "DELETE"
    Options, "OPTIONS"
    Head, "HEAD"
    Trace, "TRACE"
    Connect, "CONNECT"
    Patch, "PATCH"
}
