use diesel::RunQueryDsl;
use regex::Regex;

use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError, ValidationErrors};

use once_cell::sync::Lazy;

use super::super::view::IdPair;

use super::super::schema;

use super::db_util;
use super::db_util::is_exist_id;

// 友だち追加の流れ
// API -> (id, id): (String, String)

pub fn add_friend(id_pair: IdPair) -> bool {
    // -> (status_code: int, message: String)
    let my_id = id_pair.my_id;
    let friend_id = id_pair.target_id;

    if my_id == friend_id {
        return false;
    }

    // IDがレコードに存在してるかチェック
    if !is_exist_id(&my_id) || !is_exist_id(&friend_id) {
        return false;
    }

    let ids = AddFriend {
        acctive: &my_id,
        pussive: &friend_id,
    };

    let conn = db_util::establish_connection();
    diesel::insert_into(friends::table)
        .values(&ids)
        .execute(&conn)
        .expect("挿入失敗");

    return true;
    // DBにインサート
    // bool か Result を返す
}

pub struct SearchUser {
    user_id: String,
    user_name: String,
    icon_path: String,
    applied: bool,
    requested: bool,
}

pub enum SomeError {
    ValidationError,
    NotExistError,
    SameIdError,
}
pub fn search_user(id_pair: IdPair) -> Result<SearchUser, SomeError> {
    // pub fn search_user(id_pair: IdPair) -> (u16, String) {
    let my_id = &id_pair.my_id;
    let friend_id = &id_pair.target_id;

    // バリデーション
    if let Err(r) = &id_pair.validate() {
        // return (422, r.to_string());
        return Err(SomeError::ValidationError);
    }

    // レコード存在確認
    if !is_exist_id(&my_id) || !is_exist_id(&friend_id) {
        // return (404, "Err, id not found".to_string());
        return Err(SomeError::NotExistError);
    }

    // if invalid validation {
    //      どこでキャッチすればいいのかわかってない。axumの仕様調べる。
    // }

    // 自身を登録
    if my_id == friend_id {
        // return (471, "Err, send same id".to_string());
        return Err(SomeError::SameIdError);
    }

    return Ok(SearchUser {
        user_id: todo!(),
        user_name: todo!(),
        icon_path: todo!(),
        applied: todo!(),
        requested: todo!(),
    });
}

// fn get_friend() -> Option {}

// 正規表現をグローバルに宣言
static USER_ID: Lazy<regex::Regex> = Lazy::new(|| Regex::new(r"[0-9]{6}$").unwrap());

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct UserId {
    #[validate(regex = "USER_ID")]
    pub id: String,
}

#[derive(Debug, Validate, Deserialize, Queryable)]
pub struct User {
    pub id: i32,
    pub user_id: String,
    pub user_name: String,
    pub status: i32,
    pub beacon: Option<String>,
    pub icon_path: String,
    pub hashed_password: String,
}

// struct UserId {}
// type UserId = String;
// impl UserId {}

#[derive(Queryable)]
struct Friend {
    pub id: i32,
    pub acctive: String,
    pub passive: String,
    pub block_flag: bool,
}

use schema::friends;
#[derive(Insertable)]
#[table_name = "friends"]
pub struct AddFriend<'a> {
    pub acctive: &'a str,
    pub pussive: &'a str,
}

#[cfg(test)]
mod tests {
    use crate::{model::friends::add_friend, view::IdPair};

    #[test]
    fn t_add_friend() {
        // insert 000000
        // insert 000001

        // 上の状態までDBを復帰させる必要あり

        // 正常
        assert_eq!(
            add_friend(IdPair {
                my_id: "000000".to_string(),
                target_id: "000001".to_string()
            }),
            true
        );
        // 同じIDが挿入されるのはおかしい
        assert_eq!(
            add_friend(IdPair {
                my_id: "000000".to_string(),
                target_id: "000000".to_string()
            }),
            false
        );

        // 存在しないIDに友だち申請するのはおかしい
        assert_eq!(
            add_friend(IdPair {
                my_id: "000000".to_string(),
                target_id: "000002".to_string()
            }),
            false
        );

        // 不正なID
        assert_eq!(
            add_friend(IdPair {
                my_id: "abcdef".to_string(),
                target_id: "000000".to_string()
            }),
            false
        );

        // 不正なID
        assert_eq!(
            add_friend(IdPair {
                my_id: "12345".to_string(),
                target_id: "000000".to_string()
            }),
            false
        );
    }
}
