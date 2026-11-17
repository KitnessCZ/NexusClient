/// A numeric identifier for a block in the NexusClient engine.
///
/// These IDs are engine-internal for now. Later, the Minecraft asset
/// and registry loader will map Minecraft block states to these IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BlockId(pub u16);

impl BlockId {
    pub const AIR: Self = Self(0);
    pub const STONE: Self = Self(1);
    pub const DIRT: Self = Self(2);
    pub const GRASS_BLOCK: Self = Self(3);
    pub const BEDROCK: Self = Self(4);

    pub const fn is_air(self) -> bool {
        self.0 == Self::AIR.0
    }
}