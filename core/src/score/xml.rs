//! Minimal XML tree for MusicXML (no mixed content), read with quick-xml.

use quick_xml::Reader;
use quick_xml::XmlVersion;
use quick_xml::events::{BytesStart, Event};

#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) struct Element {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<Element>,
    pub text: String,
}

impl Element {
    pub fn child(&self, name: &str) -> Option<&Element> {
        self.children.iter().find(|c| c.name == name)
    }

    pub fn child_mut(&mut self, name: &str) -> Option<&mut Element> {
        self.children.iter_mut().find(|c| c.name == name)
    }

    pub fn children<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Element> {
        self.children.iter().filter(move |c| c.name == name)
    }

    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    pub fn has(&self, name: &str) -> bool {
        self.child(name).is_some()
    }

    pub fn child_text(&self, name: &str) -> Option<&str> {
        self.child(name).map(|c| c.text.trim())
    }

    pub fn write(&self, out: &mut String) {
        out.push('<');
        out.push_str(&self.name);
        for (key, value) in &self.attributes {
            out.push_str(&format!(" {key}=\"{}\"", escape(value)));
        }
        if self.children.is_empty() && self.text.is_empty() {
            out.push_str("/>");
            return;
        }
        out.push('>');
        // Text between child elements is only indentation (MusicXML has no mixed content).
        if self.children.is_empty() {
            out.push_str(&escape(&self.text));
        }
        for child in &self.children {
            child.write(out);
        }
        out.push_str("</");
        out.push_str(&self.name);
        out.push('>');
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub(crate) fn parse(xml: &str) -> Result<Element, String> {
    let mut reader = Reader::from_str(xml);
    let mut stack: Vec<Element> = Vec::new();
    loop {
        match reader.read_event().map_err(|e| e.to_string())? {
            Event::Start(start) => stack.push(element(&start)?),
            Event::Empty(start) => {
                let element = element(&start)?;
                match stack.last_mut() {
                    Some(parent) => parent.children.push(element),
                    None => return Ok(element),
                }
            }
            Event::End(_) => {
                let element = stack.pop().ok_or("unexpected end tag")?;
                match stack.last_mut() {
                    Some(parent) => parent.children.push(element),
                    None => return Ok(element),
                }
            }
            Event::Text(text) => {
                if let Some(top) = stack.last_mut() {
                    top.text.push_str(&text.xml10_content());
                }
            }
            Event::CData(data) => {
                if let Some(top) = stack.last_mut() {
                    top.text.push_str(&data.into_inner());
                }
            }
            Event::GeneralRef(reference) => {
                if let Some(top) = stack.last_mut() {
                    let character = match reference.resolve_char_ref().map_err(|e| e.to_string())? {
                        Some(character) => character,
                        None => resolve_entity(&reference.xml10_content())?,
                    };
                    top.text.push(character);
                }
            }
            Event::Eof => return Err("no root element".into()),
            _ => {}
        }
    }
}

fn resolve_entity(name: &str) -> Result<char, String> {
    match name {
        "amp" => Ok('&'),
        "lt" => Ok('<'),
        "gt" => Ok('>'),
        "quot" => Ok('"'),
        "apos" => Ok('\''),
        _ => Err(format!("unknown entity &{name};")),
    }
}

fn element(start: &BytesStart) -> Result<Element, String> {
    let mut attributes = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(|e| e.to_string())?;
        let value = attribute
            .normalized_value(XmlVersion::Implicit1_0)
            .map_err(|e| e.to_string())?;
        attributes.push((attribute.key.as_ref().to_owned(), value.into_owned()));
    }
    Ok(Element {
        name: start.name().as_ref().to_owned(),
        attributes,
        ..Element::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_keeps_structure_and_entities() {
        let tree = parse(r#"<?xml version="1.0"?><a x="1&amp;2"><b>T &lt; U</b><c/></a>"#).unwrap();
        assert_eq!(tree.attribute("x"), Some("1&2"));
        assert_eq!(tree.child_text("b"), Some("T < U"));
        let mut out = String::new();
        tree.write(&mut out);
        assert_eq!(out, r#"<a x="1&amp;2"><b>T &lt; U</b><c/></a>"#);
    }
}
