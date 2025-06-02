use std::fmt::Display;
use std::io::{self, Write};

struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>
}

pub struct LinkedList<T> {
    root: Option<Box<Node<T>>>
}

impl <T> Node<T> {
    fn new(value: T) -> Self {
        Node {
            value,
            next: None
        }
    }
}

impl <T: PartialEq + Display + Clone> LinkedList<T> {
    pub fn new() -> Self {
        LinkedList {
            root: None
        }
    }

    pub fn add(&mut self, value: T) {
        let new_node = Some(Box::new(Node::new(value)));
        if self.root.is_none() {
            self.root = new_node;
            return;
        }
        let mut current_node = self.root.as_mut().unwrap();
        while let Some(ref mut next_node) = current_node.next  {
            current_node = next_node;
        }
        current_node.next = new_node;
    }

    pub fn remove(&mut self, value: T) {
        if let Some(root_node) = self.root.as_mut() {
            if root_node.value == value {
                self.root = root_node.next.take();
                return;
            }
        }

        let mut current_node = self.root.as_mut();
        while let Some(node) = current_node {
            if let Some(ref mut next_node) = node.next {
                if next_node.value == value {
                    node.next = next_node.next.take();
                    return;
                }
            }
            current_node = node.next.as_mut();
        }
    }

    pub fn print(&self) {
        self.print_to_writer(&mut io::stdout());
    }

    pub fn print_to_writer<W: Write>(&self, writer: &mut W) {
        let mut current_node = self.root.as_ref();
        while let Some(node) = current_node {
            writeln!(writer, "{}", node.value).unwrap();
            current_node = node.next.as_ref();
        }
    }

    pub fn to_vector(&self) -> Vec<T> {
        let mut v: Vec<T> = Vec::new();
        let mut current_node = self.root.as_ref();
        while let Some(node) = current_node {
            v.push(node.value.clone());
            current_node = node.next.as_ref();
        }
        v
    }
}


#[cfg(test)]
mod test {
    use super::*; 
    use std::io::Cursor;

    #[test]
    fn test_add() {
        let mut linked_list = LinkedList::new();
        linked_list.add(1);
        linked_list.add(2);
        linked_list.add(3);
        let v = linked_list.to_vector();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_remove() {
        let mut linked_list = LinkedList::new();
        linked_list.add(1);
        linked_list.add(2);
        linked_list.add(3);
        linked_list.remove(2);
        let v = linked_list.to_vector();
        assert_eq!(v, vec![1, 3]);
    }
    
    #[test]
    fn test_empty_list() {
        let linked_list: LinkedList<i32>  = LinkedList::new();
        let v = linked_list.to_vector();
        assert_eq!(v, vec![]);
    }

    #[test]
    fn test_remove_all_elements() {
        let mut linked_list = LinkedList::new();
        linked_list.add(1);
        linked_list.add(2);
        linked_list.add(3);
        linked_list.remove(2);
        linked_list.remove(1);
        linked_list.remove(3);
        let v= linked_list.to_vector();
        assert_eq!(v, vec![]);
    }

    #[test]
    fn test_remove_root() {
        let mut linked_list = LinkedList::new();
        linked_list.add(1);
        linked_list.add(2);
        linked_list.add(3);
        linked_list.remove(1);
        let v = linked_list.to_vector();
        assert_eq!(v, vec![2,3]);
    }

    #[test]
    fn test_remove_last_node() {
        let mut linked_list = LinkedList::new();
        linked_list.add(1);
        linked_list.add(2);
        linked_list.add(3);
        linked_list.remove(3);
        let v = linked_list.to_vector();
        assert_eq!(v, vec![1, 2]);
    }

    #[test]
    fn test_remove_non_existent() {
        let mut linked_list = LinkedList::new();
        linked_list.add(1);
        linked_list.add(2);
        linked_list.add(3);
        linked_list.remove(42);
        let v = linked_list.to_vector();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_remove_from_empty_list() {
        let mut linked_list: LinkedList<i32> = LinkedList::new();
        linked_list.remove(42);
        let v = linked_list.to_vector();
        assert_eq!(v, vec![]);
    }

    #[test]
    fn test_print() {
        let mut linked_list = LinkedList::new();
        linked_list.add(1);
        linked_list.add(2);
        linked_list.add(3);

        let mut buffer = Cursor::new(Vec::new());
        linked_list.print_to_writer(&mut buffer);

        let output = String::from_utf8(buffer.into_inner()).unwrap();
        assert_eq!(output, "1\n2\n3\n");
    }
}
