// // # [macro_use]
// // extern  crate diesel;

// use diesel::mysql::MysqlConnection;
// use diesel::prelude::*;
// use dotenv::dotenv;
// use std::env;

// pub fn establish_connection() -> MysqlConnection {
//     dotenv().ok();

//     let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     MysqlConnection::establish(&database_url)
//         .expect(&format!("Erroe connetincg to {}", database_url))
// }
