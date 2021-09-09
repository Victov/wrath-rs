pub struct DBRealm {
    pub id: u32,
    pub name: String,
    pub realm_type: u8,
    pub flags: u8,
    pub ip: String,
    pub population: f32,
    pub timezone: u8,
    pub online: u8,
}

pub struct DBAccount {
    pub id: u32,
    pub username: String,
    pub sha_pass_hash: String,
    pub sessionkey: String,
    pub v: String,
    pub s: String,
    pub token_key: String,
    pub banned: u8,
}

pub struct DBAccountData {
    pub account_id: u32,
    pub data_type: u32,
    pub time: u64,
    pub decompressed_size: u32,
    pub data: Option<Vec<u8>>,
}
