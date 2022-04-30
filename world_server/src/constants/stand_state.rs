#[derive(PartialEq, Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum UnitStandState {
    Stand = 0,
    Sit = 1,
    SitChair = 2,
    Sleep = 3,
    SitLowChair = 4,
    SitMediumChair = 5,
    SitHighChair = 6,
    Dead = 7,
    Kneel = 8,
    Custom = 9, // Depends on model animation. Submerge, freeze, hide, hibernate, rest
}
