#[derive(Copy, Clone)]
#[allow(dead_code)]
pub enum RelationType
{
    None = 0x00,
    Friend = 0x01,
    Ignore = 0x02,
    Muted = 0x04,
    RecruitAFriend = 0x08,
}
