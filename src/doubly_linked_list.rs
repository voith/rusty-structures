use std::{
    cell::RefCell,
    fmt::Display, 
    io::{stdout, Write},
    rc::Rc
};

type Link<T> = Option<Rc<RefCell<Node<T>>>>; 

struct Node<T> {
    value: T,
    previous: Link<T>,
    next: Link<T>
}

pub struct DoublyLinkedList<T> {
    head: Link<T>
}

impl <T> Node<T> {
    fn new(value: T, previous: Link<T>) -> Self {
        Node {
            value,
            previous,
            next: None
        }
    }
}

impl <T: PartialEq + Display + Clone> DoublyLinkedList<T> {
    pub fn new() -> Self {
        DoublyLinkedList {
            head: None
        }
    }

    pub fn add(&mut self, value: T) {
        if self.head.is_none() {
            self.head = Some(Rc::new(RefCell::new(Node::new(value, None))));
            return;
        }

        let mut current_node = self.head.clone().unwrap();
        while let Some(next_node) = {
            // Use a block to limit the borrow scope
            let next = current_node.borrow().next.clone();
            next
        } {
            current_node = next_node;
        }
        current_node.borrow_mut().next = Some(Rc::new(RefCell::new(Node::new(value, Some(Rc::clone(&current_node))))));
    }

    pub fn remove(&mut self, value: T) {
        if let Some(head_node) = self.head.clone() {
            if head_node.borrow().value == value {
                self.head = head_node.borrow().next.clone();
                if let Some(node) = &self.head {
                    node.borrow_mut().previous = None;
                }
                return;
            }
        }
        let mut current_node = self.head.clone();
        while let Some(node) = current_node {
            // Use a block to limit the borrow scope and avoid BorrowMutError
            if let Some(next_node) = {
                let _node = node.borrow().next.clone();
                _node
            } {
                if next_node.borrow().value == value {
                    match next_node.borrow().next.clone() {
                        Some(_next_node) => {
                            _next_node.borrow_mut().previous = Some(node.clone());
                            node.borrow_mut().next = Some(_next_node);
                        }
                        None => {
                            node.borrow_mut().next = None;
                        }
                    }
                    return;
                }
            }
            current_node = node.borrow().next.clone();
        }
    }

    pub fn print_forward(&self) {
        self.print_to_writer(&mut stdout());
    }

    pub fn print_to_writer<W: Write>(&self, writer: &mut W) {
        let mut current_node = self.head.clone();
        while let Some(node) = current_node {
            writeln!(writer, "{}", node.borrow().value).unwrap();
            current_node = node.borrow().next.clone();
        }
    }
}


#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::*;

    fn  check_output<T: PartialEq + Display + Clone>(list: &DoublyLinkedList<T>, expected_str: &str) {
        let mut buffer = Cursor::new(Vec::new());
        list.print_to_writer(&mut buffer);

        let output = String::from_utf8(buffer.into_inner()).unwrap();
        assert_eq!(output, expected_str);
    }

    #[test]
    fn test_add_elements() {
        let mut list = DoublyLinkedList::new();
        list.add(1);
        list.add(2);
        list.add(3);
        check_output(&list, "1\n2\n3\n");

        let first_element = list.head.as_ref().unwrap().clone();
        let second_element = first_element.borrow().next.as_ref().unwrap().clone();
        let third_element = second_element.borrow().next.as_ref().unwrap().clone();
        
        assert!(first_element.borrow().previous.is_none());
        assert!(Rc::ptr_eq(
            first_element.borrow().next.as_ref().unwrap(), 
            &second_element
        ));
        assert!(Rc::ptr_eq(
            &first_element, 
            second_element.borrow().previous.as_ref().unwrap()
        ));
        assert!(Rc::ptr_eq(
            second_element.borrow().next.as_ref().unwrap(), 
            &third_element
        ));
        assert!(Rc::ptr_eq(
            &second_element, 
            &third_element.borrow().previous.as_ref().unwrap()
        ));
        assert!(third_element.borrow().next.is_none());

        assert_eq!(first_element.borrow().value, 1);
        assert_eq!(second_element.borrow().value, 2);
        assert_eq!(third_element.borrow().value, 3);
    }

    #[test]
    fn test_remove_elements() {
        let mut list = DoublyLinkedList::new();
        list.add(1);
        list.add(2);
        list.add(3);
        list.remove(2);
        check_output(&list, "1\n3\n");
        
        let first_element = list.head.as_ref().unwrap().clone();
        let second_element = first_element.borrow().next.as_ref().unwrap().clone();
        
        assert!(first_element.borrow().previous.is_none());
        assert!(Rc::ptr_eq(
            first_element.borrow().next.as_ref().unwrap(), 
            &second_element
        ));
        assert!(Rc::ptr_eq(
            &first_element, 
            second_element.borrow().previous.as_ref().unwrap()
        ));
        assert!(second_element.borrow().next.is_none());

        assert_eq!(first_element.borrow().value, 1);
        assert_eq!(second_element.borrow().value, 3);
    }

    #[test]
    fn test_remove_root() {
        let mut list = DoublyLinkedList::new();
        list.add(1);
        list.add(2);
        list.add(3);
        list.remove(1);
        check_output(&list, "2\n3\n");
        
        let first_element = list.head.as_ref().unwrap().clone();
        let second_element = first_element.borrow().next.as_ref().unwrap().clone();

        assert!(first_element.borrow().previous.is_none());
        assert!(Rc::ptr_eq(
            first_element.borrow().next.as_ref().unwrap(), 
            &second_element
        ));
        assert!(Rc::ptr_eq(
            &first_element, 
            second_element.borrow().previous.as_ref().unwrap()
        ));

        assert_eq!(first_element.borrow().value, 2);
        assert_eq!(second_element.borrow().value, 3);
    }

    #[test]
    fn test_remove_root_with_one_element() {
        let mut list = DoublyLinkedList::new();
        list.add(1);
        list.remove(1);
        check_output(&list, "");
        assert!(list.head.is_none());
    }

    #[test]
    fn test_remove_last_element() {
        let mut list = DoublyLinkedList::new();
        list.add(1);
        list.add(2);
        list.add(3);
        list.remove(3);
        check_output(&list, "1\n2\n");
        
        let first_element = list.head.as_ref().unwrap().clone();
        let second_element = first_element.borrow().next.as_ref().unwrap().clone();
        
        assert!(first_element.borrow().previous.is_none());
        assert!(Rc::ptr_eq(
            first_element.borrow().next.as_ref().unwrap(), 
            &second_element
        ));
        assert!(Rc::ptr_eq(
            &first_element, 
            second_element.borrow().previous.as_ref().unwrap()
        ));
        assert!(second_element.borrow().next.is_none());

        assert_eq!(first_element.borrow().value, 1);
        assert_eq!(second_element.borrow().value, 2);
    }

    #[test]
    fn test_add_remove_add() {
        let mut list = DoublyLinkedList::new();
        list.add(1);
        list.add(2);
        list.remove(2);
        list.add(3);
        check_output(&list, "1\n3\n");

        let first_element = list.head.as_ref().unwrap().clone();
        let second_element = first_element.borrow().next.as_ref().unwrap().clone();
        
        assert!(first_element.borrow().previous.is_none());
        assert!(Rc::ptr_eq(
            first_element.borrow().next.as_ref().unwrap(), 
            &second_element
        ));
        assert!(Rc::ptr_eq(
            &first_element, 
            second_element.borrow().previous.as_ref().unwrap()
        ));
        assert!(second_element.borrow().next.is_none());

        assert_eq!(first_element.borrow().value, 1);
        assert_eq!(second_element.borrow().value, 3);
    }

    #[test]
    fn test_remove_same_element() {
        let mut list = DoublyLinkedList::new();
        list.add(1);
        list.add(2);
        list.add(2);
        list.add(3);
        list.remove(2);
        check_output(&list, "1\n2\n3\n");
        list.remove(2);
        check_output(&list, "1\n3\n");
    }

    #[test]
    fn test_remove_non_existent() {
        let mut list = DoublyLinkedList::new();
        list.add(1);
        list.add(2);
        list.add(3);
        list.remove(4);
        check_output(&list, "1\n2\n3\n");
    }

    #[test]
    fn test_remove_non_existent_from_empty() {
        let mut list = DoublyLinkedList::new();
        list.remove(4);
        check_output(&list, "");
    }
}
