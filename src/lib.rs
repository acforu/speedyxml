#![feature(core_intrinsics)]
#![allow(unused_variables)]
#![warn(unused_imports)]
#![warn(unused_must_use)]
#![allow(dead_code)]

use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;
use std::ffi::{CString, CStr};
// struct document
// {

// }
#[derive(Debug)]
struct XmlStr {
	beg: usize,
	end: usize,
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
	attrs: Vec<XmlAttr>,
	children: Vec<Box<XmlNode>>,
}

#[derive(Debug)]
pub struct XmlDocument {
	node: Vec<XmlNode>,
}

// type ParseResult = Result<XmlElement, XmlParseError>;
// fn print_type_of<T>(_: &T) {
//     println!("{}", unsafe { std::intrinsics::type_name::<T>() });
// }
static name_end_chars: &'static [char] = &[' ', '\n', '\r', '\t', '/', '>', '?', '\0'];

// Attribute name (anything but space \n \r \t / < > = ? ! \0)
static invalid_attr_name_chars: &'static [char] =
	&[' ', '\n', '\r', '\t', '/', '<', '>', '=', '?', '!', '\0'];

pub fn parse_content(content:&[u8])-> Result<XmlDocument, XmlParseError> {
	for (i,s) in content.into_iter().enumerate() {
		println!("{},{}", s, i);
	}


	//return Err(XmlParseError::new("msg", 1));

	// let slice = &content[3..6];
 // println!("{}", slice);
 // println!("{}", content.len());

	let mut pos: usize = 0;
	let xml = content;

	// parse_element(&mut pos,str.as_bytes());

	let mut doc: XmlDocument = XmlDocument { node: Vec::new() };

	loop {
		skip_whitespace(&mut pos, xml);

		if xml[pos] == 0 {
			break;
		}

		if xml[pos] == '<' as u8 {
			advance(&mut pos);
			let result = try!(parse_node(&mut pos, xml));
			doc.node.push(result);
		} else {
			break;
		}
	}

	return Ok(doc);
}

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

	return parse_content(content_cstring.as_bytes_with_nul());
	
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

	let node_name: XmlStr = XmlStr {
		beg: name_beg,
		end: *pos,
	};

	skip_whitespace(pos, xml);

	let attr_result = parse_node_attr(pos, xml);
	if attr_result.is_err() {
		return Err(attr_result.err().unwrap());
	}

	if xml[*pos] == '>' as u8 {
		//parse content
  //maybe add children
		advance(pos);
		return pack_error("unimplemented", *pos);
	} else if xml[*pos] == '/' as u8 {
		advance(pos);
		if xml[*pos] != '>' as u8 {
			return pack_error("expected >", *pos);
		}
		advance(pos);
	} else {
		return pack_error("expected >", *pos);
	}

	return Ok(XmlNode {
		name: node_name,
		attrs: attr_result.ok().unwrap(),
		children: Vec::new(),
	});
}

fn parse_node_attr(pos: &mut usize, xml: &[u8]) -> Result<Vec<XmlAttr>, XmlParseError> {
	let mut ret = Vec::new();

	loop {
		let c = xml[*pos] as char;
		if in_chars_set(c, invalid_attr_name_chars) {
			break;
		}

		let attr_name_beg: usize = *pos;

		skip_chars(invalid_attr_name_chars, pos, xml);
		if *pos == attr_name_beg {
			return pack_error("expected attr name", *pos);
		}

		let attr_name = XmlStr {
			beg: attr_name_beg,
			end: *pos,
		};

		skip_whitespace(pos, xml);

		if xml[*pos] != '=' as u8 {
			return pack_error("expected =", *pos);
		}

		advance(pos); //skip =
		skip_whitespace(pos, xml);
		let attr_value_beg = *pos;
		let tag = xml[*pos];
		if tag != '"' as u8 && tag != '\'' as u8 {
			return pack_error("expected attr value", *pos);
		}

		advance(pos); //skip tag
		skip_until(tag as char, pos, xml);
		let attr_value = XmlStr {
			beg: attr_value_beg,
			end: (*pos) - 1,
		};
		ret.push(XmlAttr {
			name: attr_name,
			value: attr_value,
		});
		advance(pos); //skip tag

		// skip_whitespace(pos, xml);
	}

	return Ok(ret);
}

fn advance(pos: &mut usize) {
	*pos = *pos + 1;
}

fn in_chars_set(c: char, set: &[char]) -> bool {
	return set.iter().any(|&x| x == c);
}

fn skip_name(pos: &mut usize, xml: &[u8]) {
	// let c = xml[*pos] as char;
 // for cc in  name_end_chars.into_iter(){
 // 	println!("a:{}b:{}", cc,c);
 // }
 // println!("done");
	while !name_end_chars.iter().any(|&x| x == (xml[*pos] as char)) {
		*pos = *pos + 1;
	}
}

fn skip_until(target: char, pos: &mut usize, xml: &[u8]) {
	while xml[*pos] != target as u8 {
		*pos = *pos + 1;
	}
}

fn skip_char(target: char, pos: &mut usize, xml: &[u8]) {
	while xml[*pos] == target as u8 {
		*pos = *pos + 1;
	}
}

fn skip_chars(skip_chars_set: &[char], pos: &mut usize, xml: &[u8]) {
	while !in_chars_set(xml[*pos] as char, skip_chars_set) {
		*pos = *pos + 1;
	}
}

fn skip_whitespace(pos: &mut usize, xml: &[u8]) {
	skip_char(' ', pos, xml);
}

pub fn test() {
	// println!("hello world" );
//  let content = parse("./data/test.xml");
 // parse("./data/mbcs.txt");
 	let xml = CString::new("<lib count='2'/>").unwrap();
	let content = xml.as_bytes_with_nul();

	let doc = match parse_content(content){
		Ok(doc) => doc,
		Err(e) => panic!("{:?}", e),
	};

	
	println!("{:?}", doc);
	// if res.is_err() {
	// 	println!("{:#?}", res.err().unwrap());
	// } else {
	// 	let doc = res.ok().unwrap();
	// 	println!("{:#?}", doc);
	// }

	// let str = &mut string;
 // println!("{}", "test done")
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn it_works() {
		//assert_eq!(2 + 2, 4);
		test();
	}
}
