// #[cfg(test)]
// mod tests {
//     #[test]
// 	use speedyxml;
//     fn it_works() {
//         // assert_eq!(2 + 2, 4);
// 		test();
//     }
// }

use std::fs::File;
use std::io::prelude::*;

pub fn test(){
	// println!("hello world" );
	parse("./data/test.xml");
}

// struct document
// {

// }

pub fn parse(filename:&str)
{
	let mut file = File::open(filename).expect("can't find file");
	let mut content = String::new();
	file.read_to_string(&mut content).expect("read error");
	println!("{}", content);
}