
use std::{
    cell::RefCell,
    rc::Rc
};

type Link = Option<Rc<RefCell<Node>>>;

struct Node {
    row_id: Option<usize>,
    name: Option<String>,
    size: usize,
    column: Link,
    left: Link,
    right: Link,
    up: Link,
    down: Link,
}


pub struct DancingLinks {
    root: Link
}


impl Node {

    pub fn new_node(row_id: usize) -> Rc<RefCell<Self>> {
        let node = Rc::new(RefCell::new(Node {
            row_id: Some(row_id),
            name: None,
            size: 0,
            column: None,
            left: None,
            right: None,
            up: None,
            down: None
        }));

        {
            let mut mut_node = node.borrow_mut();
            mut_node.left = Some(Rc::clone(&node));
            mut_node.right = Some(Rc::clone(&node));
            mut_node.up = Some(Rc::clone(&node));
            mut_node.down = Some(Rc::clone(&node));
        }

        node
    }

    pub fn new_column_node(name: String) -> Rc<RefCell<Self>> {
        let col_node = Rc::new(RefCell::new(Node {
            row_id: None,
            name: Some(name),
            size: 0,
            column: None,
            left: None,
            right: None,
            up: None,
            down: None
        }));

        {
            let mut _node = col_node.borrow_mut();
            _node.column = Some(Rc::clone(&col_node));
            _node.left = Some(Rc::clone(&col_node));
            _node.right = Some(Rc::clone(&col_node));
            _node.up = Some(Rc::clone(&col_node));
            _node.down = Some(Rc::clone(&col_node));
        }

        col_node
    }
}


impl DancingLinks {

    pub fn from_matrix(matrix: &Vec<Vec<usize>>) -> Self {
        let dlx = DancingLinks {
            root: Some(Node::new_column_node("root".to_string()))
        };
        let no_columns = matrix[0].len();
        let mut column_headers: Vec<Link> = Vec::with_capacity(no_columns);
        let root = dlx.root.as_ref().unwrap();
        
        // create column headers and link them
        for i in 0..no_columns {
            let column_node = Node::new_column_node(format!("{i}"));
            let previous_node = root.borrow().left.clone().unwrap();
            // create a new block so that column_node can be mutated and we can have a immutable reference to it outside the block.
            {
                let mut column = column_node.borrow_mut();
                column.right = Some(root.clone());
                column.left = Some(previous_node.clone());
                root.borrow_mut().left = Some(column_node.clone());
            }
            previous_node.borrow_mut().right = Some(column_node.clone());
            column_headers.push(Some(column_node));
        }

        // create nodes
        for (row_idx, row) in matrix.iter().enumerate() {
            let mut previous_node: Link = None;
            for (col_idx, &cell) in row.iter().enumerate() {
                if cell == 1 {
                    let column = column_headers[col_idx].clone().unwrap();
                    let node = Node::new_node(row_idx);
                    // =========== begining of code for linking node to column ===========
                    {
                        let mut mutable_node = node.borrow_mut();
                        // node.column = column
                        // node.down = column
                        // node.up = column.up
                        mutable_node.column = Some(column.clone());
                        mutable_node.down = Some(column.clone());
                        mutable_node.up = column.borrow().up.clone();
                    }

                    {
                        // update previous node before the newly created node
                        // column.up.down = node
                        let up_node = {
                            // need this trick because
                            // column.as_ref().borrow().up.as_ref().unwrap().borrow_mut().down = Some(node.clone())
                            // has a borrow and borrow_mutable.
                            // Use this block to separate the borrow from the mutable borrow.
                            let col_ref = column.borrow();
                            col_ref.up.clone().unwrap()
                        };
                        up_node.borrow_mut().down = Some(node.clone());
                    }

                    {
                        // column.up = node
                        // column.size += 1
                        let mut mutable_column = column.borrow_mut();
                        mutable_column.up = Some(node.clone());
                        mutable_column.size += 1;
                    }
                    // =========== end of code for linking node to column ===========

                    // ====== begining of code for linking node to left-right in row ======

                    if previous_node.is_none() {
                        previous_node = Some(node.clone());
                    }
                    let previous_right = previous_node.as_ref().unwrap().borrow().right.clone();

                    {
                        // node.left = prev
                        // node.right = prev.right
                        let mut _node = node.borrow_mut();
                        _node.left = previous_node.clone();
                        _node.right = previous_right.clone();
                    }

                    {
                        // prev.right.left = node
                        previous_right.unwrap().borrow_mut().left = Some(node.clone());
                    }

                    {
                        // prev.right = node
                        let mut mut_previous_node = previous_node.as_ref().unwrap().borrow_mut();
                        mut_previous_node.right = Some(node.clone());
                    }

                    previous_node = Some(node);
                    // ====== end of code for linking node to left-right in row ======            
                }
            }
        }
        
        dlx
    }

    pub fn cover(column: &Link) {
        // column.right.left = column.left
        // column.left.right = column.right
        // i = column.down
        // while i != column:
        //     j = i.right
        //     while j != i:
        //         j.down.up = j.up
        //         j.up.down = j.down
        //         j.column.size -= 1
        //         j = j.right
        //     i = i.down
        let borrowed_column = column.as_ref().unwrap().borrow();
        let mut column_right = borrowed_column.right.clone().unwrap();
        let mut column_left = borrowed_column.left.clone().unwrap();
        column_right.borrow_mut().left = borrowed_column.left.clone();
        column_left.borrow_mut().right = borrowed_column.right.clone();
        
        let mut current_node = borrowed_column.down.clone().unwrap();
        while !Rc::ptr_eq(&current_node,column.as_ref().unwrap()) {
            let mut current_cell = current_node.borrow().right.clone().unwrap();
            while !Rc::ptr_eq(&current_cell, &current_node) {
                
                let next_cell = {
                    let borrowed_current_cell = current_cell.borrow();
                    let cell_down = borrowed_current_cell.down.clone().unwrap();
                    let cell_up = borrowed_current_cell.up.clone().unwrap();
                    let cell_column = borrowed_current_cell.column.clone().unwrap();
                    cell_down.borrow_mut().up = Some(cell_up.clone());
                    cell_up.borrow_mut().down = Some(cell_down.clone());
                    cell_column.borrow_mut().size -= 1;
                    borrowed_column.right.clone().unwrap()
                };
                
                current_cell = next_cell;
            }
            current_node = {
                // put this in a block otherwise assignment is not possible
                // as current_node is borrowed.
                current_node.borrow().down.clone().unwrap()
            };
        }

    }
}


#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_new_node() {
        let node = Node::new_node(1);
        let borrowed_node = node.borrow();
        assert_eq!(borrowed_node.row_id, Some(1));
        assert!(borrowed_node.name.is_none());
        assert_eq!(borrowed_node.size, 0);
        assert!(Rc::ptr_eq(
            &node,
            borrowed_node.left.as_ref().unwrap()
        ));
        assert!(Rc::ptr_eq(
            &node,
            borrowed_node.right.as_ref().unwrap()
        ));
        assert!(Rc::ptr_eq(
            &node,
            borrowed_node.up.as_ref().unwrap()
        ));
        assert!(Rc::ptr_eq(
            &node,
            borrowed_node.down.as_ref().unwrap()
        ));
        assert!(borrowed_node.column.is_none());
    }

    #[test]
    fn test_new_col_node() {
        let node = Node::new_column_node("a".to_string());
        let borrowed_node = node.borrow();
        assert_eq!(borrowed_node.name, Some("a".to_string()));
        assert!(borrowed_node.row_id.is_none());
        assert_eq!(borrowed_node.size, 0);
        assert!(Rc::ptr_eq(
            &node,
            borrowed_node.left.as_ref().unwrap()
        ));
        assert!(Rc::ptr_eq(
            &node,
            borrowed_node.right.as_ref().unwrap()
        ));
        assert!(Rc::ptr_eq(
            &node,
            borrowed_node.up.as_ref().unwrap()
        ));
        assert!(Rc::ptr_eq(
            &node,
            borrowed_node.down.as_ref().unwrap()
        ));
        assert!(Rc::ptr_eq(
            &node,
            borrowed_node.column.as_ref().unwrap()
        ));
    }

    #[test]
    fn test_from_matrix() {
        //   C0 C1 C2
        // R0 1  0  0
        // R1 1  1  0
        // R2 0  0  1
        let matrix: Vec<Vec<usize>> = vec![
            vec![1, 0, 0],
            vec![1, 1, 0],
            vec![0, 0, 1],
        ];
        let dlx = DancingLinks::from_matrix(&matrix);
        let root = dlx.root.as_ref().unwrap();
        let borrowed_root = root.borrow();
        let col0 = borrowed_root.right.as_ref().unwrap();
        let borrowed_col0 = col0.borrow();
        let col1 = borrowed_col0.right.as_ref().unwrap();
        let borrowed_col1 = col1.borrow();
        let col2 = borrowed_col1.right.as_ref().unwrap();
        let borrowed_col2 = col2.borrow();

        assert_eq!(borrowed_root.name.as_deref(),  Some("root"));
        assert_eq!(borrowed_col0.name.as_deref(),  Some("0"));
        assert_eq!(borrowed_col1.name.as_deref(),  Some("1"));
        assert_eq!(borrowed_col2.name.as_deref(),  Some("2"));

        assert!(Rc::ptr_eq(root,  borrowed_col2.right.as_ref().unwrap()));
        assert!(Rc::ptr_eq(root,  borrowed_col0.left.as_ref().unwrap()));
        assert!(Rc::ptr_eq(col0,  borrowed_col1.left.as_ref().unwrap()));
        assert!(Rc::ptr_eq(col1,  borrowed_col2.left.as_ref().unwrap()));
        assert!(Rc::ptr_eq(col2,  borrowed_root.left.as_ref().unwrap()));

        // ---------- column 0 : rows 0 & 1 ---------------------------------------
        let c0_r0 = borrowed_col0.down.as_ref().unwrap();          // row-0 node
        let borrowed_c0_r0 = c0_r0.borrow();
        let c0_r1 = borrowed_c0_r0.down.as_ref().unwrap();         // row-1 node
        let borrowed_c0_r1 = c0_r1.borrow();

        assert_eq!(borrowed_c0_r0.row_id, Some(0));
        assert_eq!(borrowed_c0_r1.row_id, Some(1));

        // circular:
        assert!(Rc::ptr_eq(borrowed_c0_r1.down.as_ref().unwrap(), col0));
        assert!(Rc::ptr_eq(borrowed_col0.up.as_ref().unwrap(),    c0_r1));
        // -------------------------------------------------------------------------

        // ---------- column 1 : only row 1 ----------------------------------------
        let c1_r1 = borrowed_col1.down.as_ref().unwrap();
        let borrowed_c1_r1 = c1_r1.borrow();
        assert_eq!(borrowed_c1_r1.row_id, Some(1));
        assert!(Rc::ptr_eq(borrowed_c1_r1.down.as_ref().unwrap(), col1));
        // -------------------------------------------------------------------------

        // ---------- column 2 : only row 2 ----------------------------------------
        let c2_r2 = borrowed_col2.down.as_ref().unwrap();
        let borrowed_c2_r2 = c2_r2.borrow();
        assert_eq!(c2_r2.borrow().row_id, Some(2));
        assert!(Rc::ptr_eq(borrowed_c2_r2.down.as_ref().unwrap(), col2));
        // -------------------------------------------------------------------------

        // ---------- row 0 : single node looping to itself ------------------------
        assert!(Rc::ptr_eq(
            borrowed_c0_r0.right.as_ref().unwrap(),
            c0_r0
        ));
        assert!(Rc::ptr_eq(
            borrowed_c0_r0.left.as_ref().unwrap(),
            c0_r0
        ));
        // -------------------------------------------------------------------------

        // ---------- row 1 : two nodes, C0-R1  <-->  C1-R1 ------------------------
        let r1_n0 = c0_r1;
        let borrowed_r1_n0 = r1_n0.borrow();                           // node in column-0
        let r1_n1 = borrowed_r1_n0.right.as_ref().unwrap(); // node in column-1
        let borrowed_r1_n1 = r1_n1.borrow();

        // neighbours
        assert!(Rc::ptr_eq(borrowed_r1_n1.right.as_ref().unwrap(), r1_n0));
        assert!(Rc::ptr_eq(borrowed_r1_n1.left.as_ref().unwrap(),  r1_n0));
        assert!(Rc::ptr_eq(borrowed_r1_n0.left.as_ref().unwrap(),  r1_n1));
        assert!(Rc::ptr_eq(borrowed_r1_n0.right.as_ref().unwrap(), r1_n1));

        // the second node really is in column-1
        assert!(Rc::ptr_eq(
            borrowed_r1_n1.column.as_ref().unwrap(),
            col1
        ));
        // -------------------------------------------------------------------------

        // ---------- row 2 : single node (column-2) looping to itself -------------
        assert!(Rc::ptr_eq(
            borrowed_c2_r2.right.as_ref().unwrap(),
            c2_r2
        ));
        assert!(Rc::ptr_eq(
            borrowed_c2_r2.left.as_ref().unwrap(),
            c2_r2
        ));
        // -------------------------------------------------------------------------
    }

    #[test]
    fn test_cover_columns() {
        //   C0 C1 C2
        // R0 1  0  0
        // R1 1  1  0
        // R2 0  0  1
        let matrix: Vec<Vec<usize>> = vec![
            vec![1, 0, 0],
            vec![1, 1, 0],
            vec![0, 0, 1],
        ];

        // ────────────────────────────────────────────────────────────────────────
        // build DLX structure
        let dlx  = DancingLinks::from_matrix(&matrix);
        let root = dlx.root.as_ref().unwrap();

        // grab the three column headers (Rc clones so we can keep them)
        let col0 = root.borrow().right.as_ref().unwrap().clone();
        let col1 = col0.borrow().right.as_ref().unwrap().clone();
        let col2 = col1.borrow().right.as_ref().unwrap().clone();
        // ────────────────────────────────────────────────────────────────────────

        // ──────────────── 1️⃣ cover column-0 ────────────────────────────────────
        DancingLinks::cover(&Some(col0.clone()));

        // header ring: root ↔ col1 ↔ col2 ↔ root
        assert!(Rc::ptr_eq(root.borrow().right.as_ref().unwrap(), &col1));
        assert!(Rc::ptr_eq(root.borrow().left.as_ref().unwrap(),  &col2));
        assert!(Rc::ptr_eq(col1.borrow().left.as_ref().unwrap(),   root));
        assert!(Rc::ptr_eq(col1.borrow().right.as_ref().unwrap(), &col2));
        assert!(Rc::ptr_eq(col2.borrow().left.as_ref().unwrap(),  &col1));
        assert!(Rc::ptr_eq(col2.borrow().right.as_ref().unwrap(),  root));

        // column-1 is now empty
        {
            let c1 = col1.borrow();
            assert_eq!(c1.size, 0);
            assert!(Rc::ptr_eq(c1.down.as_ref().unwrap(), &col1));
            assert!(Rc::ptr_eq(c1.up  .as_ref().unwrap(), &col1));
        }

        // column-2 still has its row-2 node
        {
            let c2 = col2.borrow();
            assert_eq!(c2.size, 1);
            let r2 = c2.down.as_ref().unwrap();
            assert_eq!(r2.borrow().row_id, Some(2));
            // row-2 is a single node that loops to itself left/right
            assert!(Rc::ptr_eq(r2.borrow().left.as_ref().unwrap(),  r2));
            assert!(Rc::ptr_eq(r2.borrow().right.as_ref().unwrap(), r2));
        }
        // ────────────────────────────────────────────────────────────────────────

        // ──────────────── 2️⃣ cover column-2 ────────────────────────────────────
        DancingLinks::cover(&Some(col2.clone()));

        // header ring: root ↔ col1 ↔ root   (col2 removed)
        assert!(Rc::ptr_eq(root.borrow().right.as_ref().unwrap(), &col1));
        assert!(Rc::ptr_eq(root.borrow().left .as_ref().unwrap(), &col1));
        assert!(Rc::ptr_eq(col1.borrow().left .as_ref().unwrap(),  root));
        assert!(Rc::ptr_eq(col1.borrow().right.as_ref().unwrap(),  root));

        // column-2 is now empty
        {
            let c2 = col2.borrow();
            assert_eq!(c2.size, 0);
            assert!(Rc::ptr_eq(c2.down.as_ref().unwrap(), &col2));
            assert!(Rc::ptr_eq(c2.up  .as_ref().unwrap(), &col2));
        }
    }
}
