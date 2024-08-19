//! This is a custom implementation of a Merkle Tree, used in set membership ZKPs.
//! Other crates were too over bloated and not flexible enough.

use thiserror::Error;

/// Error type for Merkle Tree operations.
#[derive(Error, Debug)]
pub enum Error {
    /// The entered element index was larger than there are elements in the tree.
    #[error("Specified element is out of bounds for this Merkle Tree {}/{}", .0, .1)]
    ElementOutOfBounds(usize, usize),
    /// Occurs if a Merkle tree is being creates without any nodes.
    #[error("Merkle Tree cannot be empty")]
    EmptyTree,
}
type Result<T> = std::result::Result<T, Error>;

/// A struct containing all the info for Merkle Proof for a leaf in a Merkle Tree.
pub struct MerkleProof<H> {
    /// The index of the leaf for which the proof is generated.
    pub _leaf_index: usize,
    /// The root of the Merkle Tree.
    pub root: H,
    /// The proof for the leaf.
    pub proof: Vec<H>,
    /// The path for hashing a leaf with it's siblings to get the root.
    /// The index of the path element corresponds to the index of the proof element.
    /// Left means that the proof element should be on the left side of the hash and
    /// the accumulated digest should be on the right.
    /// Right means that the proof element should be on the right side of the hash and.
    /// the accumulated digest should be on the left.
    pub path: Vec<MerkleHashPath>,
}

/// Alias to abstract away some complexity from the type of MerkleTree struct.
/// This type accepts a function which takes two hash values and hashes them together.
type NodeHashFn<H> = Box<dyn Fn(&H, &H) -> H>;

/// Alias to abstract away some complexity from the type of MerkleTree struct.
/// This type accepts a function which takes a preimage values and hashes it.
type LeafHashFn<T, H> = Box<dyn Fn(&T) -> H>;

/// A struct representing a Merkle Tree itself.
/// T represents the type of the initial unhashed data.
/// H represents the type of the hashed data which will be stored in the nodes.
/// The tree is meant to be immutable, if you need to change the leaf values,
/// you should create a new tree.
///
/// # Example
///
/// ```
/// use digital_voting::set_membership_zkp::merkle::MerkleTree;
///
/// struct MyStruct {
///     // A which has u64 as the initial data and [u8; 32] as the hashed data.
///     merkle_tree: MerkleTree<u64, [u8; 32]>,
/// }
/// ```
pub struct MerkleTree<T, H> {
    /// Number of data elements from which the tree had been constructed (number of leaves).
    leaf_count: usize,
    /// The nodes of the Merkle Tree containing the hashes of the leaves and all the subsequent
    /// neighboring node hashes. The last element is the root of the Merkle tree.
    /// These will be generated by the Merkle Tree.
    nodes: Vec<H>,
    /// The function used to hash two nodes together.
    node_hash_function: NodeHashFn<H>,
    /// The function used to hash a leaf.
    leaf_hash_function: LeafHashFn<T, H>,
}

// TODO make sure this is according to standard.
/// Enum representing the path to a hash in a Merkle Tree.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MerkleHashPath {
    /// Right (or false, or zero) means that the proof element's hash value is on the right side of the current hashing operation.
    /// While the accumulated hash is on the left side.
    Right = 0,
    /// Right (or true, or one) means that the proof element's hash value is on the left side of the current hashing operation.
    /// While the accumulated hash is on the right side.
    Left = 1,
}

impl From<MerkleHashPath> for bool {
    fn from(path: MerkleHashPath) -> bool {
        match path {
            MerkleHashPath::Left => true,
            MerkleHashPath::Right => false,
        }
    }
}

impl From<MerkleHashPath> for u8 {
    fn from(path: MerkleHashPath) -> u8 {
        match path {
            MerkleHashPath::Left => 1,
            MerkleHashPath::Right => 0,
        }
    }
}

impl<T, H> MerkleTree<T, H>
where
    H: PartialEq + Clone,
    T: Clone,
{
    /// Create a new Merkle Tree with the given leaves and hash functions.
    ///
    /// # Arguments
    ///
    /// - `leaves` - The leaves of the Merkle Tree containing the unhashed raw input data.
    /// - `node_hash_function` - The function used to hash two nodes together.
    /// - `leaf_hash_function` - The function used to hash a leaf.
    ///
    /// # Returns
    ///
    /// A new Merkle Tree instance.
    ///
    /// # Example
    ///
    /// ```
    /// use digital_voting::set_membership_zkp::merkle::MerkleTree;
    ///
    /// fn mock_hash(preimages: [u64; 2]) -> u64 {
    ///     preimages[0] ^ preimages[1]
    /// }
    ///
    /// let leaves = vec![1u64, 2u64, 3u64];
    /// let tree = MerkleTree::new(
    ///     &leaves,
    ///     Box::new(|a, b| mock_hash([*a, *b])),
    ///     Box::new(|x| mock_hash([*x, *x])),
    /// );
    /// ```
    ///
    /// # Errors
    ///
    /// If the Merkle Tree is empty.
    ///
    /// # Panics
    ///
    /// If the node_hash_function or leaf_hash_function panics.
    pub fn new(
        leaves: &[T],
        node_hash_function: NodeHashFn<H>,
        leaf_hash_function: LeafHashFn<T, H>,
    ) -> Result<Self> {
        if leaves.is_empty() {
            return Err(Error::EmptyTree);
        }

        let nodes = Vec::with_capacity(Self::precalc_node_count(leaves.len()));
        let mut new_tree = Self {
            leaf_count: leaves.len(),
            nodes,
            node_hash_function,
            leaf_hash_function,
        };
        new_tree.build_tree(leaves);
        Ok(new_tree)
    }

    /// Build the entire Merkle Tree.
    /// This function will hash the leaves and then the nodes to build the whole Merkle tree.
    fn build_tree(&mut self, leaves: &[T]) {
        let mut current_level: Vec<H> = leaves
            .iter()
            .map(|leaf| (self.leaf_hash_function)(leaf))
            .collect();
        while current_level.len() > 1 {
            let mut next_level = vec![];
            for chunk in current_level.chunks(2) {
                let hash = if chunk.len() == 2 {
                    (self.node_hash_function)(&chunk[0], &chunk[1])
                } else {
                    (self.node_hash_function)(&chunk[0], &chunk[0])
                };
                next_level.push(hash);
            }
            self.nodes.extend(current_level);
            current_level = next_level;
        }
        self.nodes.extend(current_level);
    }

    /// Precalculate the number of nodes in the Merkle Tree so that the nodes vector can be preallocated
    /// with the correct capacity to avoid reallocations.
    fn precalc_node_count(leaf_count: usize) -> usize {
        if leaf_count % 2 != 0 {
            (leaf_count + 1) * 2 - 1
        } else {
            leaf_count * 2 - 1
        }
    }

    /// Get the root of the Merkle Tree.
    ///
    /// # Returns
    ///
    /// The root of the Merkle Tree.
    ///
    /// # Panics
    ///
    /// If the Merkle Tree is empty, which should not happen.
    ///
    /// # Example
    ///
    /// ```
    /// use digital_voting::set_membership_zkp::merkle::MerkleTree;
    ///
    /// fn mock_hash(preimages: [u64; 2]) -> u64 {
    ///     preimages[0] ^ preimages[1]
    /// }
    ///
    /// let leaves = vec![1u64, 2u64, 3u64];
    /// let tree = MerkleTree::new(
    ///     &leaves,
    ///     Box::new(|a, b| mock_hash([*a, *b])),
    ///     Box::new(|x| mock_hash([*x, *x])),
    /// ).unwrap();
    /// let root = tree.get_root();
    /// ```
    pub fn get_root(&self) -> H {
        self.nodes.last().unwrap().clone()
    }

    /// Get the Merkle Proof for a leaf in the Merkle Tree.
    ///
    /// # Arguments
    ///
    /// - `leaf_index` - The index of the leaf for which the proof is generated.
    ///
    /// # Returns
    ///
    /// The Merkle Proof for the leaf.
    ///
    /// # Errors
    ///
    /// If the leaf index is out of bounds.
    ///
    /// # Example
    ///
    /// ```
    /// use digital_voting::set_membership_zkp::merkle::MerkleTree;
    ///
    /// fn mock_hash(preimages: [u64; 2]) -> u64 {
    ///     preimages[0] ^ preimages[1]
    /// }
    ///
    /// let leaves = vec![1u64, 2u64, 3u64];
    /// let tree = MerkleTree::new(
    ///     &leaves,
    ///     Box::new(|a, b| mock_hash([*a, *b])),
    ///     Box::new(|x| mock_hash([*x, *x])),
    /// ).unwrap();
    /// let proof = tree.get_proof(1).unwrap();
    /// ```
    pub fn get_proof(&self, leaf_index: usize) -> Result<MerkleProof<H>> {
        if leaf_index >= self.leaf_count {
            return Err(Error::ElementOutOfBounds(leaf_index, self.leaf_count));
        }
        let mut proof = MerkleProof {
            _leaf_index: leaf_index,
            root: self.nodes.last().unwrap().clone(),
            proof: vec![],
            path: vec![],
        };
        let mut cap = self.leaf_count;
        let mut current_level = 0;
        let mut current_index = leaf_index;
        while cap > 1 {
            let sibling_index = if current_index % 2 == 0 {
                // If the sibling is on the RIGHT side of the hash.
                proof.path.push(MerkleHashPath::Right);
                current_index + 1
            } else {
                // If the sibling is on the LEFT side of the hash.
                proof.path.push(MerkleHashPath::Left);
                current_index - 1
            };
            let sibling = if sibling_index < cap {
                self.nodes[current_level + sibling_index].clone()
            } else {
                // Last node without a pair, so returning itself.
                // Note that by default in this case, the path is true, but that shouldn't matter.
                self.nodes[current_level + current_index].clone()
            };
            proof.proof.push(sibling);
            current_index /= 2;
            current_level += cap;
            if cap % 2 == 1 {
                cap += 1;
            }
            cap /= 2;
        }

        Ok(proof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO add tests with more different data types.

    // A mock hasher to avoid having to link full blown hashers to this crate
    // just for testing.
    fn mock_hash(preimages: [u64; 2]) -> u64 {
        preimages[0] ^ preimages[1]
    }

    #[test]
    fn test_merkle_tree_empty() {
        let leaves: Vec<u64> = vec![];
        let tree = MerkleTree::new(
            &leaves,
            Box::new(|a, b| mock_hash([*a, *b])),
            Box::new(|x| mock_hash([*x, *x])),
        );
        assert!(tree.is_err());
    }

    #[test]
    fn test_merkle_tree_proof_out_of_bounds() {
        let leaves = vec![1u64, 2u64, 3u64];
        let tree = MerkleTree::new(
            &leaves,
            Box::new(|a, b| mock_hash([*a, *b])),
            Box::new(|x| mock_hash([*x, *x])),
        )
        .unwrap();
        assert!(tree.get_proof(leaves.len()).is_err());
    }

    #[test]
    fn test_merkle_tree() {
        let leaves = vec![1u64, 2u64, 3u64];
        let tree = MerkleTree::new(
            &leaves,
            Box::new(|a, b| mock_hash([*a, *b])),
            Box::new(|x| mock_hash([*x, *x])),
        )
        .unwrap();
        let root = tree.get_root();

        // Manually calculate all the hashes and the root.
        let hash_0 = mock_hash([leaves[0].into(), leaves[0].into()]);
        let hash_1 = mock_hash([leaves[1].into(), leaves[1].into()]);
        let hash_2 = mock_hash([leaves[2].into(), leaves[2].into()]);
        let hash_01 = mock_hash([hash_0, hash_1]);
        let hash_22 = mock_hash([hash_2, hash_2]);
        let calc_root = mock_hash([hash_01, hash_22]);

        let calc_proof = vec![
            vec![hash_1, hash_22],
            vec![hash_0, hash_22],
            vec![hash_2, hash_01],
        ];
        let calc_path = vec![
            vec![MerkleHashPath::Right, MerkleHashPath::Right],
            vec![MerkleHashPath::Left, MerkleHashPath::Right],
            vec![MerkleHashPath::Right, MerkleHashPath::Left],
        ];

        assert_eq!(root, calc_root);

        // Looping through all the leaves to ensure that no edge cases are missed.
        for leaf_index in 0..leaves.len() {
            let proof = tree.get_proof(leaf_index).unwrap();

            assert_eq!(calc_path[leaf_index], proof.path);
            assert_eq!(calc_proof[leaf_index], proof.proof);
            assert_eq!(proof.root, root);
            assert_eq!(leaf_index, proof._leaf_index);
        }
    }
}