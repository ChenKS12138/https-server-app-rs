use handlebars::Handlebars;
use serde_json::json;
use std::{borrow::Borrow,  fs, path::{Path, PathBuf},  sync::Arc};

use crate::infra::http::{
    form_data::FormData,
    message::{HandleFn, HttpMessage,  Response},
    method::{self, Method},
    mime, status,
};

#[derive(PartialEq)]
enum DirEntryType {
    File,
    Dir,
    Symlink,
    Unknown,
}

pub fn static_middleware(root: String) -> HandleFn {
    let mut reg = Handlebars::new();
    reg.register_template_string(
        "layout",
        "<!DOCTYPE html>
<html lang=\"en\">
<head>
    <meta charset=\"UTF-8\">
    <meta http-equiv=\"X-UA-Compatible\" content=\"IE=edge\">
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
    <title>{{ title }}</title>
</head>
    <body>
        {{> @partial-block }}
    </body>
</html>",
    )
    .unwrap();
    reg.register_template_string(
        "not_found",
        "{{#> layout title=\"Not Found\" }}
<h1>Not Found {{path}}</h1>        
{{/layout}}",
    )
    .unwrap();
    reg.register_template_string(
        "index",
        "{{#> layout title=path }}
<script>
function check(message){
    return function(next,args){
        if(confirm(message)) {
            next(args);
        }
    }
}
function deleteFile(filepath) {
    var xhr = new XMLHttpRequest();
    if(onload) {
        xhr.onload = onload
    }
    xhr.open(\"delete\",filepath,true);
    xhr.send(null);
    xhr.onload = function(){
        location.reload();
    }
}
function uploadFile(filepath,file,onload){
    var xhr = new XMLHttpRequest();
    if(onload) {
        xhr.onload = onload
    }
    xhr.open(\"post\",filepath,true);
    xhr.setRequestHeader(\"Content-Type\",\"multipart/form-data; boundary=----WebKitFormBoundaryyb1zYhTI38xpQxBK\");
    var formData = new FormData();
    formData.append(\"file\", file);
    xhr.send(formData);
}
</script>
<h1>Index {{path}}</h1>
<ul>
    <li>
        <a href=\"../\">../</a>
    </li>
    {{#each files as |f|}}
        <li>
            <button onclick=\"deleteFile('{{f.[1]}}')\" >delete</button>
            <img width=\"20\" src=\"{{f.[2]}}\" />
            <a href=\"{{f.[1]}}\" >{{f.[0]}}</a>
        </li>
    {{/each}}
</ul>
<input id=\"file\" type=\"file\" />
<button id=\"upload\" disabled >上传</button>
<script>
var fileInput = document.getElementById(\"file\");
var uploadBtn = document.getElementById(\"upload\")
fileInput.addEventListener(\"change\",function(){
    uploadBtn.disabled=false;
});
uploadBtn.addEventListener(\"click\",function(){
    var file = fileInput.files.length && fileInput.files[0];
    if(!file) return;
    var prefix = \"{{path}}\".replace(/\\/$/,\"\");
    uploadFile(prefix+\"/\"+file.name,file, function() {
        fileInput.value = '';
        location.reload();
    });
});
</script>
{{/layout}}",
    )
    .unwrap();
    reg.register_template_string(
        "unknown",
        "{{#> layout title=\"Not Found\" }}
<h1>Unknown Forbidden {{path}}</h1>
{{/layout}}",
    )
    .unwrap();

    Box::new(Arc::new(move |request| -> Response {
        let request = (*request).borrow();
        match method::get_methods(request.method.as_str()) {
            Some(method) => match method {
                Method::Get => {
                    let request = (*request).borrow();
                    let index_path = request.path.clone().replace("../", "").to_string();
                    let current_path = root.clone().to_string() + index_path.as_str();
                    let mut response = Response::new();
                    response.set_header("Content-Type", "text/html;utf-8");
                    match fs::metadata(&current_path) {
                        Err(_) => {
                            response.set_code(status::NOT_FOUND);
                            let body = reg
                                .render("not_found", &json!({ "path": index_path }))
                                .unwrap();
                            response.set_body(&Vec::from(body.as_bytes()));
                            response
                        }
                        Ok(info) => {
                            if info.is_dir() {
                                type DirEntryInfo = (String, String, &'static str,bool);
                                let files = fs::read_dir(&current_path).unwrap();
                                let files: Vec<DirEntryInfo> = files
                                    .into_iter()
                                    .map(|f| -> DirEntryInfo {
                                        let f = f.unwrap();
                                        let file_type = f.file_type();
                                        // let metadata = f.metadata();
                                        let file_name = f.file_name();
            
                                        let file_type = file_type.unwrap();
                                        let file_type = if file_type.is_file() {
                                            DirEntryType::File
                                        } else if file_type.is_dir() {
                                            DirEntryType::Dir
                                        } else if file_type.is_symlink() {
                                            DirEntryType::Symlink
                                        } else {
                                            DirEntryType::Unknown
                                        };
                                        let file_name = file_name.to_str().unwrap_or("");
                                        (
                                            String::from(file_name),
                                            String::from(
                                                Path::new("/")
                                                    .join(&index_path)
                                                    .join(file_name)
                                                    .to_str()
                                                    .unwrap_or(""),
                                            ),
                                            match file_type {
                                                DirEntryType::File => {
                                                    "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAYAAADgdz34AAAABmJLR0QA/wD/AP+gvaeTAAAAcklEQVRIie3VQQqAIBCF4b/ocB6rZefUg9hGoURtZiIC8YG4cGY+VwojxgEBiA9rtwJeMPwVkpslNRE4vgbUiAVQIVagiiyN5tZZWVPLrW/rXVM6pIeuRkCcCUxgAj8BIe3ST+f6OAYEceh+tbx86h0sJ1orUB8gNFrWAAAAAElFTkSuQmCC"
                                                },
                                                DirEntryType::Dir => {
                                                    "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAyCAYAAAAeP4ixAAAABmJLR0QA/wD/AP+gvaeTAAABCUlEQVRoge2ZTQ4BMRiGHyIkTuAUVjY4g4VD+VtZuZiVAzAcQWxY6MRkNLQM/cj7JF86aTrJ+3S+ZhYFIUQoTWABZMA5oDJgnCTpE+aECRTrBPRShH3Ejmu4fuD6pVu/AdqfCvUK+S6H0gLW7p3VRxK9SKwIQBc4Et+S79YOmHE915WIAIy4teW3a1qlSAoG3L7MHb8kAqW89YRBKqXhmdsDnW8HieRQnqgVnn+prYrUyhN5z4X+EFOSH3bv5uuwW0Ai1pCINSRiDYlYQyLWkIg1JGINiVhDItaQiDUkYg2JWOMvRTI3DlIEiWToRu9Fz4w012fv1MQn0nQyqe4DY2rrJLyXoUKIey7M1NDZgDzGvQAAAABJRU5ErkJggg=="
                                                },
                                                DirEntryType::Symlink => {
                                                    "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAyCAYAAAAeP4ixAAAABmJLR0QA/wD/AP+gvaeTAAACDElEQVRoge2Yu0rEQBSGv/UComLngloJa2ch1t5gn0DQUixsvFQ+gXaCla1PIAj6ACouaqFiIbp46XwCLRQsVlGLnGC87SZzy7jkQJjM7Jl//i+TmSQLWfyveLd43ABd9QDiFCYc0JauMxjbIOVImbcwzo8Bbel2Apc4mBnbIOAIxgUIOIBxBQKWYVyCgEUY1yBgCSYNELAAkxYIGIZx8WSPe5SrCeZiDBgnL2moXpw/fTQpCupG0gtTE7xB0Yh3kYH4FrZA9om/Gx1Z8vAlVLffU5JtrbZ8aAvEfZh5DwLBF9+V9D/T1K+ZZ3OxDwDdcv5qcZxYoTojC8CL9N0A2jX1U7m1poE3gllYNKTvHGQUqEif+V9+7ydYL8cEa8hLkGbgVvJXfvl9BniOaJbxFGRScq8JoMLIAasRrW2+QngHsim5c9/a16T9DVgmAMvj8YzcSW5fpG1W2irAxLf8PHAC7Br2oS0Q3v8tUu8BHqVtSsdEQh/aAqHpDqkvSX1Lx4CCD22BcMcalPq51Md0DMT1YfIV5VDKcSl7pTw3OIZyJJmRouQ+ECzkJ6m3OvZhRKAk+TvAhZwP6xhQ9KEtUADupc+rlLtAo44JBR9GBIb4hAmPPWAEaHPow4hAgb+/20sOfRgTKALrBO9f4eI/SMGHvoChSPVT12lkIL5F3H/jfVgnVaNuZiQL3+ID+YBWVoOW43UAAAAASUVORK5CYII="
                                                },
                                                DirEntryType::Unknown => {
                                                    "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAyCAYAAAAeP4ixAAAABmJLR0QA/wD/AP+gvaeTAAACXUlEQVRoge3ZPWsUQRzH8c/FCBKIlYVYKKiIGnxAlCiC+FRoEcVKjC9AEMGARHwLYuML0BdhYyoLHwIS0SKYJmiTNPGBKEiiQfEslkQNO7s7d7e3F7gvTDPH/v6/397s7MxsTWvpwSDO4RS2YDN68QWTeINHeNni2i1hHYYxhXrB9hZXUavAbyrbMaF4gNXtKXa03fUqLuKrxkMst0843mbvK1zCzxyDMe07jrU1AfZhoUnjaW0O29oVoobxHEOTuI7d6EM/jmAU0znXjrUryJUME4u4Jnsm6sVt2cNyqCTvK/TgfaD4Es5GaF0QDvOsdZbTOR0oXMfNBvTuBLR+Y2cL/Aa5Fyg8JRkysazHu4DmSIxQT2Tho4H+h/gVqUUytB4EfjvYgF5h5qTfvYEmNAcDmq+acprDUqBofxOamwKa0zEisUPrY0rfLL5F6vxLPdC/oQnNXM5jxt+7NiNZsjfDYen/yOsYkdiZZgxbI6/J40ygf7bFdUqlV3j6Ha3QVzShF2Id+yv0FcWQ8BJlokJfhenBLdmLxsuVuSvIIcmhQ9Yy/okO2senMSJ/R/lZB+zfs7grO0AdP3CiKoNFuCE/xDxOVmWwCAOSQ4WsEM+xqyqDRXksHGBBsq/v6AcbDgiHmMWe6qzFEXrAFyXT8JohdIx6v0pTsdQk02lakL0V+opmo/QQSxo7pMgldodYlL5A/7zGDilyKStI2+kG6TS6QTqNsoLMS97gq1lTJyPLDOOD/08OQ2fHXbp0WcOkffl9UVaxMreZoc8FpdTsvhA7jW6QAoyn9JX2sP8BoWVXVMudA50AAAAASUVORK5CYII="
                                                }
                                            },
                                            file_type == DirEntryType::File || file_type == DirEntryType::Dir,
                                        )
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
                                let body = reg
                                    .render("unknown", &json!({ "path": index_path }))
                                    .unwrap();
                                response.set_body(&Vec::from(body));
                            }
                            response
                        }
                    }
                },
                Method::Post => {
                    let request = (*request).borrow();
                    let data = FormData::parse(&request.body).unwrap().unwrap();
                    let file = &data.get_part("file").unwrap().data;
                    let path = PathBuf::from(&root).canonicalize().unwrap().join(&request.path[1..]);
                    fs::write(path, file).unwrap();
                    Response::with_text(status::OK, "ok")
                },
                Method::Delete => {
                    let request = (*request).borrow();
                    let path = PathBuf::from(&root).canonicalize().unwrap().join(&request.path[1..]);
                    let file_type = fs::metadata(&path).unwrap().file_type();
                    if file_type.is_file() {
                        fs::remove_file(&path).unwrap();
                    } else if file_type.is_dir() {
                        fs::remove_dir_all(&path).unwrap();
                    } else {
                        return Response::with_text(status::NOT_ACCEPTABLE, "unknown type");
                    }
                    Response::with_text(status::OK, "ok")
                }
                _ => {
                    println!("{:?}", request);
                    Response::with_text(status::METHOD_NOT_ALLOWED, "<h1>Method Not Allowed</h1>")
                }
            },
            None => Response::with_text(status::UNPROCESSABLE_ENTITY, "<h1>Unprocessable</h1>"),
        }
    }))
}
