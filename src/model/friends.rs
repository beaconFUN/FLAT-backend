use crate::model::db_util::*;
use crate::model::types::SomeError;
use crate::repository::{AddFriend, IdNamePath};
// use crate::schema::friends;
use crate::view::{FriendList, IdAndName, IdPair, SearchUser};
use axum::response::IntoResponse;
// use diesel::RunQueryDsl;
use hyper::{Body, Response, StatusCode};
// use validator::Validate;

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
    if !is_exist_id(my_id) || !is_exist_id(friend_id) {
        return false;
    }

    if let Ok(_) = insert_friend(AddFriend {
        active: my_id,
        passive: friend_id,
    }) {
        return true;
    };
    return false;

    // DBにインサート
    // bool か Result を返す
}

pub fn reject_friend(id_pair: IdPair) -> bool {
    let my_id = id_pair.my_id;
    let friend_id = id_pair.target_id;

    if my_id == friend_id {
        return false;
    }

    // IDがレコードに存在してるかチェック
    if !is_exist_id(my_id) || !is_exist_id(friend_id) {
        return false;
    };

    if let Ok(_) = delete_friend(AddFriend {
        active: my_id,
        passive: friend_id,
    }) {
        return true;
    }
    return false;
}

impl IntoResponse for SomeError {
    type Body = Body;
    type BodyError = <Self::Body as axum::body::HttpBody>::Error;
    fn into_response(self) -> Response<Self::Body> {
        let body = match self {
            SomeError::ValidationError => Body::from("something went wrong"),
            SomeError::NotExistError => Body::from("something else went wrong"),
            SomeError::SameIdError => Body::from("something else went wrong"),
        };

        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(body)
            .unwrap()
    }
}

pub fn search_user(id_and_name: IdAndName) -> Result<Vec<SearchUser>, SomeError> {
    // Input
    // id: 自分のID
    // target_name: 検索したい相手の名前の部分文字列

    // Output
    // Ok() => Vec<SearchUser>
    //
    // where SearchUser =   struct SearchUser {
    //                              pub user_id: i32,
    //                              pub user_name: String,
    //                              pub icon_path: String,
    //                              pub applied: bool,
    //                              pub requested: bool,
    //                      }
    //
    // Err() => SomeError

    // バリデーション
    // if let Err(_r) = &id_and_name.validate() {
    //     // return (422, r.to_string());
    //     return Err(SomeError::ValidationError);
    // }

    let my_id = id_and_name.my_id;
    let target_name = id_and_name.target_name;

    // レコード存在確認
    if !is_exist_id(my_id) {
        // return (404, "Err, id not found".to_string());
        return Err(SomeError::NotExistError);
    }

    // if invalid validation {
    //      どこでキャッチすればいいのかわかってない。axumの仕様調べる。
    // }

    // // 自身を登録
    // if my_id == friend_id {
    //     // return (471, "Err, send same id".to_string());
    //     return Err(SomeError::SameIdError);
    // }

    // db_util::get_user_id_name_path(id) -> (id, name, path)
    // db_util::get_friends_relation(id1, id2) -> (bool, bool)

    let id_name_path: Vec<IdNamePath> = get_user_id_name_path(target_name);
    let applied_and_requested = &id_name_path
        .iter()
        .map(|x| get_friends_relation(my_id, x.id))
        .collect::<Vec<_>>();
    let searched_users = id_name_path
        .into_iter()
        .zip(applied_and_requested)
        .filter(|(_, &y)| !(y.0 & y.1)) // 相互の友だちを落とす
        .filter(|(x, _)| x.id != my_id) // 自分を落とす
        .map(|(x, y)| SearchUser {
            id: x.id,
            name: x.name,
            icon_path: x.icon_path,
            applied: y.0,
            requested: y.1,
        })
        .collect::<Vec<SearchUser>>();

    return Ok(searched_users);
    // let (ap, req) = get_friends_relation(my_id, target_name);
    // return Ok(vec![SearchUser {
    //     user_id: id,
    //     user_name: name,
    //     icon_path: path,
    //     applied: ap,
    //     requested: req,
    // }]);
}

pub fn get_friend_list(my_id: i32) -> FriendList {
    // requested: id  自分 -> 誰か という関係があるUserIdを持ってくる
    // applied: id 誰か -> 自分 という関係があるUserIdを持ってくる
    // applied の各要素が、reqested に含まれるかどうかで
    // mutual と one_side に振り分ける
    // idを基にUserViewをとってくる -> JOINしたほうが良さそう

    // 片思われ | 両思い
    let requested = get_requested_record(my_id);
    // 片思い | 両思い
    let applied = get_applied_record(my_id);
    let (mutual, one_side): (Vec<_>, Vec<_>) =
        requested.into_iter().partition(|r| applied.contains(&r.id));
    let result = FriendList { one_side, mutual };
    // println!("{:#?}", result);
    return result;
    // todo!()
}

#[cfg(test)]
mod tests {
    use crate::{
        model::{db_util::insert_friend, users::create_user},
        repository::{AddFriend, NameAndPassword},
        view::FriendList,
    };

    use super::get_friend_list;

    // #[tokio::test]
    #[test]
    fn test_requested_and_applied() {
        let uv1 = create_user(NameAndPassword {
            name: &"test1".to_string(),
            hashed_password: &"".to_string(),
        });
        let uv2 = create_user(NameAndPassword {
            name: &"test2".to_string(),
            hashed_password: &"".to_string(),
        });
        let uv3 = create_user(NameAndPassword {
            name: &"test3".to_string(),
            hashed_password: &"".to_string(),
        });
        println!("uv1.id {:#?}", uv1.id);
        println!("uv2.id {:#?}", uv2.id);
        println!("uv3.id {:#?}", uv3.id);
        // uv1 -> uv2
        // uv1 は uv2 に片思いしている
        let _ = insert_friend(AddFriend {
            active: uv1.id,
            passive: uv2.id,
        });
        // uv2 -> uv1
        // uv1 は uv2 に片思われされている。= mutual
        let _ = insert_friend(AddFriend {
            active: uv2.id,
            passive: uv1.id,
        });
        // uv3 -> uv1
        // uv1 は uv3 に片思われされている。= one_side
        let _ = insert_friend(AddFriend {
            active: uv3.id,
            passive: uv1.id,
        });
        let result = get_friend_list(uv1.id);
        assert_eq!(
            result,
            FriendList {
                one_side: vec![uv3],
                mutual: vec![uv2]
            }
        );
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{model::friends::add_friend, view::IdPair};

//     #[test]
//     fn t_add_friend() {
//         // insert 000000
//         // insert 000001

//         // 上の状態までDBを復帰させる必要あり

//         // 正常
//         assert_eq!(
//             add_friend(IdPair {
//                 my_id: "000000".to_string(),
//                 target_id: "000001".to_string()
//             }),
//             true
//         );
//         // 同じIDが挿入されるのはおかしい
//         assert_eq!(
//             add_friend(IdPair {
//                 my_id: "000000".to_string(),
//                 target_id: "000000".to_string()
//             }),
//             false
//         );

//         // 存在しないIDに友だち申請するのはおかしい
//         assert_eq!(
//             add_friend(IdPair {
//                 my_id: "000000".to_string(),
//                 target_id: "000002".to_string()
//             }),
//             false
//         );

//         // 不正なID
//         assert_eq!(
//             add_friend(IdPair {
//                 my_id: "abcdef".to_string(),
//                 target_id: "000000".to_string()
//             }),
//             false
//         );

//         // 不正なID
//         assert_eq!(
//             add_friend(IdPair {
//                 my_id: "12345".to_string(),
//                 target_id: "000000".to_string()
//             }),
//             false
//         );
//     }
// }
