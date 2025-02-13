use actix_web::{get, App, HttpResponse, HttpServer, Responder, Error};
use actix_web::web;
use std::fs::File;
use pbf_font_tools::protobuf::Message;
use std::io::BufReader;
use std::path::Path;

#[get("/fonts/{glyph}/{range}.pbf")]
async fn get_font(path: web::Path<(String, String)>) -> Result<HttpResponse, Error> {
    let glyph = &path.0;
    let range = &path.1;

    let mut split_into_glyphs = glyph.split(",")
    .map(|x| x.to_string()).collect::<Vec<String>>();

    if split_into_glyphs.len() == 0 {
        return Ok(HttpResponse::BadRequest().body("No glyphs specified"));
    }

    split_into_glyphs.push("Arial-Unicode-Regular".to_string());

    let range_interval = range.split("-");

    let (range_start, range_end) = match range_interval.clone().count() {
        2 => {
            let range_start = range_interval.clone().next().unwrap().parse::<u32>().unwrap();
            let range_end = range_interval.clone().nth(1).unwrap().parse::<u32>().unwrap();
            (range_start, range_end)
        },
        _ => {
            return Ok(HttpResponse::BadRequest().body("Invalid range"));
        }
    };

    let mut glyphs = Vec::new();

    for glyph_name in split_into_glyphs {
        let path = format!("./output_pbfs/");

        if !Path::new(&path).exists() {
            return Ok(HttpResponse::NotFound().body("Glyph not found"));
        }

        let load_glyph = pbf_font_tools::load_glyphs(path.clone(),
             glyph_name.as_str(), range_start, range_end).await;

        if let Err(e) = &load_glyph {
            let formatted_err = format!("Error loading glyph: {:?}\n{:#?}",glyph_name, e);
            eprintln!("{}", formatted_err);
            return Ok(HttpResponse::InternalServerError().body(formatted_err));
        }

        let glyph = load_glyph.unwrap();

        glyphs.push(glyph);
    }

    let combined = pbf_font_tools::combine_glyphs(glyphs);

    match combined {
        Some(combined) => {

            return Ok(HttpResponse::Ok()
            .content_type("application/x-protobuf")
            .body(combined.write_to_bytes().unwrap())); 
        },
        None => {
            return Ok(HttpResponse::NoContent().body("No Glyphs found"));
        }
    }
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Benvenuti al server tipo di Catenary!")
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
            )
            .service(get_font)
            .service(index)
    })
    .bind("127.0.0.1:30412")?
    .run()
    .await
}