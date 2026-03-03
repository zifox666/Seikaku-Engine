use std::collections::HashMap;
use std::path::Path;

use rusqlite::Connection;
use serde::Deserialize;

/// Type data loaded from invTypes + invGroups
pub struct TypeData {
    pub group_id: i32,
    pub category_id: i32,
    pub capacity: Option<f64>,
    pub mass: Option<f64>,
    pub radius: Option<f64>,
    pub volume: Option<f64>,
    pub name: String,
}

/// Dogma attribute metadata from dgmAttributeTypes
pub struct DogmaAttributeData {
    pub default_value: f64,
    pub high_is_good: bool,
    pub stackable: bool,
    pub name: String,
}

/// Per-type dogma attribute value from dgmTypeAttributes
pub struct TypeDogmaAttributeData {
    pub attribute_id: i32,
    pub value: f64,
}

/// Per-type dogma effect from dgmTypeEffects
pub struct TypeDogmaEffectData {
    pub effect_id: i32,
    pub is_default: bool,
}

/// Combined per-type dogma data
pub struct TypeDogmaData {
    pub dogma_attributes: Vec<TypeDogmaAttributeData>,
    pub dogma_effects: Vec<TypeDogmaEffectData>,
}

/// Modifier info entry for a dogma effect
pub struct ModifierInfoData {
    pub domain: i32,
    pub func: i32,
    pub modified_attribute_id: Option<i32>,
    pub modifying_attribute_id: Option<i32>,
    pub operation: Option<i32>,
    pub group_id: Option<i32>,
    pub skill_type_id: Option<i32>,
}

/// Dogma effect data from dgmEffects
pub struct DogmaEffectData {
    pub discharge_attribute_id: Option<i32>,
    pub duration_attribute_id: Option<i32>,
    pub effect_category: i32,
    pub electronic_chance: bool,
    pub is_assistance: bool,
    pub is_offensive: bool,
    pub is_warp_safe: bool,
    pub propulsion_chance: bool,
    pub range_chance: bool,
    pub range_attribute_id: Option<i32>,
    pub falloff_attribute_id: Option<i32>,
    pub tracking_speed_attribute_id: Option<i32>,
    pub fitting_usage_chance_attribute_id: Option<i32>,
    pub resistance_attribute_id: Option<i32>,
    pub modifier_info: Vec<ModifierInfoData>,
}

/// All data loaded from the SDE SQLite database
pub struct Data {
    pub types: HashMap<i32, TypeData>,
    pub type_dogma: HashMap<i32, TypeDogmaData>,
    pub dogma_attributes: HashMap<i32, DogmaAttributeData>,
    pub dogma_effects: HashMap<i32, DogmaEffectData>,
}

// ---------- modifierInfo JSON/YAML parsing ----------

#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrInt {
    Int(i32),
    Str(String),
}

#[derive(Deserialize)]
struct ModifierInfoRaw {
    domain: StringOrInt,
    func: StringOrInt,
    #[serde(rename = "modifiedAttributeID")]
    modified_attribute_id: Option<i32>,
    #[serde(rename = "modifyingAttributeID")]
    modifying_attribute_id: Option<i32>,
    operation: Option<i32>,
    #[serde(rename = "groupID")]
    group_id: Option<i32>,
    #[serde(rename = "skillTypeID")]
    skill_type_id: Option<i32>,
}

fn domain_to_int(val: StringOrInt) -> i32 {
    match val {
        StringOrInt::Int(i) => i,
        StringOrInt::Str(s) => match s.as_str() {
            "itemID" => 0,
            "shipID" => 1,
            "charID" => 2,
            "otherID" => 3,
            "structureID" => 4,
            "target" => 5,
            "targetID" => 6,
            _ => 0,
        },
    }
}

fn func_to_int(val: StringOrInt) -> i32 {
    match val {
        StringOrInt::Int(i) => i,
        StringOrInt::Str(s) => match s.as_str() {
            "ItemModifier" => 0,
            "LocationGroupModifier" => 1,
            "LocationModifier" => 2,
            "LocationRequiredSkillModifier" => 3,
            "OwnerRequiredSkillModifier" => 4,
            "EffectStopper" => 5,
            _ => 0,
        },
    }
}

fn parse_modifier_info(text: Option<String>) -> Vec<ModifierInfoData> {
    let text = match text {
        Some(t) if !t.is_empty() => t,
        _ => return vec![],
    };

    // Try JSON first, then YAML (EVE SDE commonly stores this as YAML)
    let raw: Vec<ModifierInfoRaw> = serde_json::from_str(&text)
        .or_else(|_| serde_yaml::from_str(&text))
        .unwrap_or_default();

    raw.into_iter()
        .map(|r| ModifierInfoData {
            domain: domain_to_int(r.domain),
            func: func_to_int(r.func),
            modified_attribute_id: r.modified_attribute_id,
            modifying_attribute_id: r.modifying_attribute_id,
            operation: r.operation,
            group_id: r.group_id,
            skill_type_id: r.skill_type_id,
        })
        .collect()
}

// ---------- Data loading ----------

impl Data {
    pub fn new(sqlite_path: &Path) -> Data {
        let conn = Connection::open(sqlite_path)
            .unwrap_or_else(|e| panic!("Failed to open SQLite database at {:?}: {}", sqlite_path, e));

        let types = Self::load_types(&conn);
        let type_dogma = Self::load_type_dogma(&conn);
        let dogma_attributes = Self::load_dogma_attributes(&conn);
        let dogma_effects = Self::load_dogma_effects(&conn);

        Data {
            types,
            type_dogma,
            dogma_attributes,
            dogma_effects,
        }
    }

    fn load_types(conn: &Connection) -> HashMap<i32, TypeData> {
        let mut stmt = conn
            .prepare(
                "SELECT t.typeID, t.groupID, g.categoryID, t.typeName, \
                 t.mass, t.volume, t.capacity \
                 FROM invTypes t \
                 LEFT JOIN invGroups g ON t.groupID = g.groupID",
            )
            .unwrap();

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i32>(0)?,
                    TypeData {
                        group_id: row.get::<_, Option<i32>>(1)?.unwrap_or(0),
                        category_id: row.get::<_, Option<i32>>(2)?.unwrap_or(0),
                        name: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                        mass: row.get::<_, Option<f64>>(4)?,
                        volume: row.get::<_, Option<f64>>(5)?,
                        capacity: row.get::<_, Option<f64>>(6)?,
                        radius: None, // radius comes from dogma attributes (attributeID=162)
                    },
                ))
            })
            .unwrap();

        rows.filter_map(|r| r.ok()).collect()
    }

    fn load_type_dogma(conn: &Connection) -> HashMap<i32, TypeDogmaData> {
        let mut result: HashMap<i32, TypeDogmaData> = HashMap::new();

        // Load per-type dogma attributes
        let mut stmt = conn
            .prepare(
                "SELECT typeID, attributeID, \
                 COALESCE(valueFloat, CAST(valueInt AS REAL)) \
                 FROM dgmTypeAttributes",
            )
            .unwrap();

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i32>(0)?,
                    TypeDogmaAttributeData {
                        attribute_id: row.get::<_, i32>(1)?,
                        value: row.get::<_, Option<f64>>(2)?.unwrap_or(0.0),
                    },
                ))
            })
            .unwrap();

        for row in rows {
            if let Ok((type_id, attr)) = row {
                result
                    .entry(type_id)
                    .or_insert_with(|| TypeDogmaData {
                        dogma_attributes: vec![],
                        dogma_effects: vec![],
                    })
                    .dogma_attributes
                    .push(attr);
            }
        }

        // Load per-type dogma effects
        let mut stmt = conn
            .prepare("SELECT typeID, effectID, isDefault FROM dgmTypeEffects")
            .unwrap();

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i32>(0)?,
                    TypeDogmaEffectData {
                        effect_id: row.get::<_, i32>(1)?,
                        is_default: row.get::<_, Option<i32>>(2)?.unwrap_or(0) != 0,
                    },
                ))
            })
            .unwrap();

        for row in rows {
            if let Ok((type_id, effect)) = row {
                result
                    .entry(type_id)
                    .or_insert_with(|| TypeDogmaData {
                        dogma_attributes: vec![],
                        dogma_effects: vec![],
                    })
                    .dogma_effects
                    .push(effect);
            }
        }

        result
    }

    fn load_dogma_attributes(conn: &Connection) -> HashMap<i32, DogmaAttributeData> {
        let mut stmt = conn
            .prepare(
                "SELECT attributeID, attributeName, defaultValue, stackable, highIsGood \
                 FROM dgmAttributeTypes",
            )
            .unwrap();

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i32>(0)?,
                    DogmaAttributeData {
                        name: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                        default_value: row.get::<_, Option<f64>>(2)?.unwrap_or(0.0),
                        stackable: row.get::<_, Option<i32>>(3)?.unwrap_or(0) != 0,
                        high_is_good: row.get::<_, Option<i32>>(4)?.unwrap_or(0) != 0,
                    },
                ))
            })
            .unwrap();

        rows.filter_map(|r| r.ok()).collect()
    }

    fn load_dogma_effects(conn: &Connection) -> HashMap<i32, DogmaEffectData> {
        let mut stmt = conn
            .prepare(
                "SELECT effectID, effectCategory, isOffensive, isAssistance, \
                 durationAttributeID, trackingSpeedAttributeID, dischargeAttributeID, \
                 rangeAttributeID, falloffAttributeID, isWarpSafe, rangeChance, \
                 electronicChance, propulsionChance, fittingUsageChanceAttributeID, \
                 modifierInfo \
                 FROM dgmEffects",
            )
            .unwrap();

        let rows = stmt
            .query_map([], |row| {
                let modifier_info_text: Option<String> = row.get(14)?;

                Ok((
                    row.get::<_, i32>(0)?,
                    DogmaEffectData {
                        effect_category: row.get::<_, Option<i32>>(1)?.unwrap_or(0),
                        is_offensive: row.get::<_, Option<i32>>(2)?.unwrap_or(0) != 0,
                        is_assistance: row.get::<_, Option<i32>>(3)?.unwrap_or(0) != 0,
                        duration_attribute_id: row.get::<_, Option<i32>>(4)?,
                        tracking_speed_attribute_id: row.get::<_, Option<i32>>(5)?,
                        discharge_attribute_id: row.get::<_, Option<i32>>(6)?,
                        range_attribute_id: row.get::<_, Option<i32>>(7)?,
                        falloff_attribute_id: row.get::<_, Option<i32>>(8)?,
                        is_warp_safe: row.get::<_, Option<i32>>(9)?.unwrap_or(0) != 0,
                        range_chance: row.get::<_, Option<i32>>(10)?.unwrap_or(0) != 0,
                        electronic_chance: row.get::<_, Option<i32>>(11)?.unwrap_or(0) != 0,
                        propulsion_chance: row.get::<_, Option<i32>>(12)?.unwrap_or(0) != 0,
                        fitting_usage_chance_attribute_id: row.get::<_, Option<i32>>(13)?,
                        resistance_attribute_id: None, // Not present in SDE SQLite
                        modifier_info: parse_modifier_info(modifier_info_text),
                    },
                ))
            })
            .unwrap();

        rows.filter_map(|r| r.ok()).collect()
    }
}
