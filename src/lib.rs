#![feature(core_intrinsics)]
#![allow(unused_variables)]
#![warn(unused_imports)]
#![warn(unused_must_use)]
use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;



// struct document
// {

// }

struct XmlStr {
	beg: usize,
	end: usize,
}

struct XmlAttr {
	name: XmlStr,
	value: XmlStr,
}

enum XmlParseError {
	element_name_error,
	expect_attr_name,
}

enum XmlElement {
	XmlNode { name: XmlStr, attrs: Vec<XmlAttr> },
	XmlAttrs {attrs: Vec<XmlAttr>},
}

type ParseResult = Result<XmlElement, XmlParseError>;
// fn print_type_of<T>(_: &T) {
//     println!("{}", unsafe { std::intrinsics::type_name::<T>() });
// }
static name_end_chars: &'static [char] = &[' ', '\n', '\r', '\t', '/', '>', '?', '\0'];


// Attribute name (anything but space \n \r \t / < > = ? ! \0)
static invalid_attr_name_chars: &'static [char] =
	&[' ', '\n', '\r', '\t', '/', '<', '>', '=', '?', '!', '\0'];

pub fn parse(filename: &str) {
	let mut file = File::open(filename).expect("can't find file");
	let mut content = String::new();
	file.read_to_string(&mut content).expect("read error");
	// let xml_content = &content;
 // for (i,v) in content.chars().enumerate() {
 // 	// print!("{},{}\n", i,v);
 // 	print_type_of(&v);
 // }

	// for (s,i) in content.as_str().char_indices()
 // {
 // 	println!("{},{}", s,i);
 // }
 // let slice = &content[3..6];
 // println!("{}", slice);
 // println!("{}", content.len());

	let mut pos: usize = 0;
	let xml = content.as_bytes();
	// parse_element(&mut pos,str.as_bytes());

	loop {
		skip_whitespace(&mut pos, xml);

		if xml[pos] == 0 {
			break;
		}

		if xml[pos] == '<' as u8 {
			parse_node(&mut pos, xml);
		} else {
			//#todo throw error
			break;
		}
	}
}

fn parse_node(pos: &mut usize, xml: &[u8]) -> ParseResult {
	let c = xml[*pos];
	match c {
		_ => parse_element(pos, xml),
		//else #todo
	}
}

fn parse_element(pos: &mut usize, xml: &[u8]) -> ParseResult {
	skip_whitespace(pos, xml);
	let name_beg = *pos;
	skip_name(pos, xml);
	if name_beg == *pos {
		return Err(XmlParseError::element_name_error);
	}


	let node_name: XmlStr = XmlStr {
		beg: name_beg,
		end: *pos,
	};

	skip_whitespace(pos, xml);

	let attrResult = parse_node_attr(pos, xml);
	if attrResult.is_err()
	{
		return attrResult.unwrap_err();
	}

	return Ok(XmlElement::XmlNode(node_name,attrResult.unwrap());
}

fn parse_node_attr(pos: &mut usize, xml: &[u8]) -> ParseResult {
	loop {
		let c = xml[*pos] as char;
		if in_chars_set(c, invalid_attr_name_chars) {
			break;
		}

		let attr_name_beg: usize = *pos;

		skip_chars(invalid_attr_name_chars, pos, xml);
		if *pos == attr_name_beg {
			return Err(XmlParseError::expect_attr_name);
		}
	}
	Ok(())
}

fn in_chars_set(c: char, set: &[char]) -> bool {
	return set.iter().any(|&x| x == c);
}


fn skip_name(pos: &mut usize, xml: &[u8]) {
	let c = xml[*pos] as char;
	while !name_end_chars.iter().any(|&x| x == c) {
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
 //parse("./data/test.xml");
 // parse("./data/mbcs.txt");
	parse("./data/utf8.xml");
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
