use std::env;
use std::str::FromStr;

use actix_web::{post, web, HttpResponse, Responder};
use backend::models::*;
use backend::establish_connection;
use backend::insert_into_vault;
use backend::retrieve_from_vault;
use backend::AppState;
use diesel::insert_into;
use diesel::prelude::*;
use serde_derive::Serialize;
use serde::Deserialize;
use web3::types::Address;
use secp256k1::SecretKey;

use crate::utils;
use crate::walgen;
use crate::walgen::Wallet;
use crate::walgen::establish_web3_connection;

//Request

#[derive(Deserialize)]
struct Email {
    email: String,
}
#[derive(Deserialize)]
struct NewUser {
    name: String,
    email: String,
    password: String,
}
#[derive(Deserialize)]
struct Transaction {
    email: String,
    to: String,
    amount: f64,
}

//Response

#[derive(Serialize)]
struct Portfolio {
    balance: f64,
    address: String,
}
#[derive(Serialize)]
struct EmailExists {
    email_exists: bool,
}
#[derive(Serialize)]
struct PasswordMatches {
    password_matches: bool,
}


#[post("/checkEmailExists")]
pub async fn check_email_exists(json: web::Json<Email>) -> web::Json<EmailExists> {
    let connection = &mut establish_connection();

    use backend::schema::users::dsl::*;
    let results = users
        .filter(email.eq(&json.email))
        .select(User::as_select())
        .load(connection)
        .expect("Error loading users");

    if results.len() == 0 {
        web::Json(EmailExists { email_exists: false })
    } else {
        web::Json(EmailExists { email_exists: true })
    }
}


#[post("/createNewUser")]
pub async fn create_new_user(json: web::Json<NewUser>, data: web::Data<AppState>) -> impl Responder {
    let connection = &mut establish_connection();
    let new_name = &json.name;
    let new_email = &json.email;
    let new_password = &json.password;
    let root_key = &data.root_key;

    use backend::schema::users::dsl::*;

    // Hash the password to add security
    let (new_hashed_password, new_salt) = utils::hash_password(new_password);
    let (new_secret_key, new_public_key) = walgen::generate_keypair();

    let new_public_address = walgen::public_key_address(&new_public_key);
    let new_public_address = format!("{:?}", new_public_address);

    let private_address_str = new_secret_key.display_secret().to_string();
    println!("Private key: {}", private_address_str);
    insert_into_vault(&root_key, &new_email, &private_address_str).await;

    let _ = insert_into(users)
        .values((
            name.eq(new_name),
            email.eq(new_email),
            hashed_password.eq(new_hashed_password),
            salt.eq(new_salt),
            public_key.eq(new_public_key.to_string()),
            address.eq(new_public_address),
        ))
        .execute(connection);

    HttpResponse::Ok().body("Generated new user")
}

#[post("/checkPassword")]
pub async fn check_password(json: web::Json<NewUser>) -> web::Json<PasswordMatches> {
    println!("Checking if password matchers");
    let connection = &mut establish_connection();
    let new_email = &json.email;
    let new_password = &json.password;

    use backend::schema::users::dsl::*;

    let results = users
        .filter(email.eq(&new_email))
        .select(User::as_select())
        .load(connection)
        .expect("Error loading users");

    if results.len() == 0 {
        println!("Couldnt find user");
        web::Json(PasswordMatches { password_matches: false })
    } else {
        let user = &results[0];
        let password_matches = utils::check_hash(&new_password, &user.salt, &user.hashed_password);
        if password_matches {
            println!("Password matches");
            web::Json(PasswordMatches { password_matches: true })
        } else {
            web::Json(PasswordMatches { password_matches: false })
        }
    }
}

#[post("/getBalance")]
pub async fn get_balance(json: web::Json<Email>, data: web::Data<AppState>) -> web::Json<Portfolio> {
    let connection = &mut establish_connection();
    let user_email = &json.email;
    let mut user_balance: f64 = 0.0;
    let mut user_address = "Wallet address";

    let root_key = &data.root_key;
    let user_secret_key = retrieve_from_vault(root_key, &user_email).await;

    use backend::schema::users::dsl::*;

    let results = users
        .filter(email.eq(&user_email))
        .select(User::as_select())
        .load(connection)
        .expect("Error loading users, db problem");

    if results.len() == 0 {
        println!("Couldnt find user");
    } else {
        let user = &results[0];
        user_address = &user.address;
        let user_public_key = &user.public_key;
    
        let wallet_instance: Wallet = Wallet::from_params(
            &user_secret_key,
            &user_public_key,
            &user_address,
        ).unwrap();
        
        dotenv::dotenv().ok();
        let endpoint = env::var("INFURA_RINKEBY_WS").unwrap();
        let web3_conn = establish_web3_connection(&endpoint).await;
        let eth_balance_result = wallet_instance.get_eth_balance(&web3_conn).await;
        match eth_balance_result {
            Ok(eth_balance) => {
                user_balance = eth_balance;
            },
            Err(e) => {
                println!("Error getting balance: {}", e);
            }
        }
    }
    
    web::Json(Portfolio { balance: user_balance
    , address: user_address.to_string() })
}

#[post("/sendTransaction")]
pub async fn sign_and_send(json: web::Json<Transaction>, data: web::Data<AppState>) -> impl Responder {

    let user_email = &json.email;
    let to_address = &json.to;
    let amount = &json.amount;
    let root_key = &data.root_key;
    
    let user_secret_key = retrieve_from_vault(root_key, &user_email).await;
    println!("Secret key from vault: {}", user_secret_key);
    let user_secret_key = SecretKey::from_str(&user_secret_key).unwrap();
    
    let endpoint = env::var("INFURA_RINKEBY_WS").unwrap();
    let web3_conn = establish_web3_connection(&endpoint).await;
    let params = walgen::create_eth_transaction(
        Address::from_str(&to_address).unwrap(),
        *amount,
    );
    let signed_tx = walgen::sign_and_send(&web3_conn, params, &user_secret_key).await;
    match signed_tx {
        Ok(tx_hash) => {
            println!("Transaction sent: {:?}", tx_hash);
            return HttpResponse::Ok().body(format!("{:?}", tx_hash));
        },
        Err(e) => {
            println!("Error sending transaction: {}", e);
            return HttpResponse::Ok().body("Could not send transaction");
        }
    }
}