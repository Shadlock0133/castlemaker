use serde::{Serialize, Deserialize};
use rand::thread_rng;
use std::{
    collections::HashMap,
    fmt::{self, Display},
};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DamageRoll(u8, u8);

impl Display for DamageRoll {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}k{}", self.0, self.1)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Size {
    Fine,
    Diminutive,
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
    Gargantuan,
    Colossal,
}

impl Size {
    pub fn modifier(&self) -> i8 {
        match self {
            Size::Fine => 8,
            Size::Diminutive => 4,
            Size::Tiny => 2,
            Size::Small => 1,
            Size::Medium => 0,
            Size::Large => -1,
            Size::Huge => -2,
            Size::Gargantuan => -4,
            Size::Colossal => -8,
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Char {
    //class: Class,
    //race: Race,
    //abilities: ,
    attributes: Attributes,
    level: u8,
    experience_actual: u16,
    attack: u8,
    damage_roll: DamageRoll,
    size: Size,
    natural_armor: u8,
    base_saving_throws: SavingThrows,            
}

impl Char {
    pub fn new() -> Self {
        Self {
            attributes: Attributes {
                strength: 16,
                dexterity: 12,
                constitution: 18,
                intelligence: 14,
                wisdom: 10,
                charisma: 21,
            },
            level: 4,
            experience_actual: 1000,
            attack: 4,
            damage_roll: (2, 6),
            size: Size::Medium,
            natural_armor: 2,
            base_saving_throws: SavingThrows {
                fortitude: 4,
                reflex: 1,
                will: 4,
            }
        }
    }

    fn saving_throws_fortitude(&self, other: i8) -> u8 {
        (
            self.base_saving_throws.fortitude as i8 +
            ((self.attributes.constitution / 2).saturating_sub(5)) as i8 +
            thread_rng().gen_range(0, 20) + 1 +
            other
        ).max(0)
    }

    fn saving_throws_reflex(&self, other: i8) -> u8 {
        (
            self.base_saving_throws.reflex as i8 +
            ((self.attributes.dexterity / 2).saturating_sub(5)) as i8 +
            thread_rng().gen_range(0, 20) + 1 +
            other
        ).max(0)
    }

    fn saving_throws_will(&self, other: i8) -> u8 {
        (
            self.base_saving_throws.will as i8 +
            ((self.attributes.wisdom / 2).saturating_sub(5)) as i8 +
            thread_rng().gen_range(0, 20) + 1 +
            other
        ).max(0)
    }

    fn damage_bonus(&self) -> i8 {
        ((self.attributes.strength / 2).saturating_sub(5)) as i8
    }

    fn armor_class(&self) -> u8 {
        (10 +
        0/*armor*/ +
        ((self.attributes.dexterity / 2).saturating_sub(5)) as i8 +
        self.size.modifier() +
        self.natural_armor as i8).max(1) as u8
    }

    fn flat_footed(&self) -> u8 {
        (10 +
        0/*armor*/ +
        self.size.modifier() +
        self.natural_armor as i8).max(1) as u8
    }

    fn touch(&self) -> u8 {
        (10 +
        ((self.attributes.dexterity / 2).saturating_sub(5)) as i8 +
        self.size.modifier()).max(1) as u8
    }

    pub fn damage(&self) -> u8 {
        let mut final_damage: u8 = 0;
        for _ in 0..self.damage_roll.0 {
            final_damage += thread_rng().gen_range(0, self.damage_roll.1) + 1;
        }
        (final_damage as i8 + self.damage_bonus()).max(1) as u8
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Attributes {
    strength: u8,
    dexterity: u8,
    constitution: u8,
    intelligence: u8,
    wisdom: u8,
    charisma: u8,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct SavingThrows {
    fortitude: u8,
    reflex: u8,
    will: u8,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Class {
    name: String,
    hit_die: u8,
    skills: Vec<Skill>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Skill {
    name: String,
    properties: HashSet<Attribute>,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ability {
    name: String,
    key_attribute: Attribute,
    untrained: bool,
    armor_check_penalty: bool,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Attribute {
    #[serde(rename = "Str")] Strength,
    #[serde(rename = "Dex")] Dexterity,
    #[serde(rename = "Con")] Constitution,
    #[serde(rename = "Int")] Intelligence,
    #[serde(rename = "Wis")] Wisdom,
    #[serde(rename = "Cha")] Charisma,
}
