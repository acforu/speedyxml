// #![feature(core_intrinsics)]
// #![allow(unused_variables)]
// #![warn(unused_imports)]
// #![warn(unused_must_use)]
// #![allow(dead_code)]

use std::fs::File;
use std::io::prelude::*;
use std::ffi::{CStr, CString};
use std::str::*;
use std::slice;
use std::fmt;
use std::fmt::Write;
use std::ptr;
// struct document
// {

// }
#[derive(Debug, Copy, Clone)]
struct XmlStr {
	data: *const u8,
	length: usize,
}

impl XmlStr {
	fn from(ptr: *const u8, length: usize) -> XmlStr {
		XmlStr { data: ptr, length }
	}

	fn from_range(content: &[u8], beg: usize, end: usize) -> XmlStr {
		unsafe {
			XmlStr {
				data: content.as_ptr().offset(beg as isize),
				length: end - beg,
			}
		}
	}

	fn as_slice(&self) -> &str {
		unsafe {
			let bytes = slice::from_raw_parts(self.data, self.length);
			from_utf8_unchecked(bytes)
		}
	}

	fn null_str() -> XmlStr {
		XmlStr::from(ptr::null(), 0)
	}

	fn is_empty(&self) -> bool {
		return self.length == 0;
	}
}

impl PartialEq for XmlStr {
	fn eq(&self, other: &XmlStr) -> bool {
		if self.data == other.data && self.length == other.length {
			return true;
		} else {
			if self.length != other.length {
				return false;
			} else {
				for i in 0..self.length {
					unsafe {
						if *self.data.offset(i as isize) != *other.data.offset(i as isize) {
							return false;
						}
					}
				}
				return true;
			}
		}
	}
}

impl fmt::Display for XmlStr {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.as_slice())
	}
}

#[derive(Debug)]
struct XmlAttr {
	name: XmlStr,
	value: XmlStr,
}

// enum XmlParseError {
// 	element_name_error,
// 	expect_attr_name,
// 	expect_attr_equal,
// 	expect_attr_value,
// 	unknow_error,
// 	unimplemented,
// }
#[derive(Debug)]
pub struct XmlParseError {
	msg: &'static str,
	index: usize,
}

// impl XmlParseError {
// 	fn XmlParseError<T>(msg: &'static str, index: usize) -> Result<T,XmlParseError> {
// 		return Err(XmlParseError { msg, index });
// 	}
// }

fn pack_error<T>(msg: &'static str, index: usize) -> Result<T, XmlParseError> {
	return Err(XmlParseError { msg, index });
}

#[derive(Debug)]
pub struct XmlNode {
	name: XmlStr,
	value: XmlStr,
	attrs: Vec<XmlAttr>,
	children: Vec<Box<XmlNode>>,
}

impl fmt::Display for XmlNode {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "<{} ", self.name);
		for attr in self.attrs.iter() {
			write!(f, "{}='{}'", attr.name, attr.value);
		}
		if self.value.is_empty() {
			if self.children.is_empty() {
				write!(f, "/>")
			} else {
				write!(f, ">");
				for child in self.children.iter() {
					write!(f, "{}", child);
				}
				write!(f, "</{}>", self.name)
			}
		} else {
			write!(f, ">{}</{}>", self.value, self.name)
		}
	}
}

#[derive(Debug)]
pub struct XmlDocument {
	content: CString,
	nodes: Vec<Box<XmlNode>>,
}

impl XmlDocument {
	pub fn print(&self) -> String {
		let mut buf = String::new();
		for node in self.nodes.iter() {
			write!(&mut buf, "{}", node);
		}
		buf
	}
}

// type ParseResult = Result<XmlElement, XmlParseError>;
// fn print_type_of<T>(_: &T) {
//     println!("{}", unsafe { std::intrinsics::type_name::<T>() });
// }
static name_end_chars: &'static [char] = &[' ', '\n', '\r', '\t', '/', '>', '?', '\0'];
// Attribute name (anything but space \n \r \t / < > = ? ! \0)
static invalid_attr_name_chars: &'static [char] =
	&[' ', '\n', '\r', '\t', '/', '<', '>', '=', '?', '!', '\0'];
static invalid_text_chars: &'static [char] = &['<', '\0'];
static space_chars: &'static [char] = &[' ', '\n', '\r', '\t'];

pub fn parse(filename: &str) -> Result<XmlDocument, XmlParseError> {
	let mut file = File::open(filename).expect("can't find file");
	let mut content = String::new();
	file.read_to_string(&mut content).expect("read error");

	// let xml_content = &content;
 // for (i,v) in content.chars().enumerate() {
 // 	// print!("{},{}\n", i,v);
 // 	print_type_of(&v);
 // }

	let content_cstring = CString::new(content).expect("convert to u8 error");

	return parse_cstring(content_cstring);
}

fn parse_cstring<'a>(content: CString) -> Result<XmlDocument, XmlParseError> {
	let nodes = try!(parse_content(content.as_bytes_with_nul()));

	let doc = XmlDocument { content, nodes };
	return Ok(doc);
}

fn parse_node(pos: &mut usize, xml: &[u8]) -> Result<XmlNode, XmlParseError> {
	let c = xml[*pos];
	match c {
		_ => parse_element(pos, xml),
		//else #todo
	}
}

fn parse_element(pos: &mut usize, xml: &[u8]) -> Result<XmlNode, XmlParseError> {
	skip_whitespace(pos, xml);
	let name_beg = *pos;
	skip_name(pos, xml);
	if name_beg == *pos {
		return pack_error("element_name_error", *pos);
	}

	let node_name: XmlStr = XmlStr::from_range(xml, name_beg, *pos);
	let node_value = XmlStr::null_str();
	skip_whitespace(pos, xml);

	let attr_result = try!(parse_node_attr(pos, xml));

	let mut ret_node = XmlNode {
		name: node_name,
		value: node_value,
		attrs: attr_result,
		children: Vec::new(),
	};

	if xml[*pos] == '>' as u8 {
		advance(pos);
		let maybe_content_beg = *pos;

		skip_whitespace(pos, xml);

		if xml[*pos] == '<' as u8 {
			if xml[*pos + 1] == '/' as u8 {
				try!(parse_close_node(pos, xml, node_name));
			} else {
				ret_node.children = try!(parse_children_nodes(pos, xml,node_name));
				// skip_whitespace(pos, xml);
				// try!(parse_close_node(pos, xml, node_name));
			}
		} else {
			skip_text(pos, xml);
			ret_node.value = XmlStr::from_range(xml, maybe_content_beg, *pos);
			try!(parse_close_node(pos, xml, node_name));
		}
	} else if xml[*pos] == '/' as u8 {
		advance(pos);
		if xml[*pos] != '>' as u8 {
			return pack_error("expected >", *pos);
		}
		advance(pos);
	} else {
		return pack_error("expected >", *pos);
	}
	return Ok(ret_node);
}

//parse with "</"
fn parse_close_node(pos: &mut usize, xml: &[u8], node_name: XmlStr) -> Result<(), XmlParseError> {
	if xml[*pos] == '<' as u8 {
		advance(pos);
		if xml[*pos] == '/' as u8 {
			advance(pos);
			let name_beg = *pos;
			skip_name(pos, xml);
			let close_node_name = XmlStr::from_range(xml, name_beg, *pos);
			if close_node_name != node_name {
				return pack_error("node name mismatch", *pos);
			} else {
				advance(pos);
				return Ok(());
			}
		}
	}
	return pack_error("parse close node error", *pos);
}

fn parse_node_attr(pos: &mut usize, xml: &[u8]) -> Result<Vec<XmlAttr>, XmlParseError> {
	let mut ret = Vec::new();

	loop {
		let c = xml[*pos] as char;
		if in_chars_set(c, invalid_attr_name_chars) {
			break;
		}

		let attr_name_beg: usize = *pos;

		skip_until_met_chars(invalid_attr_name_chars, pos, xml);
		if *pos == attr_name_beg {
			return pack_error("expected attr name", *pos);
		}

		let attr_name = XmlStr::from_range(xml, attr_name_beg, *pos);

		skip_whitespace(pos, xml);

		if xml[*pos] != '=' as u8 {
			return pack_error("expected =", *pos);
		}

		advance(pos); //skip =
		skip_whitespace(pos, xml);
		let tag = xml[*pos];
		if tag != '"' as u8 && tag != '\'' as u8 {
			return pack_error("expected attr value", *pos);
		}

		advance(pos); //skip tag
		let attr_value_beg = *pos;
		skip_until(tag as char, pos, xml);
		let attr_value = XmlStr::from_range(xml, attr_value_beg, *pos);

		ret.push(XmlAttr {
			name: attr_name,
			value: attr_value,
		});
		advance(pos); //skip tag

		// skip_whitespace(pos, xml);
	}

	return Ok(ret);
}

pub fn parse_content(content: &[u8]) -> Result<Vec<Box<XmlNode>>, XmlParseError> {
	for (i, s) in content.into_iter().enumerate() {
		println!("{},{}", i, *s as char);
	}

	//return Err(XmlParseError::new("msg", 1));

	// let slice = &content[3..6];
 // println!("{}", slice);
 // println!("{}", content.len());

	let mut pos: usize = 0;
	let xml = content;

	// parse_element(&mut pos,str.as_bytes());

	return parse_nodes(&mut pos, xml);
}

fn parse_nodes(pos: &mut usize, xml: &[u8]) -> Result<Vec<Box<XmlNode>>, XmlParseError> {
	let mut nodes = Vec::new();

	loop {
		skip_whitespace(pos, xml);

		if xml[*pos] == 0 {
			break;
		}

		if xml[*pos] == '<' as u8 {
			advance(pos);
			let result = try!(parse_node(pos, xml));
			nodes.push(Box::new(result));
		} else {
			break;
		}
	}

	return Ok(nodes);
}

fn parse_children_nodes(pos: &mut usize,xml: &[u8],node_name: XmlStr) -> Result<Vec<Box<XmlNode>>, XmlParseError> {
	let mut nodes = Vec::new();

	loop {
		skip_whitespace(pos, xml);

		if xml[*pos] == 0 {
			break;
		}

		if xml[*pos] == '<' as u8 {
			if xml[*pos+1] == '/' as u8 {
				try!(parse_close_node(pos, xml, node_name));
			} else {
				// todo advance
				advance(pos);
				let result = try!(parse_node(pos, xml));
				nodes.push(Box::new(result));
			}
		} else {
			break;
		}
	}

	return Ok(nodes);
}

fn advance(pos: &mut usize) {
	*pos = *pos + 1;
}

fn in_chars_set(c: char, set: &[char]) -> bool {
	return set.iter().any(|&x| x == c);
}

fn skip_name(pos: &mut usize, xml: &[u8]) {
	skip_until_met_chars(name_end_chars, pos, xml);
}

fn skip_text(pos: &mut usize, xml: &[u8]) {
	skip_until_met_chars(invalid_text_chars, pos, xml);
}

fn skip_until(target: char, pos: &mut usize, xml: &[u8]) {
	while xml[*pos] != target as u8 && xml[*pos] != 0 {
		*pos = *pos + 1;
	}
}

fn skip_char(target: char, pos: &mut usize, xml: &[u8]) {
	while xml[*pos] == target as u8 {
		*pos = *pos + 1;
	}
}

fn skip_until_met_chars(skip_chars_set: &[char], pos: &mut usize, xml: &[u8]) {
	while !in_chars_set(xml[*pos] as char, skip_chars_set) {
		*pos = *pos + 1;
	}
}

fn skip_chars(skip_chars_set: &[char], pos: &mut usize, xml: &[u8]) {
	while in_chars_set(xml[*pos] as char, skip_chars_set) {
		*pos = *pos + 1;
	}
}

fn skip_whitespace(pos: &mut usize, xml: &[u8]) {
	skip_chars(space_chars, pos, xml);
}

pub fn test() {
	let xml = CString::new(
		"<lib count='2'>
					<book isbn='10'>math</book>
					<book isbn='20'>english</book>
			</lib>",
	).unwrap();
	let doc = match parse_cstring(xml.clone()) {
		Ok(doc) => doc,
		Err(e) => panic!("{:?}", e),
	};

	//println!("{:#?}", doc);
 // println!("{}", doc.print());
	println!("{}", doc.print());
	assert_eq!(xml.as_bytes(), doc.print().as_bytes());
}

pub fn parse_print_xml(text: String, dst: String) {
	let xml = CString::new(text).unwrap();
	let doc = match parse_cstring(xml.clone()) {
		Ok(doc) => doc,
		Err(e) => panic!("{:?}", e),
	};

	//println!("{:#?}", doc);
 // println!("{}", doc.print());
	assert_eq!(dst.as_bytes(), doc.print().as_bytes());
}

#[cfg(test)]
mod tests {
	use super::*;
	// #[test]
 // fn it_works() {
 // 	//assert_eq!(2 + 2, 4);
 // 	test();
 // }

	#[test]
	fn test_parse_print_xml() {
		let unchange_xml = [
			"<lib count='2'>hello</lib>",
		];

		let mut change_xml = Vec::new();
		
		change_xml.push(("<lib count='2'></lib>", "<lib count='2'/>"));
		change_xml.push(
			("<lib count='2'>
					<book isbn='10'>math</book>
					<book isbn='20'>english</book>
			</lib>", 
			"<lib count='2'><book isbn='10'>math</book><book isbn='20'>english</book></lib>"));


		for case in unchange_xml.iter() {
			parse_print_xml(String::from(*case), String::from(*case));
		}

		for case in change_xml.iter() {
			parse_print_xml(String::from(case.0), String::from(case.1))
		}
	}
}
