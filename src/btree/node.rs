use std::fmt::Debug;

use std::cell:: RefCell;
use std::rc::{Rc, Weak};
pub trait Key: PartialEq + PartialOrd + Clone + Debug {}

pub trait Record: Clone + Debug {}

pub type NodePtr<K, T, const FANOUT: usize> = Rc<RefCell<Node<K, T, FANOUT>>>;

pub type NodeWeakPtr<K, T, const FANOUT: usize> = Weak<RefCell<Node<K, T, FANOUT>>>;

pub type RecordPtr<T> = Rc<RefCell<T>>;

#[derive(Debug)]
pub struct Leaf<K: Key, V: Record, const FANOUT: usize> {
    pub num_keys: usize,
    pub keys: Vec<Option<K>>,
    pub records: Vec<Option<RecordPtr<V>>>,
    pub parent: Option<NodeWeakPtr<K, V, FANOUT>>,
    pub prev: Option<NodeWeakPtr<K, V, FANOUT>>,
    pub next: Option<NodeWeakPtr<K, V, FANOUT>>,
}

#[derive(Debug)]
pub struct Interior<K: Key, V: Record, const FANOUT: usize> {
    pub num_keys: usize,
    pub keys: Vec<Option<K>>,
    pub children: Vec<Option<NodePtr<K, V, FANOUT>>>,
    pub parent: Option<NodeWeakPtr<K, V, FANOUT>>,
}

#[derive(Debug, Default)]
pub enum Node<K: Key, V: Record, const FANOUT: usize> {
    #[default]
    Invalid,
    Leaf(Leaf<K, V, FANOUT>),
    Interior(Interior<K, V, FANOUT>),
}

impl<K: Key, V: Record, const FANOUT: usize> Node<K, V, FANOUT> {
    pub fn new_leaf() -> Leaf<K, V, FANOUT> {
        Leaf {
            num_keys: 0,
            keys: vec![None; FANOUT - 1],
            records: vec![None; FANOUT - 1],
            parent: None,
            prev: None,
            next: None,
        }
    }
    pub fn new_interior() -> Interior<K, V, FANOUT> {
        Interior {
            num_keys: 0,
            keys: vec![None; FANOUT],//there is only FANOUT - 1 keys,but for convenience ,make it FANOUT
            children: vec![None; FANOUT+1],//there is only FANOUT children,but for convenience ,make it FANOUT
            parent: None,
        }
    }

    pub(super) fn leaf(&self) -> Option<&Leaf<K, V, FANOUT>> {
        if let Node::Invalid = self {
            panic!("Invalid Node encountered while accessing leaf!")
        }

        if let Node::Leaf(leaf) = self {
            Some(leaf)
        } else {
            None
        }
    }

    pub(super) fn leaf_mut(&mut self) -> Option<&mut Leaf<K, V, FANOUT>> {
        if let Node::Invalid = self {
            panic!("Invalid Node encountered while accessing leaf!")
        }

        if let Node::Leaf(leaf) = self {
            Some(leaf)
        } else {
            None
        }
    }

    pub(super) fn unwrap_leaf(&self) -> &Leaf<K, V, FANOUT> {
        self.leaf().unwrap()
    }

    pub(super) fn unwrap_leaf_mut(&mut self) -> &mut Leaf<K, V, FANOUT> {
        self.leaf_mut().unwrap()
    }

    pub(super) fn interior(&self) -> Option<&Interior<K, V, FANOUT>> {
        if let Node::Invalid = self {
            panic!("Invalid Node encountered while accessing interior!")
        }

        if let Node::Interior(interior) = self {
            Some(interior)
        } else {
            None
        }
    }
    pub(super) fn interior_mut(&mut self) -> Option<&mut Interior<K, V, FANOUT>> {
        if let Node::Invalid = self {
            panic!("Invalid Node encountered while accessing interior!")
        }

        if let Node::Interior(interior) = self {
            Some(interior)
        } else {
            None
        }
    }
    pub(super) fn unwrap_interior(&self) -> &Interior<K, V, FANOUT> {
        self.interior().unwrap()
    }

}
// impl<K: Key, V: Record, const FANOUT: usize> Interior<K, V, FANOUT> {
//     pub fn set_children_parent(&mut self)
//     {
//         let parent_ptr = Rc::new(RefCell::new(Node::Interior(&self)));
//         for child_opt in &mut self.children {
//             if let Some(child_rc) = child_opt.clone() {
//                 let mut child_mut = (*child_rc).borrow_mut();
//                 match &mut *child_mut {
//                     Node::Invalid=>panic!("wrong!"),
//                     Node::Leaf(leaf_mut)=> {
                        
//                         leaf_mut.parent = Some(Rc::downgrade(&parent_ptr));
//                     }
//                     Node::Interior(interior_mut) =>{

//                         interior_mut.parent = Some(Rc::downgrade(&parent_ptr));
//                     }
//                 }
//             }
//         }
//     }
// }