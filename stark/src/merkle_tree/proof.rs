use super::{Commitment, Error, Hash, Hashable, Index, Node, Result};
use itertools::Itertools;
use std::collections::VecDeque;

// Note: we can merge and split proofs. Based on indices we can
// compute which values are redundant.
#[derive(Clone, Debug)]
pub struct Proof {
    commitment: Commitment,
    indices:    Vec<usize>,
    hashes:     Vec<Hash>,
}

impl Proof {
    pub fn from_hashes(
        commitment: &Commitment,
        indices: &[usize],
        hashes: &[Hash],
    ) -> Result<Self> {
        // Validate indices using `sort_indices`
        let _ = commitment.sort_indices(indices)?;
        if hashes.len() != commitment.proof_size(indices)? {
            return Err(Error::NotEnoughHashes);
        }
        Ok(Self {
            commitment: commitment.clone(),
            indices:    indices.to_vec(),
            hashes:     hashes.to_vec(),
        })
    }

    pub fn hashes(&self) -> &[Hash] {
        &self.hashes
    }

    pub fn verify<Leaf: Hashable>(&self, leafs: &[(usize, &Leaf)]) -> Result<()> {
        // TODO: Check if the indices line up.

        // Construct the leaf nodes
        let mut nodes: Vec<_> = leafs
            .iter()
            .map(|(index, leaf)| {
                (
                    Index::from_depth_offset(self.commitment.depth(), *index)
                        .expect("Index out of range."),
                    leaf.hash(),
                )
            })
            .collect();
        nodes.sort_unstable_by_key(|(index, _)| *index);
        // OPT: `tuple_windows` copies the hashes
        if nodes
            .iter()
            .tuple_windows()
            .any(|(a, b)| a.0 == b.0 && a.1 != b.1)
        {
            return Err(Error::DuplicateLeafMismatch);
        }
        nodes.dedup_by_key(|(index, _)| *index);
        let mut nodes: VecDeque<(Index, Hash)> = nodes.into_iter().collect();

        // Create a mutable closure to pop hashes from the list
        let mut hashes_iter = self.hashes.iter();
        let mut pop = move || hashes_iter.next().ok_or(Error::NotEnoughHashes);

        // Reconstruct the root
        while let Some((current, hash)) = nodes.pop_front() {
            if let Some(parent) = current.parent() {
                // Reconstruct the parent node
                let node = if current.is_left() {
                    if let Some((next, next_hash)) = nodes.front() {
                        // TODO: Find a better way to satisfy the borrow checker.
                        let next_hash = next_hash.clone();
                        if current.sibling().unwrap() == *next {
                            // Merge left with next
                            let _ = nodes.pop_front();
                            Node(&hash, &next_hash).hash()
                        } else {
                            // Left not merged with next
                            // TODO: Find a way to merge this branch with the next.
                            Node(&hash, pop()?).hash()
                        }
                    } else {
                        // Left not merged with next
                        Node(&hash, pop()?).hash()
                    }
                } else {
                    // Right not merged with previous (or we would have skipped)
                    Node(pop()?, &hash).hash()
                };
                // Queue the new parent node for the next iteration
                nodes.push_back((parent, node))
            } else {
                // Root node has no parent, we are done
                if hash != *self.commitment.hash() {
                    return Err(Error::RootHashMismatch);
                }
            }
        }
        Ok(())
    }
}
