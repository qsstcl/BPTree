
use std::cell:: RefCell;
use std::rc::Rc;
use crate::btree::node::*;

impl Key for i32 {}

impl Record for String {}

#[derive(Debug)]
pub struct BPTree<K: Key, V: Record, const FANOUT: usize> {
    root: Option<NodePtr<K, V, FANOUT>>,
}

impl<K: Key, V: Record, const FANOUT: usize> BPTree<K, V, FANOUT> {
    pub fn new() -> Self {
        assert!(FANOUT >= 2);
        Self { root: None }
    }

    pub fn search(&self, key: &K) -> Option<RecordPtr<V>> {
        let leaf = self.get_leaf_node(key)?;
        let leaf_borrow = leaf.borrow();

        let leaf = leaf_borrow.unwrap_leaf();
        for i in 0..leaf.num_keys {
            if *key == *leaf.keys[i].as_ref()? {
                return Some(leaf.records[i].as_ref()?.clone());
            }
        }
        None
    }

    pub fn search_range(&self, start: &K, end: &K) -> Vec<RecordPtr<V>> {
        let mut res = vec![];
        let mut leaf = self.get_leaf_node(start);
        if leaf.is_none() {
            return res;
        }
        // find the position within the leaf
        let mut i = 0;
        
        let leaf_borrow = leaf.as_ref().unwrap().borrow();
        {
            let leaf_node = leaf_borrow.unwrap_leaf();

            while i < leaf_node.num_keys && leaf_node.keys[i].as_ref().unwrap() < start {
                i += 1;
            }
            if i >= leaf_node.num_keys {
                return res;
            }
        }

        drop(leaf_borrow);

        while !leaf.is_none() {
            let next;
            let leaf_borrow = leaf.as_ref().unwrap().borrow();
            {
                let leaf_node = leaf_borrow.unwrap_leaf();

                while i < leaf_node.num_keys && leaf_node.keys[i].as_ref().unwrap() <= end {
                    res.push(leaf_node.records[i].as_ref().unwrap().clone());
                    i += 1;
                }
                next = leaf_node.next.clone();

                if i != leaf_node.num_keys {
                    break;
                }
            }

            drop(leaf_borrow);
            if next.is_none() {
                break;
            }
            leaf = next.unwrap().upgrade();
            i = 0;
        }

        return res;
    }

    pub fn insert(&mut self, key: &K, record: &V) {
        // There are 4 cases for insert

        // 1 - key exists so just update the record
        let searched_record = self.search(key);
        if let Some(value) = searched_record {
            let mut borrow_value = value.borrow_mut();
            *borrow_value = record.clone();
            return;
        }

        // 2 - Empty tree
        if self.root.is_none() {
            let mut new_node = Node::new_leaf();
            new_node.keys[0] = Some(key.clone());
            new_node.records[0] = Some(Rc::new(RefCell::new(record.clone())));
            new_node.num_keys += 1;
            self.root = Some(Rc::new(RefCell::new(Node::Leaf(new_node))));
            return;
        }

        let leaf_node = self.get_leaf_node(key).unwrap();
        let mut leaf_borrow = leaf_node.borrow_mut();

        // 3 - the leaf need to be splitted
        let leaf = leaf_borrow.unwrap_leaf_mut();
        if leaf.num_keys == FANOUT - 1 {
            let mut new_leaf: Leaf<K, V, FANOUT> = Node::new_leaf();
            let split = FANOUT / 2;
            let mut insertion_idx = 0;
            while insertion_idx < leaf.num_keys && leaf.keys[insertion_idx].as_ref().unwrap() < key
            {
                insertion_idx += 1;
            }

            // The node has FANOUT - 1 keys, so we split at (FANOUT - 1) / 2
            // Example, if FANOUT is 3, there are 2 keys (and 2 records since it's a leaf node)
            // Thus, split is at (3 / 2 = 1), and so we keep 1 key in this node, and the others in another node
            // if FANOUT is 4, there are 3 keys. Split is at (4 / 2 = 2). So we keep 2 keys in this node, and put
            // 1 keys in the new node. So there is split keys in the left half and n - split keys in the right half
            // we want the first 0

            let num_total_keys = FANOUT;
            let mut cur = 0;
            let mut old_read_idx = leaf.num_keys - 1;
            while cur < num_total_keys {
                let write_idx = num_total_keys - cur - 1;
                // insert in new leaf
                if write_idx >= split {
                    let new_write_idx = write_idx - split;
                    if write_idx == insertion_idx {
                        new_leaf.keys[new_write_idx] = Some(key.clone());
                        new_leaf.records[new_write_idx] =
                            Some(Rc::new(RefCell::new(record.clone())));
                    } else {
                        new_leaf.keys[new_write_idx] = leaf.keys[old_read_idx].clone();
                        new_leaf.records[new_write_idx] = leaf.records[old_read_idx].clone();
                        leaf.keys[old_read_idx] = None;
                        leaf.records[old_read_idx] = None;

                        // to prevent overflow
                        if old_read_idx > 0 {
                            old_read_idx -= 1;
                        }
                    }
                }
                // insert in old leaf
                else {
                    if write_idx == insertion_idx {
                        leaf.keys[write_idx] = Some(key.clone());
                        leaf.records[write_idx] = Some(Rc::new(RefCell::new(record.clone())));
                    } else {
                        leaf.keys.swap(write_idx, old_read_idx);

                        // to prevent overflow
                        if old_read_idx > 0 {
                            old_read_idx -= 1;
                        }
                    }
                }
                cur += 1
            }

            leaf.num_keys = split;
            new_leaf.num_keys = num_total_keys - split;
            new_leaf.parent = leaf.parent.clone();
            drop(leaf_borrow);
            //did not maintain the parent ptr
            self.insert_into_parent(
                leaf_node,
                &new_leaf.keys[0].clone().unwrap(),
                Rc::new(RefCell::new(Node::Leaf(new_leaf))),
            );
            return;
        }

        // 4 - the leaf has space, insert here
        let mut insertion_idx = 0;
        while insertion_idx < leaf.num_keys && leaf.keys[insertion_idx].as_ref().unwrap() < key {
            insertion_idx += 1
        }

        let mut i = leaf.num_keys;
        while i > insertion_idx {
            leaf.keys.swap(i, i - 1);
            leaf.records.swap(i, i - 1);
            i -= 1
        }
        leaf.keys[insertion_idx] = Some(key.clone());
        leaf.records[insertion_idx] = Some(Rc::new(RefCell::new(record.clone())));
        leaf.num_keys += 1;
    }

    fn insert_into_parent(
        &mut self,
        left: NodePtr<K, V, FANOUT>,
        key: &K,
        right: NodePtr<K, V, FANOUT>,
    ) {
        let mut left_borrow = left.borrow_mut();
        let parent = match &*left_borrow {
            Node::Leaf(leaf) => leaf.parent.as_ref(),
            Node::Invalid => panic!("Invalid node"),
            Node::Interior(node) => node.parent.as_ref(),
        };

        // 3 cases for insert into parent
        // 1 - No parent for left/right. We need to make a new root node if there is no parent
        if parent.is_none() {
            let mut new_node: Interior<K, V, FANOUT> = Node::new_interior();
            new_node.keys[0] = Some(key.clone());
            new_node.children[0] = Some(left.clone());
            new_node.children[1] = Some(right.clone());
            new_node.num_keys = 1;
            self.root = Some(Rc::new(RefCell::new(Node::Interior(new_node))));

            match &mut *left_borrow {
                Node::Invalid => panic!("Invalid Node"),
                Node::Leaf(leaf) => {
                    leaf.parent = Some(Rc::downgrade(&self.root.as_ref().unwrap()));
                    leaf.next = Some(Rc::downgrade(&right));
                }
                Node::Interior(node) => {
                    node.parent = Some(Rc::downgrade(&self.root.as_ref().unwrap()));
                }
            }
            drop(left_borrow);

            let mut right_borrow = right.borrow_mut();
            match &mut *right_borrow {
                Node::Invalid => panic!("Invalid Node"),
                Node::Leaf(leaf) => {   
                    leaf.parent = Some(Rc::downgrade(&self.root.as_ref().unwrap()));
                    leaf.prev = Some(Rc::downgrade(&left));
                }
                Node::Interior(node) => {
                    node.parent = Some(Rc::downgrade(&self.root.as_ref().unwrap()));
                }
            }
            drop(right_borrow);
            return;
        }
        // 2 - interior node has no space so we have to split it
        let parent= parent.unwrap().upgrade().unwrap();
        let mut parent_borrow = parent.borrow_mut();
        match &mut *parent_borrow {
            Node::Invalid => panic!("Invalid Node"),
            Node::Leaf(_) => {}
            Node::Interior(node) => {
                if node.num_keys == FANOUT - 1{
                    let mut insert_idx = node.num_keys;
                    while insert_idx > 0 && node.keys[insert_idx-1].as_ref().unwrap() > key{
                        insert_idx -= 1;
                    }
                    let mut i = node.num_keys;
                    while i>insert_idx{
                        node.keys.swap(i, i-1);
                        node.children.swap(i+1, i);
                        i -= 1;
                    }
                    node.keys[insert_idx] = Some(key.clone());
                    node.children[insert_idx] =Some(left.clone()) ;
                    node.children[insert_idx+1] = Some(right.clone());
                    node.num_keys += 1;
                    let split = FANOUT/2;
                    let mut new_left_node: Interior<K, V, FANOUT> = Node::new_interior();
                    let mut new_right_node: Interior<K, V, FANOUT>  = Node::new_interior();
                    let spilt_key = &node.keys[split].clone().unwrap();
                    i = split+1;//index of former node
                    let mut j = 0;//index of right_node
                    while i<FANOUT {
                        new_right_node.keys[j] = node.keys[i].clone();
                        new_right_node.children[j] = node.children[i].clone();

                        i += 1;
                        j += 1;
                    }
                    new_right_node.children[j] = node.children[i].clone();
                    new_right_node.num_keys = j;
                    new_right_node.parent = node.parent.clone();

                    j = 0;//index of left_node
                    while j<split {
                        new_left_node.keys[j] = node.keys[j].clone();
                        new_left_node.children[j] = node.children[j].clone();
                        j += 1;
                    }
                    new_left_node.children[j] = node.children[j].clone();
                    new_left_node.parent = node.parent.clone();
                    new_left_node.num_keys = j;
                    

                    let new_left: Rc<RefCell<Node<K, V, FANOUT>>> = Rc::new(RefCell::new(Node::Interior(new_left_node)));
                    let new_right = Rc::new(RefCell::new(Node::Interior(new_right_node)));
                    

                    //codes below have problems of borrow_mut

                    // {
                    //     let left_borrow_mut = new_left.borrow_mut();
                    //     let left_children_num = left_borrow_mut.interior().unwrap().num_keys;
                    //     for i in 0..left_children_num {
                    //         if let Some(child) = &left_borrow_mut.interior().unwrap().children[i] {
                    //             let mut child_mut = child.borrow_mut();
                    //             match child_mut.interior_mut().as_mut() { 
                    //                 Some(child)=>{child.parent = Some(Rc::downgrade(&new_left));}
                    //                 _=>()
                    //             }
                    //         }
                    //     }
                    // }

                    // {
                    //     let right_borrow_mut = new_right.borrow_mut();
                    //     let right_children_num = right_borrow_mut.interior().unwrap().num_keys;
                    //     for i in 0..right_children_num {
                    //         if let Some(child) = &right_borrow_mut.interior().unwrap().children[i] {
                    //             let mut child_mut = child.borrow_mut();
                    //             match child_mut.interior_mut().as_mut() { 
                    //                 Some(child)=>{child.parent = Some(Rc::downgrade(&new_right));}
                    //                 _=>()
                    //             }
                    //         }
                    //     }
                        
                    // }

                    self.insert_into_parent(new_left, spilt_key, new_right)
                }else {
                    let mut insert_idx = node.num_keys;
                    while insert_idx > 0 && node.keys[insert_idx-1].as_ref().unwrap() > key{
                        insert_idx -= 1;
                    }
                    let mut i = node.num_keys;
                    while i>insert_idx{
                        node.keys.swap(i, i-1);
                        node.children.swap(i+1, i);
                        i -= 1;
                    }
                    node.keys[insert_idx] = Some(key.clone());
                    node.children[insert_idx] =Some(left.clone()) ;
                    node.children[insert_idx+1] = Some(right.clone());
                    node.num_keys += 1;
                }
            }
        }
        // TODO: 3 - interior node has space so we just have to insert key / record
    }

    /* private */
    fn get_leaf_node(&self, key: &K) -> Option<NodePtr<K, V, FANOUT>> {
        let mut cur = self.root.clone()?;
        loop {
            let borrow = cur.borrow();
            if borrow.interior().is_none() {
                break;
            }

            let mut i = 0;
            let next = {
                let node = borrow.unwrap_interior();

                while i < node.num_keys && *key >= *node.keys[i].as_ref()? {
                    i += 1
                }
                node.children[i].as_ref()?.clone()
            };

            drop(borrow);
            cur = next;
        }
        Some(cur)
    }
}
