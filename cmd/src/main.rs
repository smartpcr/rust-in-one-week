mod my;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use math::{numbers::GcdParameters, operations};
use std::env;
use std::str::FromStr;

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

    match numbers.as_slice() {
        [] => {
            println!("No command-line numbers provided; submit values via the /gcd form instead.");
        }
        [single] => {
            println!(
                "Only one number ({single}) provided; add another to compute a GCD on the CLI."
            );
        }
        _ => {
            let mut result = numbers[0];
            for &value in &numbers[1..] {
                result = operations::gcd(result, value);
            }

            println!(
                "The greatest common divisor of the numbers of {:?} is {}",
                numbers, result
            );
        }
    }
    //#endregion

    //#region: web
    let server = HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(get_index))
            .route("/gcd", web::post().to(post_gcd))
    })
    .bind("127.0.0.1:3000")?;
    println!("Serving on http://localhost:3000...");
    server.run().await
    //#endregion
}

async fn get_index() -> impl Responder {
    HttpResponse::Ok().body(
        r#"
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
  "#,
    )
}

async fn post_gcd(form: web::Form<GcdParameters>) -> impl Responder {
    if (form.n == 0) || (form.m == 0) {
        return HttpResponse::BadRequest()
            .content_type("text/html")
            .body("Error: Computing the GCD with zero is boring.");
    }

    let response = format!(
        "The greatest common divisor of the numbers {} and {} is <b>{}</b>",
        form.n,
        form.m,
        operations::gcd(form.n, form.m)
    );

    HttpResponse::Ok().content_type("text/html").body(response)
}
