pub fn empty_test() {

}

pub fn not_empty_test() {
    println!("not empty test");
    println!("{{ {} added more chages", "{");
}

pub struct Test;

impl Test {
    pub fn empty_test() {
        println!("empty test");
    }

    pub fn  not_empty_test ( )  {
        println!("not empty test");
        println!("{{ {} added more chages", "{");
    }

    pub fn test_2() {
        println!("empty test");
        // }
    }

    pub fn returns () -> String {
        String::from("fn empty_test() ")
    }
}

pub trait super_trait {
    fn super_trait_method(&self);

    fn empty_test<T>() ->  String;
}