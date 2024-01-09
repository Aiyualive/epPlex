use crate::*;

// should be derived from authority and collection name
pub const SEED_COLLECTION_CONFIG: &[u8] = b"CONFIG";

#[account]
#[derive(Default, Debug)]
pub struct CollectionConfig {
    /// The bump, used for PDA validation.
    pub bump: u8,
    pub authority: Pubkey,
    // This should be denoted in USDC
    pub renewal_price: u64,
    pub standard_duration: u32,
    pub grace_period: i64,
    pub treasury: Pubkey,
    pub collection_size: u32,
    pub collection_name: String,
}

impl CollectionConfig {
    pub const LEN: usize = DISCRIMINATOR_LENGTH
        + BITS_8
        + PUBLIC_KEY_LENGTH
        + BITS_64
        + BITS_64
        + BITS_32
        + PUBLIC_KEY_LENGTH
        + (VEC_PREFIX + BITS_8 * COLLECTION_NAME_LENGTH);

    // TODO need to account for dynamic length in collection name
    pub fn new(bump: u8, params: CollectionCreateParams) -> Self {
        Self {
            bump,
            authority: params.authority,
            renewal_price: params.renewal_price,
            standard_duration: params.standard_duration,
            grace_period: params.grace_period,
            treasury: params.treasury,
            collection_size: params.collection_size,
            collection_name: params.collection_name,
        }
    }

}