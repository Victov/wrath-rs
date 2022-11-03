use wow_world_messages::wrath::{CinematicSequenceId, Class, Race};

pub const fn get_opening_cinematic_for_race_class(race: &Race, class: &Class) -> Option<CinematicSequenceId> {
    match (*race, *class) {
        (_, Class::DeathKnight) => Some(CinematicSequenceId::DeathKnight),
        (Race::Human, _) => Some(CinematicSequenceId::Human),
        (Race::Orc, _) => Some(CinematicSequenceId::Orc),
        (Race::Dwarf, _) => Some(CinematicSequenceId::Dwarf),
        (Race::NightElf, _) => Some(CinematicSequenceId::NightElf),
        (Race::Undead, _) => Some(CinematicSequenceId::Undead),
        (Race::Tauren, _) => Some(CinematicSequenceId::Tauren),
        (Race::BloodElf, _) => Some(CinematicSequenceId::BloodElf),
        (Race::Draenei, _) => Some(CinematicSequenceId::Draenei),
        (Race::Gnome, _) => Some(CinematicSequenceId::Gnome),
        _ => None,
    }
}
