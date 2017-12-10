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
    group: u16,
    #[serde(deserialize_with = "des")]
    id: i32,
    name: String,
    #[serde(deserialize_with = "des")]
    maxout: u16,
    layouts: Layouts,
    #[serde(rename = "modifierMap")]
    modifier_map: ModifierMap,
    #[serde(rename = "keyMapSet")]
    key_map_set: KeyMapSet
    // TODO actions
}

#[derive(Debug, Deserialize)]
pub struct Layouts {
    layout: Vec<Layout>,
}
#[derive(Debug, Deserialize)]
pub struct Layout {
    #[serde(deserialize_with = "des")]
    first: u16,
    #[serde(deserialize_with = "des")]
    last: u16,
    #[serde(rename = "mapSet")]
    map_set: String,
    modifiers: String,
}
#[derive(Debug, Deserialize)]
pub struct ModifierMap {
    id: String,
    #[serde(rename = "defaultIndex")]
    #[serde(deserialize_with = "des")]
    default_index: u16,
    #[serde(rename = "keyMapSelect")]
    key_map_select: Vec<KeyMapSelect>,
}
#[derive(Debug, Deserialize)]
pub struct KeyMapSelect {
    #[serde(rename = "mapIndex")]
    #[serde(deserialize_with = "des")]
    map_index: u16,
    modifier: Vec<Modifier>,
}
#[derive(Debug, Deserialize)]
pub struct Modifier {
    keys: String,
}

#[derive(Debug, Deserialize)]
pub struct KeyMapSet {
    id: String,
    #[serde(rename = "keyMap")]
    key_map: Vec<KeyMap>
}

#[derive(Debug, Deserialize)]
pub struct KeyMap {
    #[serde(deserialize_with = "des")]
    index: u16,
    key: Vec<Key>,
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

pub fn parse<R: Read>(r: R) -> Result<KeyLayout, Error> {
    deserialize(r)
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
        .attr("maxout", &format!("{}", keylayout.maxout))
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
                Key::Output{
                    code,
                    ref output
                } => {
                    let mut out = String::with_capacity(output.len());
                    for c in output.chars() {
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
                    writer.write(XmlEvent::start_element("key")
                        .attr("code", &format!("{}", code))
                        .attr("output", &out)
                    )?;
                },
                Key::Action{
                    code,
                    ref action
                } => {
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

    writer.write(XmlEvent::end_element())?;
    Ok(())
}
