use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct BetardResponse {
    pub room: BetardRoom,
    pub column: BetardColumn,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BetardRoom {
    pub nickname: String,
    pub owner_avatar: String,
    pub status: String,
    pub show_status: i32,
    pub show_details: String,
    pub room_name: String,
    pub room_pic: String,
    #[serde(rename = "coverSrc")]
    pub cover_src: String,
    pub show_time: i64,
    pub avatar: BetardRoomAvatar,
    pub cate_name: Option<String>,
    pub second_lvl_name: String,
    pub room_biz_all: BetardRoomBizAll,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BetardRoomAvatar {
    pub big: String,
    pub middle: String,
    pub small: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BetardRoomBizAll {
    pub hot: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BetardColumn {
    pub cate_id: String,
    pub cate_name: String,
}
