use std::fmt::Display;

/// A small B-tree meant for understanding how insert and get work.
pub struct BTree<K, V> {
    /// The minimum degree controls how many keys each node can hold.
    min_degree: usize,
    root: Option<Node<K, V>>,
    len: usize,
}

/// Each node stores multiple keys and values.
///
/// If the node is not a leaf, it also stores child nodes.
#[derive(Debug)]
struct Node<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
    children: Vec<Node<K, V>>,
}

/// The visualizer reads from this snapshot instead of the generic tree itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BTreeSnapshot {
    pub min_degree: usize,
    pub len: usize,
    pub root: Option<BTreeNodeSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BTreeNodeSnapshot {
    pub id: usize,
    pub depth: usize,
    pub is_leaf: bool,
    pub key_count: usize,
    pub child_count: usize,
    pub keys: Vec<String>,
    pub values: Vec<String>,
    pub children: Vec<BTreeNodeSnapshot>,
}

impl<K, V> Node<K, V> {
    fn new_leaf(key: K, value: V) -> Self {
        Self {
            keys: vec![key],
            values: vec![value],
            children: Vec::new(),
        }
    }

    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    fn is_full(&self, min_degree: usize) -> bool {
        self.keys.len() == 2 * min_degree - 1
    }

    /// Search inside this node first. If the key is not here, continue into
    /// the matching child.
    fn get<'a>(&'a self, key: &K) -> Option<&'a V>
    where
        K: Ord,
    {
        match self.keys.binary_search(key) {
            Ok(index) => Some(&self.values[index]),
            Err(index) => self.children.get(index).and_then(|child| child.get(key)),
        }
    }

    /// Split a full child into two smaller children and move the middle key up.
    fn split_child(&mut self, child_index: usize, min_degree: usize) {
        // Temporarily remove the full child from the parent so we can split it
        // into two separate nodes.
        let mut child = self.children.remove(child_index);

        // Keep the left half in `child` and move the middle+right portion into
        // fresh vectors.
        let mut right_keys = child.keys.split_off(min_degree - 1);
        let mut right_values = child.values.split_off(min_degree - 1);

        // The first element of the moved portion is the median key/value.
        // That pair must move up into the parent.
        let middle_key = right_keys.remove(0);
        let middle_value = right_values.remove(0);

        // Internal nodes also need their child pointers split.
        // The left node keeps the first `min_degree` children and the right
        // node receives the rest.
        let right_children = if child.is_leaf() {
            Vec::new()
        } else {
            child.children.split_off(min_degree)
        };

        // Insert the promoted median into the parent at `child_index`.
        self.keys.insert(child_index, middle_key);
        self.values.insert(child_index, middle_value);

        // Put the two split children back into the parent:
        // - `child` is now the left half
        // - the new node is the right half
        self.children.insert(child_index, child);
        self.children.insert(
            child_index + 1,
            Node {
                keys: right_keys,
                values: right_values,
                children: right_children,
            },
        );
    }

    /// Insert into a node that is known to have space.
    ///
    /// If the key already exists, replace its value and return the old value.
    fn insert_non_full(&mut self, key: K, value: V, min_degree: usize) -> Option<V>
    where
        K: Ord,
    {
        match self.keys.binary_search(&key) {
            // key already exists in this node, so we only
            // replace the old value.
            Ok(index) => Some(std::mem::replace(&mut self.values[index], value)),

            
            // This a leaf node, and they key was not found.
            // The `index` is the exact sorted position where the
            // new key must be inserted to keep the keys ordered.
            Err(index) if self.is_leaf() => {
                self.keys.insert(index, key);
                self.values.insert(index, value);
                None
            }

            // `Err(index)` on an internal node means the key is not stored in
            // this node, and the search must continue in child `index`.
            //
            // Example:
            // keys = [10, 20, 30]
            // key  = 25
            // binary_search returns `Err(2)`, so we continue into child 2,
            // which is the subtree between 20 and 30.
            Err(mut index) => {
                if self.children[index].is_full(min_degree) {
                    // Before descending, split a full child so we never recurse
                    // into a node that has no room left.
                    self.split_child(index, min_degree);

                    // Splitting moved one key up into the current node.
                    // We now decide whether the new key belongs:
                    // - to the left child
                    // - exactly on the promoted key
                    // - to the right child
                    if key > self.keys[index] {
                        index += 1;
                    } else if key == self.keys[index] {
                        return Some(std::mem::replace(&mut self.values[index], value));
                    }
                }

                self.children[index].insert_non_full(key, value, min_degree)
            }
        }
    }

    fn to_snapshot(&self, depth: usize, next_id: &mut usize) -> BTreeNodeSnapshot
    where
        K: Display,
        V: Display,
    {
        let id = *next_id;
        *next_id += 1;

        BTreeNodeSnapshot {
            id,
            depth,
            is_leaf: self.is_leaf(),
            key_count: self.keys.len(),
            child_count: self.children.len(),
            keys: self.keys.iter().map(ToString::to_string).collect(),
            values: self.values.iter().map(ToString::to_string).collect(),
            children: self
                .children
                .iter()
                .map(|child| child.to_snapshot(depth + 1, next_id))
                .collect(),
        }
    }
}

impl<K, V> BTree<K, V> {
    pub fn new(min_degree: usize) -> Self {
        assert!(min_degree >= 2, "minimum degree must be at least 2");

        Self {
            min_degree,
            root: None,
            len: 0,
        }
    }

    pub fn min_degree(&self) -> usize {
        self.min_degree
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: Ord,
    {
        self.root.as_ref().and_then(|root| root.get(key))
    }

    /// Insert a key/value pair.
    ///
    /// If the key already exists, its value is replaced and the old value is
    /// returned.
    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord,
    {
        if self.root.is_none() {
            self.root = Some(Node::new_leaf(key, value));
            self.len = 1;
            return None;
        }

        if self
            .root
            .as_ref()
            .is_some_and(|root| root.is_full(self.min_degree))
        {
            let old_root = self.root.take().unwrap();
            self.root = Some(Node {
                keys: Vec::new(),
                values: Vec::new(),
                children: vec![old_root],
            });
            self.root.as_mut().unwrap().split_child(0, self.min_degree);
        }

        let result = self
            .root
            .as_mut()
            .unwrap()
            .insert_non_full(key, value, self.min_degree);

        if result.is_none() {
            self.len += 1;
        }

        result
    }

    pub fn snapshot(&self) -> BTreeSnapshot
    where
        K: Display,
        V: Display,
    {
        let mut next_id = 0;

        BTreeSnapshot {
            min_degree: self.min_degree,
            len: self.len,
            root: self
                .root
                .as_ref()
                .map(|root| root.to_snapshot(0, &mut next_id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BTree;

    #[test]
    fn empty_tree_has_no_values() {
        let tree: BTree<i32, String> = BTree::new(2);

        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.get(&10), None);
    }

    #[test]
    fn insert_and_get_work_for_a_single_key() {
        let mut tree = BTree::new(2);

        assert_eq!(tree.insert(10, "ten".to_string()), None);
        assert_eq!(tree.get(&10), Some(&"ten".to_string()));
        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn inserting_an_existing_key_replaces_the_value() {
        let mut tree = BTree::new(2);

        tree.insert(10, "ten".to_string());

        assert_eq!(tree.insert(10, "TEN".to_string()), Some("ten".to_string()));
        assert_eq!(tree.get(&10), Some(&"TEN".to_string()));
        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn inserts_can_split_the_root() {
        let mut tree = BTree::new(2);

        for key in [10, 20, 5, 6] {
            tree.insert(key, key * 10);
        }

        assert_eq!(tree.get(&5), Some(&50));
        assert_eq!(tree.get(&6), Some(&60));
        assert_eq!(tree.get(&10), Some(&100));
        assert_eq!(tree.get(&20), Some(&200));
    }

    #[test]
    fn get_works_across_multiple_levels() {
        let mut tree = BTree::new(2);

        for key in [50, 40, 60, 30, 70, 20, 80, 10, 90, 0] {
            tree.insert(key, key + 1);
        }

        assert_eq!(tree.get(&0), Some(&1));
        assert_eq!(tree.get(&30), Some(&31));
        assert_eq!(tree.get(&60), Some(&61));
        assert_eq!(tree.get(&90), Some(&91));
        assert_eq!(tree.get(&999), None);
    }

    #[test]
    fn snapshot_contains_the_data_needed_by_the_ui() {
        let mut tree = BTree::new(2);
        for (key, value) in [(10, "ten"), (20, "twenty"), (5, "five"), (6, "six")] {
            tree.insert(key, value.to_string());
        }

        let snapshot = tree.snapshot();
        let root = snapshot.root.expect("tree should not be empty");

        assert_eq!(snapshot.len, 4);
        assert_eq!(root.depth, 0);
        assert_eq!(root.key_count, 1);
        assert_eq!(root.child_count, 2);
        assert!(!root.children.is_empty());
    }

    #[test]
    fn duplicate_key_can_be_promoted_during_split() {
        let mut tree = BTree::new(2);

        for key in [10, 20, 5, 6, 7] {
            tree.insert(key, key);
        }

        assert_eq!(tree.insert(6, 600), Some(6));
        assert_eq!(tree.get(&6), Some(&600));
    }

    #[test]
    #[should_panic(expected = "minimum degree")]
    fn minimum_degree_must_be_at_least_two() {
        let _tree: BTree<i32, i32> = BTree::new(1);
    }
}
