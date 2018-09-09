 #![feature(core_intrinsics)]
 #![allow(unused_variables)]
 #![allow(unused_imports)]
 #![allow(unused_must_use)]
 #![allow(dead_code)]
 #![allow(non_upper_case_globals)]

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

impl fmt::Display for XmlAttr {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, " {}=\"{}\"", self.name, self.value)
	}
}

#[derive(Debug)]
pub struct XmlParseError {
	msg: &'static str,
	index: usize,
}

fn pack_error<T>(msg: &'static str, index: usize) -> Result<T, XmlParseError> {
	return Err(XmlParseError { msg, index });
}

// #[derive(Debug)]
// pub struct XmlNode {
// 	name: XmlStr,
// 	value: XmlStr,
// 	attrs: Vec<XmlAttr>,
// 	children: Vec<Box<XmlNode>>,
// }

#[derive(Debug)]
pub struct XmlElement {
	name: XmlStr,
	value: XmlStr,
	attrs: Vec<XmlAttr>,
	children: Vec<Box<XmlNode>>,
}

impl fmt::Display for XmlElement {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "<{}", self.name);
		for attr in self.attrs.iter() {
			write!(f, "{}", attr);
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
pub struct XmlComment {
	value: XmlStr,
}

impl fmt::Display for XmlComment {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "<!--{}-->", self.value)
	}
}

#[derive(Debug)]
pub struct XmlCData {
	value: XmlStr,
}

impl fmt::Display for XmlCData {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "<![CDATA[{}]]>", self.value)
	}
}

#[derive(Debug)]
pub struct XmlDeclaration {
	attrs: Vec<XmlAttr>,
}

impl fmt::Display for XmlDeclaration {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "<?xml");
		for child in self.attrs.iter() {
			write!(f, "{}", child);
		}
		write!(f, "?>")
	}
}


#[derive(Debug)]
pub struct XmlPi {
	name:XmlStr,
	value:XmlStr
}

impl fmt::Display for XmlPi {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "<?{} {}?>",self.name,self.value)
	}
}


#[derive(Debug)]
pub enum XmlNode {
	XmlElement(XmlElement),
	XmlComment(XmlComment),
	XmlCData(XmlCData),
	XmlDeclaration(XmlDeclaration),
	XmlPi(XmlPi),
	Undefine,
}

impl fmt::Display for XmlNode {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			XmlNode::XmlElement(ref element) => write!(f, "{}", element),
			XmlNode::XmlComment(ref comment) => write!(f, "{}", comment),
			XmlNode::XmlCData(ref cdata) => write!(f, "{}", cdata),
			XmlNode::XmlDeclaration(ref declaration) => write!(f, "{}", declaration),
			XmlNode::XmlPi(ref pi) => write!(f, "{}", pi),
			_ => write!(f, ""),
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

	if c == '!' as u8 {
		match xml[*pos + 1] as char {
			'-' => {
				if xml[*pos + 2] == '-' as u8 {
					advance_n(pos, 3);
					return parse_comment(pos, xml);
				}
			}
			'[' => {
				if is_begin_with(xml, *pos + 2, "CDATA[") {
					advance_n(pos, 8);
					return parse_cdata(pos, xml);
				}
			}
			_ => {
				return pack_error("todo", *pos);
			}
		}

		advance(pos); // Skip !
		while xml[*pos] != '>' as u8 {
			if xml[*pos] == 0 {
				return pack_error("unexpected end of data", *pos);
			}
			advance(pos);
		}
		advance(pos); // Skip '>'
		return Ok(XmlNode::Undefine);
	} else if c == '?' as u8 {
		advance(pos);
		let p = *pos;
		if (xml[p] == 'x' as u8 || xml[p] == 'X' as u8) 
		&& (xml[p] == 'x' as u8 || xml[p] == 'M' as u8) 
		&& (xml[p] == 'x' as u8 || xml[p] == 'L' as u8) 
		&& in_chars_set(xml[p + 3] as char, space_chars)
		{
			advance_n(pos, 4);
			return parse_declaration(pos,xml);
		} else {
			return parse_pi(pos, xml);
		}
	} else {
		parse_element(pos, xml)
	}
}

fn parse_declaration(pos: &mut usize, xml: &[u8]) -> Result<XmlNode, XmlParseError> {
	skip_whitespace(pos, xml);
	let attrs = try!(parse_node_attr(pos, xml));

	if xml[*pos] != '?' as u8 || xml[*pos+1] != '>' as u8 {
		return pack_error("expected ?>", *pos);
	}
	advance_n(pos,2);
	Ok(XmlNode::XmlDeclaration(XmlDeclaration{attrs})) 
}

fn parse_comment(pos: &mut usize, xml: &[u8]) -> Result<XmlNode, XmlParseError> {
	let comment_beg = *pos;
	while !is_begin_with(xml, *pos, "-->") {
		if xml[*pos] == 0 {
			return pack_error("unexpect end", *pos);
		}
		advance(pos);
	}

	let comment = XmlComment {
		value: XmlStr::from_range(xml, comment_beg, *pos),
	};
	advance_n(pos, 3);
	return Ok(XmlNode::XmlComment(comment));
}

fn parse_cdata(pos: &mut usize, xml: &[u8]) -> Result<XmlNode, XmlParseError> {
	let beg = *pos;
	while !is_begin_with(xml, *pos, "]]>") {
		if xml[*pos] == 0 {
			return pack_error("unexpect end", *pos);
		}
		advance(pos);
	}

	let cdata = XmlCData {
		value: XmlStr::from_range(xml, beg, *pos),
	};
	advance_n(pos, 3);
	return Ok(XmlNode::XmlCData(cdata));
}

fn parse_pi(pos: &mut usize, xml: &[u8]) -> Result<XmlNode, XmlParseError> {
	let beg = *pos;
	skip_name(pos, xml);
	if beg == *pos{
		return pack_error("expect pi name", beg);
	}

	let name = XmlStr::from_range(xml, beg, *pos);

	skip_whitespace(pos, xml);

	let beg = *pos;

	while !is_begin_with(xml, *pos, "?>") {
		if xml[*pos] == 0 {
			return pack_error("unexpect end", *pos);
		}
		advance(pos);
	}

	let pi = XmlPi { 
		name : name,
		value: XmlStr::from_range(xml, beg, *pos),
	};

	advance_n(pos, 2);
	return Ok(XmlNode::XmlPi(pi));
}

fn is_begin_with(xml: &[u8], mut pos: usize, string: &'static str) -> bool {
	for c in string.as_bytes() {
		if xml[pos] == *c {
			pos = pos + 1;
		} else {
			return false;
		}
	}
	return true;
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

	let mut ret_node = XmlNode::XmlElement(XmlElement {
		name: node_name,
		value: node_value,
		attrs: attr_result,
		children: Vec::new(),
	});

	if xml[*pos] == '>' as u8 {
		advance(pos);
		let maybe_content_beg = *pos;

		skip_whitespace(pos, xml);

		if xml[*pos] == '<' as u8 {
			if xml[*pos + 1] == '/' as u8 {
				try!(parse_close_node(pos, xml, node_name));
			} else {
				// match ret_node {
	// 	XmlNode::XmlElement(ref element) => {
	// 		element.children = try!(parse_children_nodes(pos, xml, node_name));
	// 	}
	// }

				if let XmlNode::XmlElement(ref mut element) = ret_node {
					element.children = try!(parse_children_nodes(pos, xml, node_name));
				}
				// ret_node.children = try!(parse_children_nodes(pos, xml, node_name));
			}
		} else {
			skip_text(pos, xml);
			// ret_node.value = XmlStr::from_range(xml, maybe_content_beg, *pos);

			if let XmlNode::XmlElement(ref mut element) = ret_node {
				element.value = XmlStr::from_range(xml, maybe_content_beg, *pos);
			}

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

		skip_whitespace(pos, xml);
	}

	return Ok(ret);
}

pub fn parse_content(content: &[u8]) -> Result<Vec<Box<XmlNode>>, XmlParseError> {
//	for (i, s) in content.into_iter().enumerate() {
//		println!("{},{}", i, *s as char);
//	}

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

fn parse_children_nodes(
	pos: &mut usize,
	xml: &[u8],
	node_name: XmlStr,
) -> Result<Vec<Box<XmlNode>>, XmlParseError> {
	let mut nodes = Vec::new();

	loop {
		skip_whitespace(pos, xml);

		if xml[*pos] == 0 {
			break;
		}

		if xml[*pos] == '<' as u8 {
			if xml[*pos + 1] == '/' as u8 {
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

fn advance_n(pos: &mut usize, n: usize) {
	*pos = *pos + n;
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
		r##"<?xml-stylesheet href="mystyle.css" type="text/css"?>"##, 
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
		let mut unchange_cases = Vec::new();

		unchange_cases.push(r#"<lib count="2">hello</lib>"#);
		unchange_cases.push(r#"<lib count="2"><!-- comment --></lib>"#);
		unchange_cases.push(r#"<?xml-stylesheet href="mystyle.css" type="text/css"?>"#);

		unchange_cases.push(
			"<!-- comment --><script><![CDATA[
function matchwo(a,b)
{
if (a < b && a < 0) then
  {
  return 1;
  }
else
  {
  return 0;
  }
}
]]></script>",
		);

		unchange_cases.push(r##"<?xml version="1.0" encoding="UTF-8" standalone="no"?>"##);

		let mut change_cases = Vec::new();
		change_cases.push((r#"<lib count="2"></lib>"#, r#"<lib count="2"/>"#));
		change_cases.push((
			r#"<lib count="2">
					<book isbn="10">math</book>
					<book isbn="20">english</book>
			</lib>"#,
			r#"<lib count="2"><book isbn="10">math</book><book isbn="20">english</book></lib>"#,
		));

		for case in unchange_cases.iter() {
			parse_print_xml(String::from(*case), String::from(*case)); 
		}

		for case in change_cases.iter() {
			parse_print_xml(String::from(case.0), String::from(case.1))
		}
	}
}
