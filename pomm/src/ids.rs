pub mod pyth_program {
    use solana_program::declare_id;
    #[cfg(feature = "mainnet-beta")]
    declare_id!("FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH");
    #[cfg(not(feature = "mainnet-beta"))]
    declare_id!("gSbePebfvPy7tRqimPoVecS2UsBvYv46ynrzWocc92s");
}

pub mod phoenix_onchain_mm_program {
    use solana_program::declare_id;
    // #[cfg(feature = "mainnet-beta")]
    // declare_id!("FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH");
    #[cfg(not(feature = "mainnet-beta"))]
    declare_id!("Be2ydETBafXycLHCGgPcW4VCwoexmbdectPG1Wh2Xihk");
}

pub mod bonk_oracle {
    use solana_program::declare_id;
    #[cfg(feature = "mainnet-beta")]
    declare_id!("8ihFLu5FimgTQ1Unh4dVyEHUGodJ5gJQCrQf4KUVB9bN");
    #[cfg(not(feature = "mainnet-beta"))]
    declare_id!("6bquU99ktV1VRiHDr8gMhDFt3kMfhCQo5nfNrg2Urvsn");
}

pub mod sol_oracle {
    use solana_program::declare_id;
    #[cfg(feature = "mainnet-beta")]
    declare_id!("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG");
    #[cfg(not(feature = "mainnet-beta"))]
    declare_id!("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix");
}

pub mod usdc_oracle {
    use solana_program::declare_id;
    #[cfg(feature = "mainnet-beta")]
    declare_id!("Gnt27xtC473ZT2Mw5u8wZ68Z3gULkSTb5DuxJy7eJotD");
    #[cfg(not(feature = "mainnet-beta"))]
    declare_id!("5SSkXsEKQepHHAewytPVwdej4epN1nxgLVM84L4KXgy7");
}

pub mod eth_oracle {
    use solana_program::declare_id;
    #[cfg(feature = "mainnet-beta")]
    declare_id!("JBu1AL4obBcCMqKBBxhpWCNUt136ijcuMZLFvTP7iWdB");
    #[cfg(not(feature = "mainnet-beta"))]
    declare_id!("EdVCmQ9FSPcVe5YySXDPCRmc8aDQLKJ9xvYBMZPie1Vw");
}

pub mod msol_oracle {
    use solana_program::declare_id;
    #[cfg(feature = "mainnet-beta")]
    declare_id!("E4v1BBgoso9s64TQvmyownAVJbhbEPGyzA3qn4n46qj9");
    #[cfg(not(feature = "mainnet-beta"))]
    declare_id!("9a6RNx3tCu1TSs6TBSfV2XRXEPEZXQ6WB7jRojZRvyeZ");
}

pub mod jitosol_oracle {
    use solana_program::declare_id;
    #[cfg(feature = "mainnet-beta")]
    declare_id!("7yyaeuJ1GGtVBLT2z2xub5ZWYKaNhF28mj1RdV4VDFVk");
    #[cfg(not(feature = "mainnet-beta"))]
    declare_id!("3d4eLK2TF6UdpSjKvS5ZUnDY1uZq2sEj9Tk3cujpUaAk");
}
