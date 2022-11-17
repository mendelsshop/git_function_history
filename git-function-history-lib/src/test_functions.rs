use std::error::Error;
use std::fmt::Debug;
pub fn empty_test() {

}

pub fn not_empty_test() {
    println!("not empty test");
    println!("{{ {} added more chages", "{");
}

#[derive(Debug)]
pub struct Test<a> {
    pub contents: String,
    pub history: Vec<a>,
}

impl<a>Test<a>  {
    /// empty test
    pub fn empty_test<'a>() {
        println!("empty test");
    }

    pub fn  not_empty_test( )  {
        println!("not empty test");
        println!("{{ {} added more chages", "{");
    }

    pub fn test_2() {
        pub fn empty_test() {
            println!("empty test");
        }
        println!("empty test");
        // }
    }

    pub fn returns() -> String {
        String::from("fn empty_test() ")
    }

    pub fn test_in_test() {
        println!("test_in_test");
        pub fn empty_test() {

        }

        pub fn test_in_test_in_test() {
            pub fn empty_test() {

            }
        }
    }
}


pub trait super_trait {
    fn super_trait_method(&self);

    fn empty_test<T: Clone + Send>() ->  String where T: super_trait;
}

impl <'a, t> super_trait for t {
    fn super_trait_method (&self) {
        println!("super trait method");}
/*   dff
gdg
*/
        fn empty_test<T>() ->  String where T: super_trait + Clone {
            String::from("fn empty_test() ");
            fn broken() {
                r#"#"}"#;
            }
            String::from("fn empty_test() ")
        }
    
}


trait Example {
    const CONST_NO_DEFAULT: i32;
    const CONST_WITH_DEFAULT: i32 = 99;
    type TypeNoDefault;
    fn method_without_default(&self);
    fn method_with_default(&self) {}
}

trait Seq<T> {
    fn len(&self) -> u32;
    fn elt_at(&self, n: u32) -> T;
    fn iter<F>(&self, f: F) where F: Fn(T);
}
trait super_trait_2 <T> where T: super_trait {
    fn super_trait_method(&self) {
        println!("super trait method");
    }
}
extern "C" {
    fn abs(input: i32) -> i32;
}

/// function why inner doc comments no worky
pub fn function_within(t: String) -> Result<i32, Box<dyn Error>> {
    //! function within
    pub fn empty_test(t: String) -> Result<i32, Box<dyn Error>> {
        println!("empty test");
        match t.parse::<i32>() {
            Ok(i) => Ok(i),
            Err(e) => Err(e)?,
        
            
        }
    }
    empty_test(t)
}

pub struct Test2<A> 
where 
A: 
super_trait {
    pub contents: String,
    pub history: Vec<A>,
}

impl<A> Test2<A,> 
where A:
super_trait  + Clone,
A: Debug + Clone

{
    pub fn empty_test<'a>() {
        println!("empty test");
    }

}

mod c {
    extern "C" {
        pub fn empty_test(t: String);
    }
}


fn main() {
    println!("Hello, world!");
}