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

    pub fn returns () -> String {
        String::from("returns")
    }
}