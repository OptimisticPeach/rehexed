//! # `rehexed`
//! This crate is meant to process the output of `hexasphere`'s
//! icosahedron subdivision (aka `IcoSphere`) into an adjacency
//! list for use in instances where hexagonal tiles are needed.
//!
//! Such examples include geometry generation, board algorithms
//! etc.
//!
//! # Usage
//! Generate an icosphere subdivision:
//! ```rs
//! use hexasphere::shapes::IcoSphere;
//!
//! let sphere = IcoSphere::new(12, |_| {});
//! ```
//! Accumulate its indices:
//! ```rs
//! let indices = sphere.get_all_indices();
//! ```
//! And then apply the one function:
//! ```rs
//! let adjacency_list = rehexed::rehexed(&indices, sphere.raw_points.len());
//! ```
//!

use std::fmt::Debug;

use arrayvec::ArrayVec;

/// Keeps track of which neighbours have been seen, and in what
/// configuration.
/// For reference, consider the following hexagon:
/// ```
///   a---b
///  /     \
/// f   i   c
///  \     /
///   e---d
/// ```
/// The resulting list for entry `i` should be some rotation of
/// `[a, b, c, d, e, f]`.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum RehexState {
    /// No neighbours have been seen.
    Empty,
    /// It is clear what the ordering is; there is no ambiguity.
    ///
    /// This indicates that the elements in the temporary list
    /// are ordered sequentially thus far. For example, `[a, b]`.
    Clear,
    /// There are two pairs of disjoint indices.
    /// For example, `[a, b, d, e]`.
    TwoTwo,
    /// There is a group of three, and a group of two, and their
    /// ordering is unknown.
    /// For example, suppose we had seen:
    /// `[a, b], [b, c], [d, e]`, then we would write down:
    /// `[a, b, c, d, e]`, but not know whether it should
    /// be ordered as such or otherwise as `[d, e, a, b, c]`.
    ThreeTwo,
    /// We've seen three disjoint pairs but do not know their
    /// relative ordering. For example, we could have seen:
    /// `[b, c], [f, a], [d, e]`, and not know how to order them
    /// nor join them.
    TwoTwoTwo,
    /// We are in a complete state, the loop is formed.
    Complete,
}

/// Turns the output of `hexasphere`'s subdivision algorithm for the
/// icosphere into an adjacency list of elements.
///
/// # Notes
/// - By default, hexasphere will have the first 12 points have only
///   5 neighbours and the rest have 6, however to support weird possibilities,
///   this code will emit `usize::MAX` in the last element of the 6-element
///   array if there are fewer than 6 neighbours.
/// - Winding is preserved by this code, so that mesh generation can
///   be correctly implemented.
pub fn rehexed(indices: &[u32], len: usize) -> Vec<[usize; 6]> {
    let mut state = std::iter::repeat(RehexState::Empty)
        .take(len)
        .collect::<Vec<_>>();
    let mut result = std::iter::repeat(ArrayVec::<usize, 6>::new())
        .take(len)
        .collect::<Vec<_>>();

    // Does the actual insertions.
    let mut insert = |a: u32, b: u32, c: u32| {
        let (a, b, c) = (a as usize, b as usize, c as usize);
        let state = &mut state[a];
        if let RehexState::Complete = state {
            return;
        }

        let result = &mut result[a];

        match state {
            RehexState::Empty => {
                result.extend([b, c]);
                *state = RehexState::Clear;
            }
            RehexState::Clear => {
                if result[result.len() - 1] == b {
                    if result[0] == c {
                        *state = RehexState::Complete;
                    } else {
                        result.push(c);
                        if result.len() == 6 {
                            *state = RehexState::Complete;
                        }
                    }
                } else if result[0] == c {
                    result.insert(0, b);
                    if result.len() == 6 {
                        *state = RehexState::Complete;
                    }
                } else {
                    *state = match result.len() {
                        2 => RehexState::TwoTwo,
                        3 => RehexState::ThreeTwo,
                        4 => RehexState::Complete,
                        _ => unreachable!(),
                    };
                    result.extend([b, c]);
                }
            }
            RehexState::TwoTwo => {
                if result[1] == b {
                    if result[2] == c {
                        *state = RehexState::Clear;
                    } else {
                        result.insert(2, c);
                        *state = RehexState::ThreeTwo;
                    }
                } else if result[0] == c {
                    if result[3] == b {
                        let temp = result[2];
                        result.pop();
                        result.pop();
                        result.insert(0, temp);
                        result.insert(1, b);
                        *state = RehexState::Clear;
                    } else {
                        result.insert(0, b);
                        *state = RehexState::ThreeTwo;
                    }
                } else if result[2] == c {
                    result.insert(0, b);
                    let t2 = result.swap_remove(2);
                    let t1 = result.swap_remove(1);
                    result.push(t1);
                    result.push(t2);
                    *state = RehexState::ThreeTwo;
                } else {
                    result.extend([b, c]);
                    *state = RehexState::TwoTwoTwo;
                }
            }
            RehexState::ThreeTwo => {
                if result[2] == b {
                    if result[3] == c {
                        *state = RehexState::Clear;
                    } else {
                        result.insert(3, c);
                        *state = RehexState::Complete;
                    }
                } else {
                    if result[4] == b {
                        result.pop();
                        let temp = result.pop().unwrap();
                        result.insert(0, b);
                        result.insert(0, temp);
                        *state = RehexState::Clear;
                    } else {
                        result.insert(0, b);
                        *state = RehexState::Complete;
                    }
                }
            }
            RehexState::TwoTwoTwo => {
                if (result[1] != b || result[2] != c)
                    && (result[3] != b || result[4] != c)
                    && (result[5] != b || result[0] != c)
                {
                    let t2 = result.swap_remove(3);
                    let t1 = result.swap_remove(2);
                    result.extend([t1, t2]);
                }
                *state = RehexState::Complete;
            }
            RehexState::Complete => unreachable!(),
        }
    };

    for chunk in indices.chunks_exact(3) {
        let &[a, b, c] = chunk else { unreachable!() };

        insert(a, b, c);
        insert(c, a, b);
        insert(b, c, a);
    }

    drop(insert);

    for (idx, around) in result.iter().enumerate() {
        if around.contains(&idx) {
            panic!("idx {} contains itself: {:?}", idx, around);
        }
    }

    result
        .into_iter()
        .map(|x| {
            let mut result = [usize::MAX; 6];
            result[..x.len()].copy_from_slice(&x);
            result
        })
        .collect()
}
