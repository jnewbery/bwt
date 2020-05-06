use std::str::FromStr;

use bitcoin::util::{base58, bip32::ExtendedPubKey};
use bitcoin::{Address, Network};
use bitcoin_hashes::Hash;

use crate::types::{ScriptHash, ScriptType};

pub fn address_to_scripthash(address: &Address) -> ScriptHash {
    ScriptHash::hash(&address.script_pubkey().into_bytes())
}

pub struct XyzPubKey {
    pub network: Network,
    pub script_type: ScriptType,
    pub extended_pubkey: ExtendedPubKey,
}

impl FromStr for XyzPubKey {
    type Err = base58::Error;

    fn from_str(inp: &str) -> Result<XyzPubKey, base58::Error> {
        let mut data = base58::from_check(inp)?;

        if data.len() != 78 {
            return Err(base58::Error::InvalidLength(data.len()));
        }

        // rust-bitcoin's bip32 implementation does not support ypubs/zpubs.
        // instead, figure out the network and script type ourselves and feed rust-bitcoin with a
        // modified faux xpub string that uses the regular p2pkh xpub version bytes it expects.
        //
        // NOTE: this does mean that the fingerprints will be computed using the fauxed version
        // bytes instead of the real ones. that's okay as long as the fingerprints as consistent
        // within pxt, but does mean that they will mismatch the fingerprints reported by other
        // software.

        let version = &data[0..4];
        let (network, script_type) = parse_xyz_version(version)?;
        data.splice(0..4, get_xpub_p2pkh_version(network).iter().cloned());

        let faux_xpub = base58::check_encode_slice(&data);
        let extended_pubkey = ExtendedPubKey::from_str(&faux_xpub)?;

        Ok(XyzPubKey {
            network,
            script_type,
            extended_pubkey,
        })
    }
}

impl XyzPubKey {
    pub fn matches_network(&self, network: Network) -> bool {
        self.network == network || (self.network == Network::Testnet && network == Network::Regtest)
    }
}

fn parse_xyz_version(version: &[u8]) -> Result<(Network, ScriptType), base58::Error> {
    Ok(match version {
        [0x04u8, 0x88, 0xB2, 0x1E] => (Network::Bitcoin, ScriptType::P2pkh),
        [0x04u8, 0xB2, 0x47, 0x46] => (Network::Bitcoin, ScriptType::P2wpkh),
        [0x04u8, 0x9D, 0x7C, 0xB2] => (Network::Bitcoin, ScriptType::P2shP2wpkh),

        [0x04u8, 0x35, 0x87, 0xCF] => (Network::Testnet, ScriptType::P2pkh),
        [0x04u8, 0x5F, 0x1C, 0xF6] => (Network::Testnet, ScriptType::P2wpkh),
        [0x04u8, 0x4A, 0x52, 0x62] => (Network::Testnet, ScriptType::P2shP2wpkh),

        _ => return Err(base58::Error::InvalidVersion(version.to_vec())),
    })
}

fn get_xpub_p2pkh_version(network: Network) -> [u8; 4] {
    match network {
        Network::Bitcoin => [0x04u8, 0x88, 0xB2, 0x1E],
        Network::Testnet | Network::Regtest => [0x04u8, 0x35, 0x87, 0xCF],
    }
}
