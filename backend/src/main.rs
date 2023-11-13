use actix_web::middleware::DefaultHeaders;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use backend::models::*;
use backend::establish_connection;
use diesel::prelude::*;
use std::env;
use std::{thread, time};

mod walgen;
mod utils;
mod web_methods;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    //Esperamos 0.5s para asegurar que la base de datos estará lista para aceptar conexiones
    //En caso de que no fuese suficiente porque la base de datos ha crecido mucho, habría que aumentar el tiempo que se espera y reconstruir la imagen
    let ten_millis = time::Duration::from_millis(500);
    let now = time::Instant::now();
    thread::sleep(ten_millis);
    assert!(now.elapsed() >= ten_millis);

    /*
    let (secret_key, public_key) = walgen::generate_keypair();
    let public_address = walgen::public_key_address(&public_key);
    println!("Secret key: {}", secret_key.display_secret());
    println!("Public key: {}", public_key.to_string());
    println!("Public address: {}", &public_address);
    */
    
    //let wallet_instance = walgen::Wallet::new(&secret_key, &public_key);
    //wallet_instance.save_to_file("crypto_wallet.json");
    /*let wallet_instance = walgen::Wallet::from_file("crypto_wallet.json").unwrap();
    println!("Public key: {}", wallet_instance.public_address.to_string());

    let endpoint = env::var("INFURA_RINKEBY_WS").unwrap();
    let web3_conn = walgen::establish_web3_connection(&endpoint).await;

    let block_number = web3_conn.eth().block_number().await.unwrap();
    println!("block number: {}", &block_number);

    let balance = wallet_instance.get_eth_balance(&web3_conn).await.unwrap();
    println!("Balance: {balance}");
     */
    

    HttpServer::new(|| {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .service(web_methods::prueba)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}
