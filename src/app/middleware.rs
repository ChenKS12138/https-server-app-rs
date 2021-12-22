use handlebars::Handlebars;
use serde_json::json;
use std::{fs, path::Path, sync::Arc};

use crate::infra::http::{
    message::{HandleFn, HttpMessage, Response},
    mime, status,
};

pub fn static_middleware(root: String) -> HandleFn {
    let mut reg = Handlebars::new();
    reg.register_template_string("forbidden", "<h1>Path Forbidden {{path}}</h1>")
        .unwrap();
    reg.register_template_string(
        "index",
        "<h1>Index {{path}}</h1>
<ul>
    <li>
        <a href=\"../\">../</a>
    </li>
    {{#each files as |f|}}
        <li>
            <a href=\"javascript: location.pathname='{{../path}}/{{f}}' \" >{{f}}</a>
        </li>
    {{/each}}
</ul>
",
    )
    .unwrap();
    reg.register_template_string("known", "<h1>Known Forbidden {{path}}</h1>")
        .unwrap();

    Box::new(Arc::new(move |request| -> Response {
        let index_path = request.path.clone().replace("../", "").to_string();
        let current_path = root.clone() + index_path.as_str();
        let mut response = Response::new();
        response.set_header("Content-Type", "text/html");
        match fs::metadata(&current_path) {
            Err(_) => {
                response.set_code(status::FORBIDDEN);
                let body = reg
                    .render("forbidden", &json!({ "path": index_path }))
                    .unwrap();
                response.set_body(&Vec::from(body.as_bytes()));
                response
            }
            Ok(info) => {
                if info.is_dir() {
                    let files = fs::read_dir(&current_path).unwrap();
                    let files: Vec<String> = files
                        .into_iter()
                        .map(|f| -> String {
                            String::from(f.unwrap().file_name().to_str().unwrap())
                        })
                        .collect();
                    let body = reg
                        .render("index", &json!({ "path":index_path, "files": files }))
                        .unwrap();
                    response.set_body(&Vec::from(body));
                } else if info.is_file() {
                    let path = Path::new(&current_path);
                    let body = fs::read_to_string(&current_path).unwrap();
                    let content_type = mime::get_mime(if let Some(str) = path.extension() {
                        str.to_str().unwrap_or("")
                    } else {
                        ""
                    });
                    if content_type.is_none() {
                        response.set_header(
                            "Content-Disposition",
                            format!(
                                "attachment; filename=\"{}\"",
                                if let Some(str) = path.file_name() {
                                    str.to_str().unwrap_or("")
                                } else {
                                    ""
                                }
                            )
                            .as_str(),
                        );
                    }
                    response.set_header(
                        "Content-Type:",
                        content_type.unwrap_or("application/octet-stream"),
                    );

                    response.set_body(&Vec::from(body));
                } else {
                    let body = reg.render("known", &json!({ "path": index_path })).unwrap();
                    response.set_body(&Vec::from(body));
                }
                response
            }
        }
    }))
}
