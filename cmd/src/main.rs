use std::env;
use std::str::FromStr;
use actix_web::{web, App, HttpResponse, HttpServer};

mod my;
mod lib;

fn function() {
  println!("called `function()`");
}

fn main() {

  //#region: my
  my::function();

  function();

  my::indirect_access();

  my::nested::function();
  //#endregion

  //#region: web
  let server = HttpServer::new(|| {
    App::new()
      .route("/", web::get().to(get_index))
      .route("/gcd", web::post().to(post_gcd))
  });

  println!("Serving on http://localhost:3000...");
  server
    .bind("127.0.0.1:3000").expect("Can not bind to port 3000")
    .run().expect("Failed to start server");
  //#endregion

  //#region: lib
  let mut numbers = Vec::new();
  for arg in env::args().skip(1) {
    numbers.push(u64::from_str(&arg).expect("error parsing argument"));
  }

  if numbers.len() == 0 {
    panic!("Error: No numbers provided")
  }

  let mut result = numbers[0];
  for i in 1..numbers.len() {
    result = lib::math::gcd(result, numbers[i]);
  }

  println!("The greatest common divisor of the numbers of {:?} is {}", numbers, result);
  //#endregion
}

fn get_index() -> HttpResponse {
  HttpResponse::Ok()
    .body(r#"
      <html>
        <head><title>GCD Calculator</title></head>
        <body>
          <form action="/gcd" method="post">
            <input type="text" name="n" />
            <input type="text" name="m" />
            <button type="submit">Compute GCD</button>
          </form>
        </body>
      </html>
    "#)
}

fn post_gcd(form: web::Form<lib::math::GcdParameters>) -> HttpResponse {
  if (form.n == 0) || (form.m == 0) {
    return HttpResponse::BadRequest()
      .content_type("text/html")
      .body("Error: Computing the GCD with zero is boring.");
  }

  let response =
    format!("The greatest common divisor of the numbers {} and {} is <b>{}</b>",
    form.n, form.m, lib::math::gcd(form.n, form.m));

  HttpResponse::Ok()
    .content_type("text/html")
    .body(response)
}
