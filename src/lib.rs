// #[cfg(test)]
// mod tests {
//     #[test]
// 	use speedyxml;
//     fn it_works() {
//         // assert_eq!(2 + 2, 4);
// 		test();
//     }
// }
#![feature(core_intrinsics)]
use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;

pub fn test(){
	// println!("hello world" );
	//parse("./data/test.xml");
	// parse("./data/mbcs.txt");
	parse("./data/utf8.txt");
	// let str = &mut string;

}

// struct document
// {

// }

struct XmlStr
{
	beg:usize,
	end:usize,
}

struct XmlAttr
{
	name:XmlStr,
	value:XmlStr,
}

struct XmlNode
{
	name:XmlStr,
	attrs:Vec<XmlAttr>,
}

// fn print_type_of<T>(_: &T) {
//     println!("{}", unsafe { std::intrinsics::type_name::<T>() });
// }
static name_end_chars:&'static[char] = &[' ','\n','\r','\t','/','>','?','\0'];

pub fn parse(filename:&str)
{
	let mut file = File::open(filename).expect("can't find file");
	let mut content = String::new();
	file.read_to_string(&mut content).expect("read error");
	let xml_content = &content;
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

	let str = "  12";
	let mut pos:usize = 0;
	let xml = str.as_bytes();
	// parse_element(&mut pos,str.as_bytes());

	loop{
		skip_whitespace(&mut pos,xml);

		if xml[pos] == 0
		{
			break;
		}

		if xml[pos] == '<' as u8
		{
			parse_node(&mut pos,xml);
		}
		else
		{
			//#todo throw error
		}
		
	}

}

fn parse_node(pos:&mut usize,xml:&[u8])
{
	let c = xml[*pos];
	match c
	{
		_ => parse_element(pos,xml),
		//else #todo
	}
}

fn parse_element(pos:&mut usize,xml:&[u8])
{
	skip_whitespace(pos,xml);

}

fn skip_name()
{
	//#todo 
}

fn skip_char(target:char,pos:&mut usize,xml:&[u8])
{
	while xml[*pos] == target as u8
	{
		*pos = *pos + 1;
	}	
}

fn skip_chars(target:&[u8],pos:&mut usize,xml:&[u8])
{

}

fn skip_whitespace(pos:&mut usize,xml:&[u8])
{
	skip_char(' ',pos,xml);
}