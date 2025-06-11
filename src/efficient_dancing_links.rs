use std::ptr;

#[derive(Debug)]
pub struct Node {
    row_id: usize,
    size: usize,
    column: *mut Node,
    left: *mut Node,
    right: *mut Node,
    up: *mut Node,
    down: *mut Node,
}

impl Node {
    fn new(row_id: usize) -> Box<Self> {
        let mut node = Box::new(Node {
            row_id: row_id,
            size: 0,
            column: ptr::null_mut(),
            left: ptr::null_mut(),
            right: ptr::null_mut(),
            up: ptr::null_mut(),
            down: ptr::null_mut(),
        });

        let node_ptr = &mut *node;
        (*node_ptr).column = node_ptr;
        (*node_ptr).left = node_ptr;
        (*node_ptr).right = node_ptr;
        (*node_ptr).up = node_ptr;
        (*node_ptr).down = node_ptr;

        node
    }
}

pub struct EfficientDancingLinks {
    root: Box<Node>,
    _column_headers: Vec<Box<Node>>,
    _rows: Vec<Box<Node>>,
}

impl EfficientDancingLinks {
    pub fn from_matrix(grid: &Vec<Vec<bool>>) -> Box<Self> {
        assert!(
            !grid.is_empty() && !grid[0].is_empty(),
            "matrix must be nonempty"
        );

        let mut root = Node::new(0);
        let root_ptr: *mut _ = &mut *root;
        let no_columns = grid[0].len();
        let mut column_headers: Vec<Box<Node>> = Vec::with_capacity(no_columns);
        let mut rows_vec: Vec<Box<Node>> = Vec::new();

        for _ in 0..no_columns {
            let mut column = Node::new(0);
            let column_ptr = &mut *column;
            unsafe {
                (*column_ptr).right = root_ptr;
                (*column_ptr).left = (*root_ptr).left;
                (*(*root_ptr).left).right = column_ptr;
                (*root_ptr).left = column_ptr;
            }
            column_headers.push(column);
        }
        for (row_idx, row) in grid.iter().enumerate() {
            let mut previous_node_ptr: *mut Node = ptr::null_mut();
            for (col_idx, &cell) in row.iter().enumerate() {
                if !cell {
                    continue;
                }
                let mut row_node = Node::new(row_idx);
                let column_ptr = &mut *column_headers[col_idx];
                let row_node_ptr = &mut *row_node;
                unsafe {
                    (*row_node_ptr).column = column_ptr;
                    (*row_node_ptr).down = column_ptr;
                    (*row_node_ptr).up = (*column_ptr).up;
                    (*(*column_ptr).up).down = row_node_ptr;
                    (*column_ptr).up = row_node_ptr;
                    (*column_ptr).size += 1;
                }

                unsafe {
                    if !previous_node_ptr.is_null() {
                        (*row_node_ptr).left = previous_node_ptr;
                        (*row_node_ptr).right = (*previous_node_ptr).right;
                        (*(*previous_node_ptr).right).left = row_node_ptr;
                        (*previous_node_ptr).right = row_node_ptr;
                    }
                }
                previous_node_ptr = row_node_ptr;
                rows_vec.push(row_node);
            }
        }
        Box::new(EfficientDancingLinks {
            root,
            _column_headers: column_headers,
            _rows: rows_vec,
        })
    }

    fn cover(column_ptr: *mut Node) {
        unsafe {
            (*(*column_ptr).right).left = (*column_ptr).left;
            (*(*column_ptr).left).right = (*column_ptr).right;
            let mut vertical_ptr = (*column_ptr).down;
            while vertical_ptr != column_ptr {
                let mut horizontal_ptr = (*vertical_ptr).right;
                while horizontal_ptr != vertical_ptr {
                    (*(*horizontal_ptr).down).up = (*horizontal_ptr).up;
                    (*(*horizontal_ptr).up).down = (*horizontal_ptr).down;
                    (*(*horizontal_ptr).column).size -= 1;
                    horizontal_ptr = (*horizontal_ptr).right;
                }
                vertical_ptr = (*vertical_ptr).down;
            }
        }
    }

    fn uncover(column_ptr: *mut Node) {
        unsafe {
            let mut vertical_ptr = (*column_ptr).up;
            while vertical_ptr != column_ptr {
                let mut horizontal_ptr = (*vertical_ptr).left;
                while horizontal_ptr != vertical_ptr {
                    (*(*horizontal_ptr).column).size += 1;
                    (*(*horizontal_ptr).down).up = horizontal_ptr;
                    (*(*horizontal_ptr).up).down = horizontal_ptr;
                    horizontal_ptr = (*horizontal_ptr).left;
                }
                vertical_ptr = (*vertical_ptr).up;
            }
            (*(*column_ptr).right).left = column_ptr;
            (*(*column_ptr).left).right = column_ptr;
        }
    }

    pub fn search(&self, solution: &mut Vec<*mut Node>, results: &mut Vec<Vec<usize>>) {
        unsafe {
            let root_ptr = &*self.root as *const Node as *mut Node;
            // If no columns remain, store the solution.
            if (*root_ptr).right == root_ptr {
                let row_ids: Vec<usize> = solution.iter().map(|&_node| (*_node).row_id).collect();
                results.push(row_ids);
                return;
            }

            let column_ptr = {
                let mut best_col = (*root_ptr).right;
                let mut min_size = (*best_col).size;
                let mut col = (*best_col).right;
                while col != root_ptr {
                    if (*col).size < min_size {
                        best_col = col;
                        min_size = (*col).size;
                    }
                    col = (*col).right;
                }
                best_col
            };
            Self::cover(column_ptr);
            let mut fwd_row = (*column_ptr).down;
            while fwd_row != column_ptr {
                solution.push(fwd_row);
                let mut fwd_next_row = (*fwd_row).right;
                while fwd_next_row != fwd_row {
                    Self::cover((*fwd_next_row).column);
                    fwd_next_row = (*fwd_next_row).right;
                }

                self.search(solution, results);

                // backtrack
                let bck_row = solution.pop().unwrap();
                let mut bck_next_row = (*bck_row).left;
                while bck_next_row != bck_row {
                    Self::uncover((*bck_next_row).column);
                    bck_next_row = (*bck_next_row).left;
                }
                fwd_row = (*fwd_row).down;
            }
            Self::uncover(column_ptr);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new_node() {
        let node = Node::new(1);
        let node_ptr = &*node as *const Node as *mut Node;

        assert_eq!(node.column, node_ptr);
        assert_eq!(node.left, node_ptr);
        assert_eq!(node.right, node_ptr);
        assert_eq!(node.up, node_ptr);
        assert_eq!(node.down, node_ptr);

        assert_eq!(node.row_id, 1);
        assert_eq!(node.size, 0);
    }

    #[test]
    fn test_from_matrix_small_diagonal() {
        // ┌ a b ┐
        // │ 1 0 │
        // │ 0 1 │
        // └     ┘
        let grid = vec![vec![true, false], vec![false, true]];
        let mut dlx = EfficientDancingLinks::from_matrix(&grid);

        // pointers to root and columns
        let root_ptr: *mut Node = &mut *dlx.root;
        let col0_ptr: *mut Node = &mut *dlx._column_headers[0];
        let col1_ptr: *mut Node = &mut *dlx._column_headers[1];

        // 1) we should have exactly 2 column headers
        assert_eq!(dlx._column_headers.len(), 2);

        unsafe {
            // 2) root ↔ columns circular linkage:
            //    root.right → col0 → col1 → root
            assert_eq!((*root_ptr).right, col0_ptr);
            assert_eq!((*col0_ptr).right, col1_ptr);
            assert_eq!((*col1_ptr).right, root_ptr);
            //    backwards:
            assert_eq!((*root_ptr).left, col1_ptr);
            assert_eq!((*col1_ptr).left, col0_ptr);
            assert_eq!((*col0_ptr).left, root_ptr);

            // 3) each column saw exactly one “true” in its list
            assert_eq!((*col0_ptr).size, 1);
            assert_eq!((*col1_ptr).size, 1);
        }

        // 4) we should have created exactly two row‐nodes
        assert_eq!(dlx._rows.len(), 2);

        // 5) verify each row‐node links back into the correct column
        for (i, row_node_box) in dlx._rows.iter().enumerate() {
            // row 0 → column 0, row 1 → column 1
            let expected_col = if i == 0 { col0_ptr } else { col1_ptr };
            let n_ptr = &**row_node_box as *const Node as *mut Node;

            unsafe {
                // row_id matches
                assert_eq!((*n_ptr).row_id, i);

                // horizontal: single‐node circle
                assert_eq!((*n_ptr).left, n_ptr);
                assert_eq!((*n_ptr).right, n_ptr);

                // vertical: should live in its column’s circular list
                assert_eq!((*(*n_ptr).up).down, n_ptr);
                assert_eq!((*(*n_ptr).down).up, n_ptr);

                // column pointer is correct
                assert_eq!((*n_ptr).column, expected_col);
            }
        }
    }

    #[test]
    fn test_from_matrix_3x3_diagonal() {
        // 3×3 identity matrix:
        // ┌ a b c ┐
        // │ 1 0 0 │
        // │ 0 1 0 │
        // │ 0 0 1 │
        // └       ┘
        let grid = vec![
            vec![true, false, false],
            vec![false, true, false],
            vec![false, false, true],
        ];
        let mut dlx = EfficientDancingLinks::from_matrix(&grid);

        // pointers to root and columns
        let root_ptr: *mut Node = &mut *dlx.root;
        let col0_ptr: *mut Node = &mut *dlx._column_headers[0];
        let col1_ptr: *mut Node = &mut *dlx._column_headers[1];
        let col2_ptr: *mut Node = &mut *dlx._column_headers[2];

        // 1) three column headers
        assert_eq!(dlx._column_headers.len(), 3);

        unsafe {
            // 2) root ↔ columns circular linkage:
            //    root → col0 → col1 → col2 → root
            assert_eq!((*root_ptr).right, col0_ptr);
            assert_eq!((*col0_ptr).right, col1_ptr);
            assert_eq!((*col1_ptr).right, col2_ptr);
            assert_eq!((*col2_ptr).right, root_ptr);
            //    backwards:
            assert_eq!((*root_ptr).left, col2_ptr);
            assert_eq!((*col2_ptr).left, col1_ptr);
            assert_eq!((*col1_ptr).left, col0_ptr);
            assert_eq!((*col0_ptr).left, root_ptr);

            // 3) each column saw exactly one “true”
            assert_eq!((*col0_ptr).size, 1);
            assert_eq!((*col1_ptr).size, 1);
            assert_eq!((*col2_ptr).size, 1);
        }

        // 4) exactly three row‐nodes created
        assert_eq!(dlx._rows.len(), 3);

        // 5) each row‐node is in its own 1‐element horizontal circle
        //    and linked vertically under the correct column
        for (i, row_node_box) in dlx._rows.iter().enumerate() {
            // expected column for row i
            let expected_col = match i {
                0 => col0_ptr,
                1 => col1_ptr,
                2 => col2_ptr,
                _ => unreachable!(),
            };
            let n_ptr: *mut Node = &**row_node_box as *const Node as *mut Node;

            unsafe {
                // row_id matches
                assert_eq!((*n_ptr).row_id, i);

                // horizontal: single‐node loop
                assert_eq!((*n_ptr).left, n_ptr);
                assert_eq!((*n_ptr).right, n_ptr);

                // vertical: in its column’s circular list
                assert_eq!((*(*n_ptr).up).down, n_ptr);
                assert_eq!((*(*n_ptr).down).up, n_ptr);

                // column pointer correct
                assert_eq!((*n_ptr).column, expected_col);
            }
        }
    }

    #[test]
    fn test_cover_uncover_column0_on_3x3() {
        // Matrix:
        //    C0   C1   C2
        // R0  1    0    0
        // R1  1    1    0
        // R2  0    0    1
        let grid = vec![
            vec![true, false, false],
            vec![true, true, false],
            vec![false, false, true],
        ];
        let mut dlx = EfficientDancingLinks::from_matrix(&grid);

        // raw pointers to root and each column header
        let root_ptr: *mut Node = &mut *dlx.root;
        let col0_ptr: *mut Node = &mut *dlx._column_headers[0];
        let col1_ptr: *mut Node = &mut *dlx._column_headers[1];
        let col2_ptr: *mut Node = &mut *dlx._column_headers[2];

        // 1) capture initial state
        let (init_rr, init_rl, init_c0_sz, init_c1_sz, init_c2_sz) = unsafe {
            (
                (*root_ptr).right,
                (*root_ptr).left,
                (*col0_ptr).size,
                (*col1_ptr).size,
                (*col2_ptr).size,
            )
        };

        // 2) cover C0
        EfficientDancingLinks::cover(col0_ptr);

        unsafe {
            // after cover, C0 is spliced out: root.right should now be C1
            assert_eq!((*root_ptr).right, col1_ptr);
            assert_eq!((*col1_ptr).left, root_ptr);
            // C0 should no longer be adjacent to root
            assert_ne!((*root_ptr).left, col0_ptr);
            assert_ne!((*root_ptr).right, col0_ptr);
        }

        // 3) uncover C0
        EfficientDancingLinks::uncover(col0_ptr);

        unsafe {
            // root links restored
            assert_eq!((*root_ptr).right, init_rr);
            assert_eq!((*root_ptr).left, init_rl);

            // column sizes restored
            assert_eq!((*col0_ptr).size, init_c0_sz);
            assert_eq!((*col1_ptr).size, init_c1_sz);
            assert_eq!((*col2_ptr).size, init_c2_sz);

            // verify C0's vertical chain is intact again
            let mut v = (*col0_ptr).down;
            while v != col0_ptr {
                // each node in C0 should point back into the column
                assert_eq!((*(*v).down).up, v);
                assert_eq!((*(*v).up).down, v);
                v = (*v).down;
            }
        }
    }

    #[test]
    fn test_search_exact_cover() {
        // Example matrix from Knuth’s paper; the unique exact cover is rows 1, 3, and 5.
        let matrix: Vec<Vec<usize>> = vec![
            vec![1, 0, 0, 1, 0, 0, 1], // 0
            vec![1, 0, 0, 1, 0, 0, 0], // 1  ← in solution
            vec![0, 0, 0, 1, 1, 0, 1], // 2
            vec![0, 0, 1, 0, 1, 1, 0], // 3  ← in solution
            vec![0, 1, 1, 0, 0, 1, 1], // 4
            vec![0, 1, 0, 0, 0, 0, 1], // 5  ← in solution
        ];
        // Convert usize matrix to bool grid
        let grid: Vec<Vec<bool>> = matrix
            .iter()
            .map(|row| row.iter().map(|&v| v == 1).collect())
            .collect();

        // Build the DLX structure
        let dlx = EfficientDancingLinks::from_matrix(&grid);

        // Prepare containers for the search
        let mut solution: Vec<*mut Node> = Vec::new();
        let mut results: Vec<Vec<usize>> = Vec::new();

        // Run Algorithm X
        dlx.search(&mut solution, &mut results);

        // We expect exactly one solution, and it should be rows [1, 3, 5]
        assert_eq!(results.len(), 1, "should find exactly one cover");
        assert_eq!(results[0], vec![1, 3, 5], "solution rows should be [1,3,5]");
    }
}
