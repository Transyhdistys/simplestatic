use args::MainArgs;
use errors::{GenericError, PathError};
use std::fs;
use std::io;
use std::path::PathBuf;
use template::Template;
use warp::http::header::{HeaderMap, HeaderValue};
use warp::Filter;

mod args;
mod errors;
mod template;

#[tokio::main]
async fn main() {
    let args: MainArgs = argh::from_env();

    let html_path = args.clone().html.unwrap_or(PathBuf::from("index.html"));

    let template = match get_files(&html_path, &args) {
        Ok((html, css, js)) => match Template::new(html, css, js) {
            Ok(template) => template,
            Err(e) => panic!("Error: {}", e),
        },
        Err(e) => panic!("Error: {}", e),
    };

    //fs::write("test.html", &template.text).unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("text/html"));
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static("default-src 'self'; script-src none; image-src 'https://*.example.com' 'https://foobar.com'; font-src 'jokusource' 'joku toinen source' 'etc.'"),
    );

    let server = warp::any()
        .and(warp::header("Host"))
        .map(move |host: String| template.render(host))
        .with(warp::reply::with::headers(headers));

    warp::serve(server).run(([127, 0, 0, 1], 3030)).await;
}

fn get_files(
    html_path: &PathBuf,
    args: &MainArgs,
) -> Result<(String, Option<Vec<String>>, Option<Vec<String>>), GenericError> {
    let metadata = fs::metadata(&html_path)?;
    if metadata.is_dir() {
        Err(PathError::new(
            html_path.clone(),
            "html path must not be a directory",
        ))?
    }
    let html_file = fs::read_to_string(html_path)?;

    let css_files = if let Some(css_path) = &args.css {
        Some(handle_dir_or_file(&css_path)?)
    } else {
        None
    };
    let js_files = if let Some(js_path) = &args.js {
        Some(handle_dir_or_file(&js_path)?)
    } else {
        None
    };

    Ok((html_file, css_files, js_files))
}

fn handle_dir_or_file(path: &PathBuf) -> Result<Vec<String>, io::Error> {
    let metadata = fs::metadata(&path)?;
    let mut list = Vec::new();

    if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry_path = entry?.path();
            if entry_path.is_dir() {
                list.extend(handle_dir_or_file(&entry_path)?.drain(..));
            } else {
                list.push(fs::read_to_string(&entry_path)?);
            }
        }
    } else {
        list.push(fs::read_to_string(path)?);
    }

    Ok(list)
}