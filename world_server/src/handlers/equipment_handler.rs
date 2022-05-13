use crate::client_manager::ClientManager;
use crate::opcodes::Opcodes;
use crate::packet::*;
use crate::packet_handler::PacketToHandle;
use crate::prelude::*;
use crate::world::World;
use podio::{LittleEndian, ReadPodExt, WritePodExt};
use std::io::Write;

pub async fn handle_cmsg_item_query_single(client_manager: &ClientManager, packet: &PacketToHandle, world: &World) -> Result<()> {
    let client = client_manager.get_authenticated_client(packet.client_id).await?;

    let item_id = {
        let mut reader = std::io::Cursor::new(&packet.payload);
        reader.read_u32::<LittleEndian>()?
    };

    let item_template = world.get_realm_database().get_item_template(item_id).await?;

    let (header, mut writer) = create_packet(Opcodes::SMSG_ITEM_QUERY_SINGLE_RESPONSE, 20);
    writer.write_u32::<LittleEndian>(item_template.id)?;
    writer.write_u32::<LittleEndian>(item_template.class as u32)?;
    writer.write_u32::<LittleEndian>(item_template.subclass as u32)?;
    writer.write_i32::<LittleEndian>(-1)?;
    writer.write_all(item_template.name.as_bytes())?;
    writer.write_u8(0)?; //nullterminator
    writer.write_u8(0)?; //name2 nullterminator
    writer.write_u8(0)?; //name3
    writer.write_u8(0)?; //name4
    writer.write_u32::<LittleEndian>(item_template.displayid)?;
    writer.write_u32::<LittleEndian>(item_template.quality as u32)?;
    writer.write_u32::<LittleEndian>(item_template.flags)?;
    writer.write_u32::<LittleEndian>(item_template.flags2)?;
    writer.write_u32::<LittleEndian>(item_template.buy_price)?;
    writer.write_u32::<LittleEndian>(item_template.sell_price)?;
    writer.write_u32::<LittleEndian>(item_template.inventory_type as u32)?;
    writer.write_u32::<LittleEndian>(item_template.allowed_classes_mask.unwrap_or(u32::max_value()))?;
    writer.write_u32::<LittleEndian>(item_template.allowed_races_mask.unwrap_or(u32::max_value()))?;
    writer.write_u32::<LittleEndian>(item_template.item_level as u32)?;
    writer.write_u32::<LittleEndian>(item_template.required_level.unwrap_or_default() as u32)?;

    let required_skill = item_template.required_skill.unwrap_or_default();
    writer.write_u32::<LittleEndian>(required_skill.skill_id as u32)?;
    writer.write_u32::<LittleEndian>(required_skill.required_rank as u32)?;
    writer.write_u32::<LittleEndian>(item_template.required_spell_id.unwrap_or_default())?;
    writer.write_u32::<LittleEndian>(item_template.required_honor_rank.unwrap_or_default())?;
    writer.write_u32::<LittleEndian>(0)?; //Required cityrank. deprecated

    let required_faction = item_template.required_faction.unwrap_or_default();
    writer.write_u32::<LittleEndian>(required_faction.faction_id as u32)?;
    writer.write_u32::<LittleEndian>(required_faction.required_rank as u32)?;
    writer.write_i32::<LittleEndian>(item_template.max_count as i32)?;
    writer.write_i32::<LittleEndian>(item_template.stackable as i32)?;
    writer.write_u32::<LittleEndian>(item_template.container_slots as u32)?;
    writer.write_u32::<LittleEndian>(item_template.granted_stats.len() as u32)?;
    //for each granted stat...u32+u32
    writer.write_u32::<LittleEndian>(item_template.scaling_stat_distribution as u32)?;
    writer.write_u32::<LittleEndian>(item_template.scaling_stat_value)?;
    for i in 0..2 {
        let dmg = item_template.damage.get(i).cloned().unwrap_or_default();
        writer.write_f32::<LittleEndian>(dmg.min)?;
        writer.write_f32::<LittleEndian>(dmg.max)?;
        writer.write_u32::<LittleEndian>(dmg.damage_type as u32)?;
    }
    writer.write_u32::<LittleEndian>(item_template.granted_armor.unwrap_or_default() as u32)?;
    let resistances = item_template.granted_resistances.unwrap_or_default();
    writer.write_u32::<LittleEndian>(resistances.holy as u32)?;
    writer.write_u32::<LittleEndian>(resistances.fire as u32)?;
    writer.write_u32::<LittleEndian>(resistances.nature as u32)?;
    writer.write_u32::<LittleEndian>(resistances.frost as u32)?;
    writer.write_u32::<LittleEndian>(resistances.shadow as u32)?;
    writer.write_u32::<LittleEndian>(resistances.arcane as u32)?;
    writer.write_u32::<LittleEndian>(item_template.delay.unwrap_or_default() as u32)?;
    writer.write_u32::<LittleEndian>(item_template.ammo_type.unwrap_or_default() as u32)?;
    writer.write_f32::<LittleEndian>(item_template.ranged_mod_range)?;
    for i in 0..5 {
        let spell_proc = item_template.spell_procs.get(i).cloned().unwrap_or_default();
        writer.write_u32::<LittleEndian>(spell_proc.spell_id)?;
        writer.write_u32::<LittleEndian>(spell_proc.trigger_type as u32)?;
        writer.write_u32::<LittleEndian>(u32::max_value())?; //spell_proc.charges as u32)?;
        writer.write_u32::<LittleEndian>(spell_proc.cooldown)?;
        writer.write_u32::<LittleEndian>(spell_proc.category as u32)?;
        writer.write_u32::<LittleEndian>(spell_proc.category_cooldown)?;

        /*writer.write_u32::<LittleEndian>(0)?;
        writer.write_u32::<LittleEndian>(0)?;
        writer.write_u32::<LittleEndian>(0)?;
        writer.write_u32::<LittleEndian>(u32::MAX)?;
        writer.write_u32::<LittleEndian>(0)?;
        writer.write_u32::<LittleEndian>(u32::MAX)?;*/
    }
    writer.write_u32::<LittleEndian>(item_template.bonding as u32)?;
    writer.write_all(item_template.description.as_bytes())?;
    writer.write_u8(0)?; //description string terminator
    let page_text = item_template.readable_info.unwrap_or_default();
    writer.write_u32::<LittleEndian>(page_text.text_id)?;
    writer.write_u32::<LittleEndian>(page_text.language_id as u32)?;
    writer.write_u32::<LittleEndian>(page_text.page_material as u32)?;
    writer.write_u32::<LittleEndian>(item_template.start_quest_id.unwrap_or_default())?;
    writer.write_u32::<LittleEndian>(item_template.lock_id.unwrap_or_default())?;
    writer.write_i32::<LittleEndian>(item_template.material as i32)?;
    writer.write_u32::<LittleEndian>(item_template.sheath_style as u32)?;
    writer.write_u32::<LittleEndian>(item_template.random_property)?;
    writer.write_u32::<LittleEndian>(item_template.random_suffix)?;
    writer.write_u32::<LittleEndian>(item_template.block_value.unwrap_or_default())?;
    writer.write_u32::<LittleEndian>(item_template.item_set_id.unwrap_or_default())?;
    writer.write_u32::<LittleEndian>(item_template.max_durability as u32)?;
    writer.write_u32::<LittleEndian>(item_template.usable_area.unwrap_or_default() as u32)?;
    writer.write_u32::<LittleEndian>(item_template.usable_map.unwrap_or_default() as u32)?;
    writer.write_u32::<LittleEndian>(item_template.bag_family_mask.unwrap_or_default() as u32)?;
    writer.write_u32::<LittleEndian>(item_template.totem_category.unwrap_or_default() as u32)?;
    for i in 0..3 {
        let socket = item_template.sockets.get(i).cloned().unwrap_or_default();
        writer.write_u32::<LittleEndian>(socket.color as u32)?;
        writer.write_u32::<LittleEndian>(socket.content as u32)?;
    }
    writer.write_u32::<LittleEndian>(item_template.socket_bonus.unwrap_or_default())?;
    writer.write_u32::<LittleEndian>(item_template.gem_properties)?;
    writer.write_i32::<LittleEndian>(item_template.required_disenchant_skill.unwrap_or_default() as i32)?;
    writer.write_f32::<LittleEndian>(item_template.armor_damage_modifier)?;
    writer.write_u32::<LittleEndian>(item_template.duration)?;
    writer.write_u32::<LittleEndian>(item_template.item_limit_category as u32)?;
    writer.write_u32::<LittleEndian>(item_template.holiday_id)?;

    send_packet(&client, &header, &writer).await
}
