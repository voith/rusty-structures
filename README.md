# rusty-structures

A collection of basic data structures implemented in Rust, designed to help you understand ownership, borrowing, and the power of Rust’s type system.

## 🚀 Features

- Singly linked list (`LinkedList`)
    - Basic operations: add, remove, print
    - Conversion to vector for easier testing
    - Clean and idiomatic Rust code

- Doubly Linked List (DoublyLinkedList)
    - Bidirectional links (previous and next)
    - Safe shared ownership with Rc and interior mutability using RefCell
    - Tests that validate pointer integrity (Rc::ptr_eq)

- Dancing Links (`DancingLinks`, `dancing_links.rs`)
    - Safe, `Rc<RefCell<Node>>`–based implementation of Knuth’s DLX (Exact Cover)
    - Build from a `Vec<Vec<bool>>` or `Vec<Vec<usize>>` matrix
    - Core operations: `cover`, `uncover`, and `search` (Algorithm X)
    - Circular doubly-linked lists for both columns and rows
    - Unit tests for structure integrity, cover/uncover correctness, and exact-cover solutions  
      (small diagonal matrices, Knuth’s 6×7 example, Sudoku)

- Efficient Dancing Links (`EfficientDancingLinks`, `efficient_dancing_links.rs`)
    - High-performance, raw-pointer–based DLX using `Box<Node>` and `*mut Node`
    - Self-referential node initialization without runtime borrow checks
    - All `cover`, `uncover`, and `search` logic in `unsafe` blocks for minimal overhead
    - Heuristic column selection (smallest size) and full Algorithm X recursion
    - Comprehensive tests ensuring pointer correctness and correct solution enumeration

## 🔧 Getting Started

Clone the repository:

```bash
git clone https://github.com/your-username/rusty-structures.git
cd rusty-structures
```

Build and run tests:
```bash
cargo test
```

## 📚 Learning Goals
This project helps you:

- Understand heap allocation in Rust with `Box` and how to create self-referential structures.  
- Practice safe shared mutability using `Rc<RefCell>` and interior mutability patterns.  
- Manage optional and nullable links with `Option`, pattern matching, and pointer-equality checks (`Rc::ptr_eq`).  
- Dive into `unsafe` Rust: use raw pointers (`*mut T`), initialize pointers manually, and wrap operations in `unsafe` blocks for performance.  
- Implement Knuth’s Dancing Links (DLX) for the exact-cover problem, including the `cover`, `uncover`, and recursive `search` (Algorithm X).  
- Design and test both safe (Rust-managed) and efficient (raw-pointer) versions of the same algorithm, comparing idiomatic and low-level approaches.  
- Build a real-world Sudoku solver on top of DLX to see exact cover in action.  
- Write comprehensive unit tests that validate pointer integrity and algorithm correctness in both safe and unsafe contexts.  
- Compare and contrast idiomatic Rust data-structure design with manual, performance-oriented optimizations.

## 📦 License
This project is licensed under the MIT License.
