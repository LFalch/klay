use std::io::{Write, Read};
use std::str::FromStr;
use std::fmt::Debug;

use serde_xml_rs::{deserialize, Error};
use serde::{Deserialize, Deserializer};
use xml::EmitterConfig;
use xml::writer::Result as WResult;
use xml::writer::events::XmlEvent;
use xml::common::XmlVersion;

fn des<'de, T: FromStr, D: Deserializer<'de>>(d: D) -> Result<T, D::Error>
where <T as FromStr>::Err: Debug {
    String::deserialize(d).map(|s| s.parse().unwrap())
}

#[derive(Debug, Deserialize)]
pub struct KeyLayout {
    #[serde(deserialize_with = "des")]
    pub group: u16,
    #[serde(deserialize_with = "des")]
    pub id: i32,
    pub name: String,
    pub layouts: Layouts,
    #[serde(rename = "modifierMap")]
    pub modifier_map: ModifierMap,
    #[serde(rename = "keyMapSet")]
    pub key_map_set: KeyMapSet,
    pub actions: Actions,
    pub terminators: Terminators,
}

#[derive(Debug, Deserialize)]
pub struct Layouts {
    pub layout: Vec<Layout>,
}
#[derive(Debug, Deserialize)]
pub struct Layout {
    #[serde(deserialize_with = "des")]
    pub first: u16,
    #[serde(deserialize_with = "des")]
    pub last: u16,
    #[serde(rename = "mapSet")]
    pub map_set: String,
    pub modifiers: String,
}
#[derive(Debug, Deserialize)]
pub struct ModifierMap {
    pub id: String,
    #[serde(rename = "defaultIndex")]
    #[serde(deserialize_with = "des")]
    pub default_index: u16,
    #[serde(rename = "keyMapSelect")]
    pub key_map_select: Vec<KeyMapSelect>,
}
#[derive(Debug, Deserialize)]
pub struct KeyMapSelect {
    #[serde(rename = "mapIndex")]
    #[serde(deserialize_with = "des")]
    pub map_index: u16,
    pub modifier: Vec<Modifier>,
}
#[derive(Debug, Deserialize)]
pub struct Modifier {
    pub keys: String,
}

#[derive(Debug, Deserialize)]
pub struct KeyMapSet {
    pub id: String,
    #[serde(rename = "keyMap")]
    pub key_map: Vec<KeyMap>
}

#[derive(Debug, Deserialize)]
pub struct KeyMap {
    #[serde(deserialize_with = "des")]
    pub index: u16,
    pub key: Vec<Key>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Key {
    Output {
        #[serde(deserialize_with = "des")]
        code: u16,
        output: String,
    },
    Action {
        #[serde(deserialize_with = "des")]
        code: u16,
        action: String,
    }
}

#[derive(Debug, Deserialize)]
pub struct Actions {
    pub action: Vec<Action>
}

#[derive(Debug, Deserialize)]
pub struct Terminators {
    pub when: Vec<When>
}

#[derive(Debug, Deserialize)]
pub struct Action {
    pub id: String,
    pub when: Vec<When>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum When {
    Output {
        state: String,
        output: String,
    },
    Next {
        state: String,
        next: String,
    }
}

pub fn parse<R: Read>(r: R) -> Result<KeyLayout, Error> {
    deserialize(r)
}

pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        let codepoint = c as u32;
        if codepoint >= 0x20 && codepoint <= 0x7e {
            out.push(c);
        } else {
            out.reserve(7);
            out.push_str("&#x");
            out.push_str(&format!("{:04X}", codepoint));
            out.push(';');
        }
    }
    out
}

pub fn write<W: Write>(w: W, keylayout: KeyLayout) -> WResult<()> {
    let mut writer = EmitterConfig::new().create_writer(w);
    writer.write(XmlEvent::StartDocument {
        version: XmlVersion::Version11,
        encoding: None,
        standalone: None,
    })?;
    let mut w = writer.into_inner();
    w.write_all(b"\n<!DOCTYPE keyboard SYSTEM \"file://localhost/System/Library/DTDs/KeyboardLayout.dtd\">\n")?;

    let mut writer = EmitterConfig{
        perform_escaping: false,
        perform_indent: true,
        .. Default::default()
    }.indent_string("\t").create_writer(w);
    writer.write(XmlEvent::Comment("Created by LFalch's key-layout tool"))?;
    writer.write(XmlEvent::start_element("keyboard")
        .attr("group", &format!("{}", keylayout.group))
        .attr("id", &format!("{}", keylayout.id))
        .attr("name", &keylayout.name)
        // .attr("maxout", &format!("{}", keylayout.maxout))
    )?;
    writer.write(XmlEvent::start_element("layouts"))?;
    for layout in &keylayout.layouts.layout {
        writer.write(XmlEvent::start_element("layout")
            .attr("first", &format!("{}", layout.first))
            .attr("last", &format!("{}", layout.last))
            .attr("mapSet", &layout.map_set)
            .attr("modifiers", &layout.modifiers)
        )?;
        writer.write(XmlEvent::end_element())?;
    }
    writer.write(XmlEvent::end_element())?;

    writer.write(XmlEvent::start_element("modifierMap")
        .attr("id", &keylayout.modifier_map.id)
        .attr("defaultIndex", &format!("{}", keylayout.modifier_map.default_index))
    )?;
    for kms in &keylayout.modifier_map.key_map_select {
        writer.write(XmlEvent::start_element("keyMapSelect")
            .attr("mapIndex", &format!("{}", kms.map_index))
        )?;
        for modifier in &kms.modifier {
            writer.write(XmlEvent::start_element("modifier")
                .attr("keys", &modifier.keys)
            )?;
            writer.write(XmlEvent::end_element())?;
        }
        writer.write(XmlEvent::end_element())?;
    }
    writer.write(XmlEvent::end_element())?;

    writer.write(XmlEvent::start_element("keyMapSet")
        .attr("id", &keylayout.key_map_set.id)
    )?;
    for km in &keylayout.key_map_set.key_map {
        writer.write(XmlEvent::start_element("keyMap")
            .attr("index", &format!("{}", km.index))
        )?;
        for key in &km.key {
            match *key {
                Key::Output{code, ref output} => {
                    writer.write(XmlEvent::start_element("key")
                        .attr("code", &format!("{}", code))
                        .attr("output", &escape(output))
                    )?;
                },
                Key::Action{code, ref action} => {
                    writer.write(XmlEvent::start_element("key")
                        .attr("code", &format!("{}", code))
                        .attr("action", action)
                    )?;
                }
            }
            writer.write(XmlEvent::end_element())?;
        }
        writer.write(XmlEvent::end_element())?;
    }
    writer.write(XmlEvent::end_element())?;

    writer.write(XmlEvent::start_element("actions"))?;
    for action in &keylayout.actions.action {
        writer.write(XmlEvent::start_element("action")
            .attr("id", &action.id)
        )?;
        for when in &action.when {
            match *when {
                When::Output{ref state, ref output} => {
                    writer.write(XmlEvent::start_element("when")
                        .attr("state", state)
                        .attr("output", &escape(output))
                    )?;
                },
                When::Next{ref state, ref next} => {
                    writer.write(XmlEvent::start_element("when")
                        .attr("state", state)
                        .attr("next", next)
                    )?;
                }
            }
            writer.write(XmlEvent::end_element())?;
        }
        writer.write(XmlEvent::end_element())?;
    }
    writer.write(XmlEvent::end_element())?;

    writer.write(XmlEvent::start_element("terminators"))?;
    for when in &keylayout.terminators.when {
        match *when {
            When::Output{ref state, ref output} => {
                writer.write(XmlEvent::start_element("when")
                    .attr("state", state)
                    .attr("output", &escape(output))
                )?;
            },
            When::Next{ref state, ref next} => {
                writer.write(XmlEvent::start_element("when")
                    .attr("state", state)
                    .attr("next", next)
                )?;
            }
        }
        writer.write(XmlEvent::end_element())?;
    }
    writer.write(XmlEvent::end_element())?;

    writer.write(XmlEvent::end_element())?;
    Ok(())
}
