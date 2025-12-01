mod my;
mod math;

use std::env;
use std::str::FromStr;
use actix_web::{web, App, HttpResponse, HttpServer};

fn function() {
  println!("called `function()`");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

  //#region: my
  my::function();

  function();

  my::indirect_access();

  my::nested::function();
  //#endregion

  //#region: lib
  let mut numbers = Vec::new();
  for arg in env::args().skip(1) {
    numbers.push(u64::from_str(&arg).expect("error parsing argument"));
  }

  if numbers.is_empty() {
    panic!("Error: No numbers provided")
  }

  let mut result = numbers[0];
  for i in 1..numbers.len() {
    result = math::operations::gcd(result, numbers[i]);
  }

  println!("The greatest common divisor of the numbers of {:?} is {}", numbers, result);
  //#endregion

  //#region: web
  let server = HttpServer::new(|| {
    App::new()
      .route("/", web::get().to(get_index))
      .route("/gcd", web::post().to(post_gcd))
  });

  println!("Serving on http://localhost:3000...");
  server
    .bind("127.0.0.1:3000")?
    .run()
    .await
  //#endregion
  
}

async fn get_index() -> HttpResponse {
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

async fn post_gcd(form: web::Form<math::GcdParameters>) -> HttpResponse {
  if (form.n == 0) || (form.m == 0) {
    return HttpResponse::BadRequest()
      .content_type("text/html")
      .body("Error: Computing the GCD with zero is boring.");
  }

  let response =
    format!("The greatest common divisor of the numbers {} and {} is <b>{}</b>",
    form.n, form.m, math::operations::gcd(form.n, form.m));

  HttpResponse::Ok()
    .content_type("text/html")
    .body(response)
}
