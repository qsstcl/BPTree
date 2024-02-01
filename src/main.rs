
use btree::btree::BPTree;
pub mod btree;

fn main() {

    let mut bptree: BPTree<i32, String, 3> = BPTree::new();

    bptree.insert(&1, &String::from("zzz"));
    bptree.insert(&3, &String::from("xxx"));
    let string = String::from("ccc");
    bptree.insert(&5, &string);

    bptree.insert(&7, &String::from("vvv"));
    bptree.insert(&9,
                  &String::from("bbb"));
    let string = String::from("11");

    bptree.insert(&11, &string);
    bptree.insert(&13, &String::from("13"));

    match bptree.search(&1) {
        Some(result)=> {
            let output = result.clone();
            let output_string = (*output).borrow();
            println!("{}",output_string);
        }
        None =>{}
    } 
}
