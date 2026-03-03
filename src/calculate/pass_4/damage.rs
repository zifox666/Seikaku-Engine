use crate::info::Info;

use super::super::item::SlotType;
use super::super::Ship;

/// Virtual attribute IDs for DPS stats (negative to distinguish from SDE attributes).
const ATTR_DPS_WITHOUT_RELOAD: i32 = -10;
const ATTR_DPS_WITH_RELOAD: i32 = -11;
const ATTR_ALPHA_STRIKE: i32 = -12;
const ATTR_DRONE_DPS: i32 = -20;

/// Standard SDE attribute IDs used in DPS calculation.
const ATTR_DAMAGE_MULTIPLIER: i32 = 64;
const ATTR_SPEED: i32 = 51; // cycleTime / rate of fire (ms)
const ATTR_EM_DAMAGE: i32 = 114;
const ATTR_EXPLOSIVE_DAMAGE: i32 = 116;
const ATTR_KINETIC_DAMAGE: i32 = 117;
const ATTR_THERMAL_DAMAGE: i32 = 118;
const ATTR_RELOAD_TIME: i32 = 1795; // reload time (ms)
const ATTR_CHARGE_RATE: i32 = 56; // charges consumed per cycle
const ATTR_CAPACITY: i32 = 38; // module capacity (max ammo)

/// Get the final value of an attribute on an item, fallback to base_value, then to default.
fn get_attr_value(item: &super::super::Item, attr_id: i32) -> Option<f64> {
    item.attributes
        .get(&attr_id)
        .map(|a| a.value.unwrap_or(a.base_value))
}

/// Sum the four damage types from an item (charge or drone).
fn total_damage(item: &super::super::Item) -> f64 {
    let em = get_attr_value(item, ATTR_EM_DAMAGE).unwrap_or(0.0);
    let exp = get_attr_value(item, ATTR_EXPLOSIVE_DAMAGE).unwrap_or(0.0);
    let kin = get_attr_value(item, ATTR_KINETIC_DAMAGE).unwrap_or(0.0);
    let therm = get_attr_value(item, ATTR_THERMAL_DAMAGE).unwrap_or(0.0);
    em + exp + kin + therm
}

pub fn attribute_damage_per_second(_info: &impl Info, ship: &mut Ship) {
    let mut total_dps_no_reload = 0.0;
    let mut total_dps_reload = 0.0;
    let mut total_alpha = 0.0;
    let mut total_drone_dps = 0.0;

    for item in &ship.items {
        let is_module = item.slot.is_module();
        let is_drone = item.slot.r#type == SlotType::DroneBay;

        if !item.state.is_active() && !is_drone_passive_dps(item) {
            continue;
        }

        if is_module {
            // Turret / Missile weapon module — damage comes from charge
            if let Some(charge) = &item.charge {
                let charge_damage = total_damage(charge);
                if charge_damage <= 0.0 {
                    continue;
                }

                let damage_multiplier =
                    get_attr_value(item, ATTR_DAMAGE_MULTIPLIER).unwrap_or(1.0);
                let cycle_time_ms = get_attr_value(item, ATTR_SPEED).unwrap_or(0.0);

                if cycle_time_ms <= 0.0 {
                    continue;
                }

                let damage_per_shot = charge_damage * damage_multiplier;
                let cycle_time_s = cycle_time_ms / 1000.0;
                let dps = damage_per_shot / cycle_time_s;

                total_alpha += damage_per_shot;
                total_dps_no_reload += dps;

                // DPS with reload
                let reload_time_ms = get_attr_value(item, ATTR_RELOAD_TIME).unwrap_or(0.0);
                if reload_time_ms > 0.0 {
                    // Determine charges per reload (module capacity / charge rate or 1)
                    let capacity = get_attr_value(item, ATTR_CAPACITY).unwrap_or(0.0);
                    let charge_rate = get_attr_value(item, ATTR_CHARGE_RATE).unwrap_or(1.0);
                    let charges_per_reload = if capacity > 0.0 && charge_rate > 0.0 {
                        (capacity / charge_rate).floor().max(1.0)
                    } else {
                        1.0
                    };
                    let total_cycle_s =
                        charges_per_reload * cycle_time_s + reload_time_ms / 1000.0;
                    let total_dmg = charges_per_reload * damage_per_shot;
                    total_dps_reload += total_dmg / total_cycle_s;
                } else {
                    // No reload — same as no-reload DPS
                    total_dps_reload += dps;
                }
            }
        } else if is_drone {
            // Drone — damage comes from the drone item itself
            if !item.state.is_active() {
                continue;
            }
            let drone_damage = total_damage(item);
            if drone_damage <= 0.0 {
                continue;
            }

            let damage_multiplier =
                get_attr_value(item, ATTR_DAMAGE_MULTIPLIER).unwrap_or(1.0);
            let cycle_time_ms = get_attr_value(item, ATTR_SPEED).unwrap_or(0.0);

            if cycle_time_ms <= 0.0 {
                continue;
            }

            let damage_per_shot = drone_damage * damage_multiplier;
            let cycle_time_s = cycle_time_ms / 1000.0;
            total_drone_dps += damage_per_shot / cycle_time_s;
        }
    }

    // Store computed DPS as virtual (negative ID) attributes on the hull
    ship.hull
        .add_attribute(ATTR_DPS_WITHOUT_RELOAD, 0.0, total_dps_no_reload);
    ship.hull
        .add_attribute(ATTR_DPS_WITH_RELOAD, 0.0, total_dps_reload);
    ship.hull.add_attribute(ATTR_ALPHA_STRIKE, 0.0, total_alpha);
    ship.hull
        .add_attribute(ATTR_DRONE_DPS, 0.0, total_drone_dps);
}

/// Check if a drone contributes "passive" DPS (drones set to Passive state don't attack).
fn is_drone_passive_dps(_item: &super::super::Item) -> bool {
    false
}
