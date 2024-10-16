/// Describes the strategy of updating the field.
#[derive(Clone, Copy, PartialEq)]
pub enum Topology {
    /// Bounds of the field are stitched together.
    Torus,
    /// Field is unbounded and can grow infinitely.
    Unbounded,
}
