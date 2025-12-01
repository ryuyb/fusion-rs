use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RespWrapper<T> {
    pub code: i64,
    pub message: String,
    pub data: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoomInfoResp {
    pub room_id: i64,
    pub uid: i64,
    pub title: String,
    pub live_status: i32,
    pub live_time: String,
    pub online: u64,
    pub user_cover: String,
    pub description: String,
    pub area_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MasterInfoResp {
    pub info: MasterInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MasterInfo {
    pub uid: i64,
    pub uname: String,
    pub face: String,
}
