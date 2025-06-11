use std::{cell::RefCell, rc::Rc};

type Link = Option<Rc<RefCell<Node>>>;

pub struct Node {
    row_id: Option<usize>,
    size: usize,
    column: Link,
    left: Link,
    right: Link,
    up: Link,
    down: Link,
}

pub struct DancingLinks {
    root: Link,
}

impl Node {
    pub fn new_node(row_id: usize) -> Rc<RefCell<Self>> {
        let node = Rc::new(RefCell::new(Node {
            row_id: Some(row_id),
            size: 0,
            column: None,
            left: None,
            right: None,
            up: None,
            down: None,
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

    pub fn new_column_node() -> Rc<RefCell<Self>> {
        let col_node = Rc::new(RefCell::new(Node {
            row_id: None,
            size: 0,
            column: None,
            left: None,
            right: None,
            up: None,
            down: None,
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
            root: Some(Node::new_column_node()),
        };
        let no_columns = matrix[0].len();
        let mut column_headers: Vec<Link> = Vec::with_capacity(no_columns);
        let root = dlx.root.as_ref().unwrap();

        // create column headers and link them
        for _ in 0..no_columns {
            let column_node = Node::new_column_node();
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

    fn cover(column: &Link) {
        let column_rc = column.as_ref().unwrap().clone();
        {
            let (left, right) = {
                let col = column_rc.borrow();
                (col.left.clone().unwrap(), col.right.clone().unwrap())
            };
            right.borrow_mut().left = Some(left.clone());
            left.borrow_mut().right = Some(right);
        }

        let mut current_node = column_rc.borrow().down.clone().unwrap();
        while !Rc::ptr_eq(&current_node, &column_rc) {
            let mut current_cell = current_node.borrow().right.clone().unwrap();
            while !Rc::ptr_eq(&current_cell, &current_node) {
                let (up, down, col, next) = {
                    let cell = current_cell.borrow();
                    (
                        cell.up.clone().unwrap(),
                        cell.down.clone().unwrap(),
                        cell.column.clone().unwrap(),
                        cell.right.clone().unwrap(),
                    )
                };
                down.borrow_mut().up = Some(up.clone());
                up.borrow_mut().down = Some(down.clone());
                col.borrow_mut().size -= 1;
                current_cell = next;
            }
            current_node = {
                // put this in a block otherwise assignment is not possible
                // as current_node is borrowed.
                current_node.borrow().down.clone().unwrap()
            };
        }
    }

    fn uncover(column: &Link) {
        let column_rc = column.as_ref().unwrap().clone();

        let mut current_node = column_rc.borrow().up.clone().unwrap();
        while !Rc::ptr_eq(&current_node, &column_rc) {
            let mut current_cell = current_node.borrow().left.clone().unwrap();
            while !Rc::ptr_eq(&current_cell, &current_node) {
                let (up, down, col, next) = {
                    let cell = current_cell.borrow();
                    (
                        cell.up.clone().unwrap(),
                        cell.down.clone().unwrap(),
                        cell.column.clone().unwrap(),
                        cell.left.clone().unwrap(),
                    )
                };
                col.borrow_mut().size += 1;
                down.borrow_mut().up = Some(current_cell.clone());
                up.borrow_mut().down = Some(current_cell);
                current_cell = next;
            }
            current_node = { current_node.borrow().up.clone().unwrap() };
        }

        {
            let (left, right) = {
                let col = column_rc.borrow();
                (col.left.clone().unwrap(), col.right.clone().unwrap())
            };
            right.borrow_mut().left = Some(column_rc.clone());
            left.borrow_mut().right = Some(column_rc);
        }
    }

    pub fn search(
        &self,
        solution: &mut Vec<Link>,      // current partial solution (stack)
        results: &mut Vec<Vec<usize>>, // all complete solutions
    ) {
        let root = self.root.as_ref().unwrap();

        /* ---------- base case: all columns covered ---------- */
        if Rc::ptr_eq(root.borrow().right.as_ref().unwrap(), root) {
            let row_ids: Vec<usize> = solution
                .iter()
                .map(|n| n.as_ref().unwrap().borrow().row_id.unwrap())
                .collect();
            results.push(row_ids);
            return;
        }

        /* ---------- choose the column with the fewest nodes ---------- */
        let column = {
            // start with first column to the right of root
            let mut best = root.borrow().right.as_ref().unwrap().clone();
            let mut c = best.borrow().right.clone().unwrap();
            while !Rc::ptr_eq(&c, root) {
                if c.borrow().size < best.borrow().size {
                    best = c.clone();
                }
                c = { c.borrow().right.clone().unwrap() };
            }
            best
        };

        Self::cover(&Some(column.clone()));

        /* ---------- iterate over each row in that column ---------- */
        let mut row = column.borrow().down.clone().unwrap();
        while !Rc::ptr_eq(&row, &column) {
            // push row onto partial solution
            solution.push(Some(row.clone()));

            /* cover every other column that has a 1 in this row */
            {
                let mut j = row.borrow().right.clone().unwrap();
                while !Rc::ptr_eq(&j, &row) {
                    let next = {
                        let cell_ref = j.borrow();
                        let col_hdr = cell_ref.column.clone().unwrap();
                        Self::cover(&Some(col_hdr));
                        cell_ref.right.clone().unwrap()
                    };
                    j = next;
                }
            }

            // recursive descent
            self.search(solution, results);

            /* ---------- back-track ---------- */
            let r = solution.pop().unwrap().unwrap(); // the row we just explored

            // Restore columns in reverse order
            let mut j = r.borrow().left.clone().unwrap();
            while !Rc::ptr_eq(&j, &r) {
                let next = {
                    let cell_ref = j.borrow();
                    let col_hdr = cell_ref.column.clone().unwrap();
                    Self::uncover(&Some(col_hdr));
                    cell_ref.left.clone().unwrap()
                };
                j = next;
            }

            // advance to the next row *in the same column*
            row = r.borrow().down.clone().unwrap();
        }

        // finally uncover the column we originally chose
        Self::uncover(&Some(column));
    }
}

#[allow(dead_code)]
struct SudokuSolver {
    matrix: Vec<[u8; Self::NO_COLUMNS]>,
    row_map: Vec<(u8, u8, u8)>,
}

impl SudokuSolver {
    const GRID_SIZE: usize = 9;
    const NO_COLUMNS: usize = 4 * Self::GRID_SIZE * Self::GRID_SIZE;
    const NO_ROWS: usize = Self::GRID_SIZE * Self::GRID_SIZE * Self::GRID_SIZE;
    const BOX_ROW_SIZE: usize = 3;

    #[allow(dead_code)]
    fn build_row(row_no: usize, col_no: usize, digit: usize) -> [u8; 324] {
        let grid_size = Self::GRID_SIZE;
        let mut row: [u8; Self::NO_COLUMNS] = [0; Self::NO_COLUMNS];

        // set row column constraint eg. R1C1, R1C2, etc
        row[grid_size * row_no + col_no] = 1;
        // set row constraint eg. R1#1, R1#2, etc
        row[grid_size.pow(2) + grid_size * row_no + (digit - 1)] = 1;
        // set column constraint eg. C1#1, C1#2, etc
        row[2 * grid_size.pow(2) + grid_size * col_no + (digit - 1)] = 1;
        // set box constraint eg. B1#1, B1#1, etc
        let _box_no = (row_no / 3) * 3 + (col_no / 3);
        row[3 * grid_size.pow(2) + grid_size * _box_no + (digit - 1)] = 1;

        row
    }

    #[allow(dead_code)]
    fn from_grid_to_exact_cover(sudoku_grid: &Vec<Vec<u8>>) -> Self {
        let mut matrix: Vec<[u8; Self::NO_COLUMNS]> = Vec::with_capacity(Self::NO_ROWS);
        let mut row_map: Vec<(u8, u8, u8)> = Vec::with_capacity(Self::NO_ROWS);
        for row_no in 0..9 {
            for col_no in 0..9 {
                if sudoku_grid[row_no][col_no] != 0 {
                    let digit = sudoku_grid[row_no][col_no] as usize;
                    matrix.push(Self::build_row(row_no, col_no, digit));
                    row_map.push((row_no as u8, col_no as u8, digit as u8));
                } else {
                    for d in 1..10 {
                        let digit = d as usize;
                        matrix.push(Self::build_row(row_no, col_no, digit));
                        row_map.push((row_no as u8, col_no as u8, digit as u8));
                    }
                }
            }
        }
        SudokuSolver { matrix, row_map }
    }

    #[allow(dead_code)]
    fn solve(&self) -> Option<Vec<[u8; Self::GRID_SIZE]>> {
        let vec_matrix: Vec<Vec<usize>> = self
            .matrix
            .iter()
            .map(|row| row.iter().map(|&v| v as usize).collect())
            .collect();
        let dlx = DancingLinks::from_matrix(&vec_matrix);
        let mut solution: Vec<Link> = Vec::new();
        let mut results: Vec<Vec<usize>> = Vec::new();
        dlx.search(&mut solution, &mut results);

        if results.is_empty() {
            return None;
        }

        let sol = &results[0];
        let mut grid = vec![[0u8; Self::GRID_SIZE]; Self::GRID_SIZE];
        for &row_idx in sol {
            let &(row_no, col_no, digit) = &self.row_map[row_idx];
            grid[row_no as usize][col_no as usize] = digit;
        }
        Some(grid)
    }

    fn _pretty_print_grid(grid: &Vec<Vec<u8>>, title: &str) {
        println!("{}", title);
        // " d |" -> len 4
        // "|" -> Self::BOX_ROW_SIZE number of extra `|` for demarcation of each box
        // "|" -> 2 extra `|` at the start and end of each row
        // total_size = Self::GRID_SIZE * 4 + Self::BOX_ROW_SIZE + 2;
        let row_display_size = Self::GRID_SIZE * 4 + Self::BOX_ROW_SIZE + 2;
        for (row_no, row) in grid.iter().enumerate() {
            if row_no % Self::BOX_ROW_SIZE == 0 {
                println!("{}", "=".repeat(row_display_size));
            } else {
                println!("{}", "-".repeat(row_display_size));
            }
            print!("|");
            for (col_no, &cell) in row.iter().enumerate() {
                if col_no % Self::BOX_ROW_SIZE == 0 {
                    print!("|");
                }
                if cell == 0 {
                    print!(" {} |", " ");
                } else {
                    print!(" {} |", cell);
                }
            }
            println!("|");
        }
        println!("{}", "=".repeat(row_display_size));
    }

    #[allow(dead_code)]
    fn pretty_print_grid(grid: &Vec<Vec<u8>>) {
        Self::_pretty_print_grid(grid, "problem:");
    }

    #[allow(dead_code)]
    fn pretty_print_solution(grid: &Vec<[u8; Self::GRID_SIZE]>) {
        let converted_grid: Vec<Vec<u8>> = grid.iter().map(|arr| arr.to_vec()).collect();
        Self::_pretty_print_grid(&converted_grid, "solution:");
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
        assert_eq!(borrowed_node.size, 0);
        assert!(Rc::ptr_eq(&node, borrowed_node.left.as_ref().unwrap()));
        assert!(Rc::ptr_eq(&node, borrowed_node.right.as_ref().unwrap()));
        assert!(Rc::ptr_eq(&node, borrowed_node.up.as_ref().unwrap()));
        assert!(Rc::ptr_eq(&node, borrowed_node.down.as_ref().unwrap()));
        assert!(borrowed_node.column.is_none());
    }

    #[test]
    fn test_new_col_node() {
        let node = Node::new_column_node();
        let borrowed_node = node.borrow();
        assert!(borrowed_node.row_id.is_none());
        assert_eq!(borrowed_node.size, 0);
        assert!(Rc::ptr_eq(&node, borrowed_node.left.as_ref().unwrap()));
        assert!(Rc::ptr_eq(&node, borrowed_node.right.as_ref().unwrap()));
        assert!(Rc::ptr_eq(&node, borrowed_node.up.as_ref().unwrap()));
        assert!(Rc::ptr_eq(&node, borrowed_node.down.as_ref().unwrap()));
        assert!(Rc::ptr_eq(&node, borrowed_node.column.as_ref().unwrap()));
    }

    #[test]
    fn test_from_matrix() {
        //   C0 C1 C2
        // R0 1  0  0
        // R1 1  1  0
        // R2 0  0  1
        let matrix: Vec<Vec<usize>> = vec![vec![1, 0, 0], vec![1, 1, 0], vec![0, 0, 1]];
        let dlx = DancingLinks::from_matrix(&matrix);
        let root = dlx.root.as_ref().unwrap();
        let borrowed_root = root.borrow();
        let col0 = borrowed_root.right.as_ref().unwrap();
        let borrowed_col0 = col0.borrow();
        let col1 = borrowed_col0.right.as_ref().unwrap();
        let borrowed_col1 = col1.borrow();
        let col2 = borrowed_col1.right.as_ref().unwrap();
        let borrowed_col2 = col2.borrow();

        assert!(Rc::ptr_eq(root, borrowed_col2.right.as_ref().unwrap()));
        assert!(Rc::ptr_eq(root, borrowed_col0.left.as_ref().unwrap()));
        assert!(Rc::ptr_eq(col0, borrowed_col1.left.as_ref().unwrap()));
        assert!(Rc::ptr_eq(col1, borrowed_col2.left.as_ref().unwrap()));
        assert!(Rc::ptr_eq(col2, borrowed_root.left.as_ref().unwrap()));

        // ---------- column 0 : rows 0 & 1 ---------------------------------------
        let c0_r0 = borrowed_col0.down.as_ref().unwrap(); // row-0 node
        let borrowed_c0_r0 = c0_r0.borrow();
        let c0_r1 = borrowed_c0_r0.down.as_ref().unwrap(); // row-1 node
        let borrowed_c0_r1 = c0_r1.borrow();

        assert_eq!(borrowed_c0_r0.row_id, Some(0));
        assert_eq!(borrowed_c0_r1.row_id, Some(1));

        // circular:
        assert!(Rc::ptr_eq(borrowed_c0_r1.down.as_ref().unwrap(), col0));
        assert!(Rc::ptr_eq(borrowed_col0.up.as_ref().unwrap(), c0_r1));
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
        assert!(Rc::ptr_eq(borrowed_c0_r0.right.as_ref().unwrap(), c0_r0));
        assert!(Rc::ptr_eq(borrowed_c0_r0.left.as_ref().unwrap(), c0_r0));
        // -------------------------------------------------------------------------

        // ---------- row 1 : two nodes, C0-R1  <-->  C1-R1 ------------------------
        let r1_n0 = c0_r1;
        let borrowed_r1_n0 = r1_n0.borrow(); // node in column-0
        let r1_n1 = borrowed_r1_n0.right.as_ref().unwrap(); // node in column-1
        let borrowed_r1_n1 = r1_n1.borrow();

        // neighbours
        assert!(Rc::ptr_eq(borrowed_r1_n1.right.as_ref().unwrap(), r1_n0));
        assert!(Rc::ptr_eq(borrowed_r1_n1.left.as_ref().unwrap(), r1_n0));
        assert!(Rc::ptr_eq(borrowed_r1_n0.left.as_ref().unwrap(), r1_n1));
        assert!(Rc::ptr_eq(borrowed_r1_n0.right.as_ref().unwrap(), r1_n1));

        // the second node really is in column-1
        assert!(Rc::ptr_eq(borrowed_r1_n1.column.as_ref().unwrap(), col1));
        // -------------------------------------------------------------------------

        // ---------- row 2 : single node (column-2) looping to itself -------------
        assert!(Rc::ptr_eq(borrowed_c2_r2.right.as_ref().unwrap(), c2_r2));
        assert!(Rc::ptr_eq(borrowed_c2_r2.left.as_ref().unwrap(), c2_r2));
        // -------------------------------------------------------------------------
    }

    #[test]
    fn test_cover_columns() {
        //   C0 C1 C2
        // R0 1  0  0
        // R1 1  1  0
        // R2 0  0  1
        let matrix: Vec<Vec<usize>> = vec![vec![1, 0, 0], vec![1, 1, 0], vec![0, 0, 1]];

        // ────────────────────────────────────────────────────────────────────────
        // build DLX structure
        let dlx = DancingLinks::from_matrix(&matrix);
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
        assert!(Rc::ptr_eq(root.borrow().left.as_ref().unwrap(), &col2));
        assert!(Rc::ptr_eq(col1.borrow().left.as_ref().unwrap(), root));
        assert!(Rc::ptr_eq(col1.borrow().right.as_ref().unwrap(), &col2));
        assert!(Rc::ptr_eq(col2.borrow().left.as_ref().unwrap(), &col1));
        assert!(Rc::ptr_eq(col2.borrow().right.as_ref().unwrap(), root));

        // column-1 is now empty
        {
            let c1 = col1.borrow();
            assert_eq!(c1.size, 0);
            assert!(Rc::ptr_eq(c1.down.as_ref().unwrap(), &col1));
            assert!(Rc::ptr_eq(c1.up.as_ref().unwrap(), &col1));
        }

        // column-2 still has its row-2 node
        {
            let c2 = col2.borrow();
            assert_eq!(c2.size, 1);
            let r2 = c2.down.as_ref().unwrap();
            assert_eq!(r2.borrow().row_id, Some(2));
            // row-2 is a single node that loops to itself left/right
            assert!(Rc::ptr_eq(r2.borrow().left.as_ref().unwrap(), r2));
            assert!(Rc::ptr_eq(r2.borrow().right.as_ref().unwrap(), r2));
        }
        // ────────────────────────────────────────────────────────────────────────

        // ──────────────── 2️⃣ cover column-2 ────────────────────────────────────
        DancingLinks::cover(&Some(col2.clone()));

        // header ring: root ↔ col1 ↔ root   (col2 removed)
        assert!(Rc::ptr_eq(root.borrow().right.as_ref().unwrap(), &col1));
        assert!(Rc::ptr_eq(root.borrow().left.as_ref().unwrap(), &col1));
        assert!(Rc::ptr_eq(col1.borrow().left.as_ref().unwrap(), root));
        assert!(Rc::ptr_eq(col1.borrow().right.as_ref().unwrap(), root));
    }

    #[test]
    fn test_uncover_columns() {
        //   C0 C1 C2
        // R0 1  0  0
        // R1 1  1  0
        // R2 0  0  1
        let matrix: Vec<Vec<usize>> = vec![vec![1, 0, 0], vec![1, 1, 0], vec![0, 0, 1]];

        // ───────────────── build structure ──────────────────────────────────
        let dlx = DancingLinks::from_matrix(&matrix);
        let root = dlx.root.as_ref().unwrap();
        // grab the three column headers (Rc clones so we can keep them)
        let col0 = root.borrow().right.as_ref().unwrap().clone();
        let col1 = col0.borrow().right.as_ref().unwrap().clone();
        let col2 = col1.borrow().right.as_ref().unwrap().clone();

        // ─────────── cover & immediately uncover column-0  ──────────────────
        DancingLinks::cover(&Some(col0.clone()));
        DancingLinks::uncover(&Some(col0.clone()));
        // --------------------------------------------------------------------
        let borrowed_root = root.borrow();
        let borrowed_col0 = col0.borrow();
        let borrowed_col1 = col1.borrow();
        let borrowed_col2 = col2.borrow();

        // 1️⃣ header ring restored:  root ↔ 0 ↔ 1 ↔ 2 ↔ root
        assert!(Rc::ptr_eq(borrowed_root.right.as_ref().unwrap(), &col0));
        assert!(Rc::ptr_eq(borrowed_root.left.as_ref().unwrap(), &col2));

        assert!(Rc::ptr_eq(borrowed_col0.left.as_ref().unwrap(), &root));
        assert!(Rc::ptr_eq(borrowed_col0.right.as_ref().unwrap(), &col1));

        assert!(Rc::ptr_eq(borrowed_col1.left.as_ref().unwrap(), &col0));
        assert!(Rc::ptr_eq(borrowed_col1.right.as_ref().unwrap(), &col2));

        assert!(Rc::ptr_eq(borrowed_col2.left.as_ref().unwrap(), &col1));
        assert!(Rc::ptr_eq(borrowed_col2.right.as_ref().unwrap(), &root));

        // 2️⃣ column sizes back to original
        assert_eq!(borrowed_col0.size, 2); // rows 0 & 1
        assert_eq!(borrowed_col1.size, 1); // row 1
        assert_eq!(borrowed_col2.size, 1); // row 2

        // 3️⃣ column-0 vertical list again holds rows 0 & 1, in order
        let c0_r0 = borrowed_col0.down.as_ref().unwrap();
        let borrowed_c0_r0 = c0_r0.borrow();
        assert_eq!(c0_r0.borrow().row_id, Some(0));
        let c0_r1 = borrowed_c0_r0.down.as_ref().unwrap();
        let borrowed_c0_r1 = c0_r1.borrow();
        assert_eq!(c0_r1.borrow().row_id, Some(1));
        assert!(Rc::ptr_eq(borrowed_c0_r1.down.as_ref().unwrap(), &col0)); // circular
        assert!(Rc::ptr_eq(borrowed_col0.up.as_ref().unwrap(), c0_r1)); // circular

        // 4️⃣ column-1 once again owns only row-1
        let c1_r1 = borrowed_col1.down.as_ref().unwrap();
        let borrowed_c1_r1 = c1_r1.borrow();
        assert_eq!(borrowed_c1_r1.row_id, Some(1));
        assert!(Rc::ptr_eq(borrowed_c1_r1.down.as_ref().unwrap(), &col1));
        assert!(Rc::ptr_eq(borrowed_col1.up.as_ref().unwrap(), c1_r1));

        // 5️⃣ column-2 once again owns only row-2
        let c2_r2 = borrowed_col2.down.as_ref().unwrap();
        let borrowed_c2_r2 = c2_r2.borrow();
        assert_eq!(borrowed_c2_r2.row_id, Some(2));
        assert!(Rc::ptr_eq(borrowed_c2_r2.down.as_ref().unwrap(), &col2));
        assert!(Rc::ptr_eq(borrowed_col2.up.as_ref().unwrap(), c2_r2));

        // 6️⃣ check a row link we know changed during cover/uncover:
        //     row-0 is back to a single self-looping node.
        assert!(Rc::ptr_eq(borrowed_c0_r0.left.as_ref().unwrap(), c0_r0));
        assert!(Rc::ptr_eq(borrowed_c0_r0.right.as_ref().unwrap(), c0_r0));
    }

    #[test]
    fn test_cover0_cover1_uncover1() {
        //   C0 C1 C2
        // R0 1  0  0
        // R1 1  1  0
        // R2 0  0  1
        let matrix: Vec<Vec<usize>> = vec![vec![1, 0, 0], vec![1, 1, 0], vec![0, 0, 1]];

        // ───────── build DLX ───────────────────────────────────────────────
        let dlx = DancingLinks::from_matrix(&matrix);
        let root = dlx.root.as_ref().unwrap();

        let col0 = root.borrow().right.as_ref().unwrap().clone();
        let col1 = col0.borrow().right.as_ref().unwrap().clone();
        let col2 = col1.borrow().right.as_ref().unwrap().clone();

        // ───────── sequence: cover C0 → cover C1 → uncover C1 ──────────────
        DancingLinks::cover(&Some(col0.clone()));
        DancingLinks::cover(&Some(col1.clone()));
        DancingLinks::uncover(&Some(col1.clone()));
        // ------------------------------------------------------------------

        // == HEADER RING should now be  root ↔ C1 ↔ C2 ↔ root ===============
        assert!(Rc::ptr_eq(root.borrow().right.as_ref().unwrap(), &col1));
        assert!(Rc::ptr_eq(root.borrow().left.as_ref().unwrap(), &col2));

        assert!(Rc::ptr_eq(col1.borrow().left.as_ref().unwrap(), root));
        assert!(Rc::ptr_eq(col1.borrow().right.as_ref().unwrap(), &col2));

        assert!(Rc::ptr_eq(col2.borrow().left.as_ref().unwrap(), &col1));
        assert!(Rc::ptr_eq(col2.borrow().right.as_ref().unwrap(), root));

        // == COLUMN STATES ==================================================
        // C0 is still covered (detached horizontally)
        assert!(!Rc::ptr_eq(root.borrow().right.as_ref().unwrap(), &col0));
        // C1 is empty: size 0, vertical list points to itself
        {
            let c1 = col1.borrow();
            assert_eq!(c1.size, 0);
            assert!(Rc::ptr_eq(c1.down.as_ref().unwrap(), &col1));
            assert!(Rc::ptr_eq(c1.up.as_ref().unwrap(), &col1));
        }
        // C2 still holds its single row-2 node
        {
            let c2 = col2.borrow();
            assert_eq!(c2.size, 1);
            let r2 = c2.down.as_ref().unwrap();
            assert_eq!(r2.borrow().row_id, Some(2));
            assert!(Rc::ptr_eq(r2.borrow().left.as_ref().unwrap(), r2));
            assert!(Rc::ptr_eq(r2.borrow().right.as_ref().unwrap(), r2));
        }
    }

    #[test]
    fn test_search_finds_r1_r2() {
        //   C0 C1 C2
        // R0 1  0  0
        // R1 1  1  0
        // R2 0  0  1
        let matrix: Vec<Vec<usize>> = vec![vec![1, 0, 0], vec![1, 1, 0], vec![0, 0, 1]];

        // Build DLX structure
        let dlx = DancingLinks::from_matrix(&matrix);

        // Run Algorithm X
        let mut solution: Vec<Link> = Vec::new(); // working stack
        let mut results: Vec<Vec<usize>> = Vec::new(); // all solutions
        dlx.search(&mut solution, &mut results);

        // There is exactly one solution …
        assert_eq!(results.len(), 1);

        // … and it must be rows {1, 2} (order independent)
        let mut sol = results[0].clone();
        sol.sort_unstable();
        assert_eq!(sol, vec![1, 2]);
    }

    #[test]
    fn test_matrix_6x7_solution_135() {
        // Matrix rows are numbered 0-5
        //   C0 C1 C2 C3 C4 C5 C6
        let matrix: Vec<Vec<usize>> = vec![
            vec![1, 0, 0, 1, 0, 0, 1], // 0
            vec![1, 0, 0, 1, 0, 0, 0], // 1  ← want this
            vec![0, 0, 0, 1, 1, 0, 1], // 2
            vec![0, 0, 1, 0, 1, 1, 0], // 3  ← want this
            vec![0, 1, 1, 0, 0, 1, 1], // 4
            vec![0, 1, 0, 0, 0, 0, 1], // 5  ← want this
        ];

        // Build Dancing Links structure
        let dlx = DancingLinks::from_matrix(&matrix);

        // Run Algorithm X
        let mut partial: Vec<Link> = Vec::new();
        let mut solutions: Vec<Vec<usize>> = Vec::new();
        dlx.search(&mut partial, &mut solutions);

        // There must be at least one solution that is exactly [1, 3, 5]
        let target = vec![1, 3, 5];
        let found = solutions.iter().any(|sol| {
            let mut s = sol.clone();
            s.sort_unstable();
            s == target
        });

        assert!(found, "expected solution [1,3,5] not found");
    }

    #[test]
    fn test_sudo_solver() {
        let sudoku_grid: Vec<Vec<u8>> = vec![
            vec![5, 3, 0, 0, 7, 0, 0, 0, 0],
            vec![6, 0, 0, 1, 9, 5, 0, 0, 0],
            vec![0, 9, 8, 0, 0, 0, 0, 6, 0],
            vec![8, 0, 0, 0, 6, 0, 0, 0, 3],
            vec![4, 0, 0, 8, 0, 3, 0, 0, 1],
            vec![7, 0, 0, 0, 2, 0, 0, 0, 6],
            vec![0, 6, 0, 0, 0, 0, 2, 8, 0],
            vec![0, 0, 0, 4, 1, 9, 0, 0, 5],
            vec![0, 0, 0, 0, 8, 0, 0, 7, 9],
        ];
        let sudoku_solver = SudokuSolver::from_grid_to_exact_cover(&sudoku_grid);
        let solution = sudoku_solver.solve();
        SudokuSolver::pretty_print_grid(&sudoku_grid);
        if let Some(sol) = solution {
            SudokuSolver::pretty_print_solution(&sol);
        }
    }

    #[test]
    fn test_sudo_solver_for_extreme_difficulty() {
        let sudoku_grid: Vec<Vec<u8>> = vec![
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 3, 0, 5, 0, 0, 0, 6, 0],
            vec![0, 0, 2, 0, 8, 0, 0, 4, 0],
            vec![0, 5, 0, 0, 6, 0, 0, 8, 0],
            vec![0, 1, 0, 0, 7, 0, 0, 0, 0],
            vec![3, 0, 0, 0, 2, 0, 6, 7, 0],
            vec![0, 0, 0, 0, 0, 0, 1, 0, 7],
            vec![1, 0, 0, 8, 0, 0, 0, 0, 4],
            vec![0, 4, 0, 9, 0, 0, 0, 3, 0],
        ];
        let sudoku_solver = SudokuSolver::from_grid_to_exact_cover(&sudoku_grid);
        let solution = sudoku_solver.solve();
        SudokuSolver::pretty_print_grid(&sudoku_grid);
        if let Some(sol) = solution {
            SudokuSolver::pretty_print_solution(&sol);
        }
    }
}

// Python code for dancing links
// class Node:
//     def __init__(self, row_id=None):
//         self.left = self
//         self.right = self
//         self.up = self
//         self.down = self
//         self.column = None
//         self.row_id = row_id  # Added to track real row indices

// class ColumnNode(Node):
//     def __init__(self, name):
//         super().__init__()
//         self.name = name
//         self.size = 0
//         self.column = self

// def build_dlx_matrix(matrix):
//     root = ColumnNode("root")
//     column_headers = []

//     # Create column headers and link them left-right
//     for i in range(len(matrix[0])):
//         column = ColumnNode(str(i))
//         column_headers.append(column)
//         column.right = root
//         column.left = root.left
//         root.left.right = column
//         root.left = column

//     # Create nodes
//     for row_idx, row in enumerate(matrix):
//         prev = None
//         for j, cell in enumerate(row):
//             if cell == 1:
//                 column = column_headers[j]
//                 node = Node(row_idx)
//                 node.column = column

//                 # Link into column
//                 node.down = column
//                 node.up = column.up
//                 column.up.down = node
//                 column.up = node
//                 column.size += 1

//                 # Link left-right in row
//                 if prev is None:
//                     prev = node
//                 node.left = prev
//                 node.right = prev.right
//                 prev.right.left = node
//                 prev.right = node
//                 prev = node

//     return root

// def cover(column):
//     column.right.left = column.left
//     column.left.right = column.right
//     i = column.down
//     while i != column:
//         j = i.right
//         while j != i:
//             j.down.up = j.up
//             j.up.down = j.down
//             j.column.size -= 1
//             j = j.right
//         i = i.down

// def uncover(column):
//     i = column.up
//     while i != column:
//         j = i.left
//         while j != i:
//             j.column.size += 1
//             j.down.up = j
//             j.up.down = j
//             j = j.left
//         i = i.up
//     column.right.left = column
//     column.left.right = column

// def search(root, solution, results):
//     if root.right == root:
//         results.append([node.row_id for node in solution])
//         return

//     # Choose column with fewest nodes
//     column = root.right
//     min_size = column.size
//     c = column.right
//     while c != root:
//         if c.size < min_size:
//             column = c
//             min_size = c.size
//         c = c.right

//     cover(column)
//     r = column.down
//     while r != column:
//         solution.append(r)
//         j = r.right
//         while j != r:
//             cover(j.column)
//             j = j.right
//         search(root, solution, results)
//         r = solution.pop()
//         column = r.column
//         j = r.left
//         while j != r:
//             uncover(j.column)
//             j = j.left
//         r = r.down
//     uncover(column)

// def solve_exact_cover(matrix):
//     root = build_dlx_matrix(matrix)
//     results = []
//     search(root, [], results)
//     return results

// if __name__ == "__main__":
//     # matrix = [
//     #     [1, 0, 0, 1, 0, 0, 1],
//     #     [1, 0, 0, 1, 0, 0, 0],
//     #     [0, 0, 0, 1, 1, 0, 1],
//     #     [0, 0, 1, 0, 1, 1, 0],
//     #     [0, 1, 1, 0, 0, 1, 1],
//     #     [0, 1, 0, 0, 0, 0, 1],
//     # ]
//     matrix = [
//       [1, 0, 0],
//       [1, 1, 0],
//       [0, 0, 1]
//     ]
//     solutions = solve_exact_cover(matrix)
//     print(f"Solutions found: {len(solutions)}")
//     for sol in solutions:
//         print("Solution rows:", sol)
