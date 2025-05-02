use crate::{Pattern, Topology};
use anyhow::Result;
use num_bigint::BigInt;

/// Game engine for Game of Life
pub trait GoLEngine {
    /// Creates a new Game of Life engine instance with a blank pattern.
    ///
    /// This method initializes a Game of Life engine with an empty grid of the
    /// implementation's default size.
    ///
    /// # Returns
    /// A new instance of the Game of Life engine with a blank pattern
    fn new() -> Self
    where
        Self: Sized;

    /// Loads a pattern into the Game of Life engine with the specified topology.
    ///
    /// # Parameters
    /// * `pattern` - The cell configuration to load into the simulation
    /// * `topology` - The topology rules that define how the grid boundaries behave
    ///
    /// # Returns
    /// A Result indicating success or failure:
    /// * `Ok(())` - Pattern was successfully loaded
    /// * `Err(_)` - If loading fails (e.g., invalid pattern or unsupported topology)
    fn load_pattern(&mut self, pattern: &Pattern, topology: Topology) -> Result<()>;

    /// Returns the current state of the Game of Life field.
    ///
    /// This method retrieves the current configuration of cells in the grid
    /// and returns it as a Pattern structure, which can be used to save the state
    /// or initialize another Game of Life engine.
    ///
    /// # Returns
    /// A Pattern representing the current configuration of cells in the grid.
    fn current_state(&self) -> Pattern;

    /// Updates the Game of Life field by simulating multiple generations.
    ///
    /// This method advances the simulation by `2^generations_log2` generations.
    ///
    /// # Arguments
    ///
    /// * `generations_log2` - Power of 2 exponent determining number of generations to simulate
    ///
    /// # Returns
    ///
    /// An array `[dx, dy]` containing the coordinate shifts of the pattern's top-left corner.
    /// Only relevant for unbounded topologies where patterns can grow and move.
    /// For bounded topologies, returns `[0, 0]`.
    ///
    /// # Notes
    ///
    /// When using [`Topology::Unbounded`], the field size may grow to accommodate expanding patterns.
    fn update(&mut self, generations_log2: u32) -> [BigInt; 2];

    /// Runs garbage collection to free accumulated caches and temporary data.
    ///
    /// Some engine implementations may accumulate temporary data structures or caches
    /// during simulation. This method allows engines to free that memory when needed.
    ///
    /// # Note
    ///
    /// The default implementation does nothing. Engines should override this if they
    /// implement caching mechanisms.
    fn run_gc(&mut self) {}

    /// Returns the approximate heap memory usage of the engine in bytes.
    fn bytes_total(&self) -> usize;

    /// Returns multiline string reporting engine stats.
    fn statistics(&mut self) -> String {
        String::new()
    }
}
