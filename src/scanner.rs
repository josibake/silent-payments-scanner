use bitcoin::consensus::deserialize;
use bitcoin::hashes::Hash;
use bitcoin::{PrivateKey, XOnlyPublicKey};
use libbitcoinkernel_sys::ChainstateManager;
use log::info;
use rayon::prelude::*;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use silentpayments::receiving::{Label, Receiver};
use silentpayments::utils::receiving::{
    calculate_shared_secret, calculate_tweak_data, get_pubkey_from_input,
};
use std::fmt;
use std::str::FromStr;

pub fn vec_to_hex_string(data: &[u8]) -> String {
    let mut hex_string = String::with_capacity(data.len() * 2);
    for byte in data {
        hex_string.push_str(&format!("{:02x}", byte));
    }
    hex_string
}

#[derive(Debug, Clone)]
struct Input {
    prevout: Vec<u8>,
    script_sig: Vec<u8>,
    witness: Vec<Vec<u8>>,
    prevout_data: (Vec<u8>, u32),
}

#[derive(Debug, Clone)]
struct ScanTxHelper {
    ins: Vec<Input>,
    outs: Vec<Vec<u8>>,
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "prevout: {}, ", vec_to_hex_string(&self.prevout))?;
        write!(f, "script_sig: {}, ", vec_to_hex_string(&self.script_sig))?;
        for witness_elem in &self.witness {
            write!(f, "witness: {}, ", vec_to_hex_string(&witness_elem))?;
        }
        write!(
            f,
            "prevout txid: {}, ",
            bitcoin::Txid::from_slice(&self.prevout_data.0).unwrap()
        )?;
        write!(f, "prevout n: {}, ", self.prevout_data.1)
    }
}

impl fmt::Display for ScanTxHelper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for input in &self.ins {
            write!(f, "input: {}\n", input)?;
        }
        for output in &self.outs {
            write!(f, "output: {}\n", vec_to_hex_string(&output))?;
        }
        write!(f, "")
    }
}

pub fn parse_keys(scan_key: String, spend_pub_key: String) -> (Receiver, SecretKey) {
    let scan_key = PrivateKey::from_wif(&scan_key).unwrap();
    let public_spend_key = PublicKey::from_str(&spend_pub_key).unwrap();

    let secp = Secp256k1::new();
    let public_scan_key: PublicKey = scan_key.public_key(&secp).inner;

    let label = Label::new(scan_key.inner, 0);
    let receiver = Receiver::new(0, public_scan_key, public_spend_key, label, false).unwrap();
    (receiver, scan_key.inner)
}

fn scan_tx(receiver: &Receiver, secret_scan_key: &SecretKey, scan_tx_helper: &ScanTxHelper) {
    let input_pub_keys: Vec<PublicKey> = scan_tx_helper
        .ins
        .iter()
        .filter_map(|input| {
            get_pubkey_from_input(&input.script_sig, &input.witness, &input.prevout).unwrap()
        })
        .collect();
    if input_pub_keys.is_empty() {
        return;
    }
    let pubkeys_ref: Vec<&PublicKey> = input_pub_keys.iter().collect();
    let outpoints_data: Vec<_> = scan_tx_helper
        .ins
        .iter()
        .map(|input| {
            let txid = bitcoin::Txid::from_slice(&input.prevout_data.0)
                .unwrap()
                .to_string();
            (txid, input.prevout_data.1)
        })
        .collect();
    let tweak_data = match calculate_tweak_data(&pubkeys_ref, &outpoints_data) {
        Ok(data) => data,
        Err(e) => {
            println!("Error calculating tweak data: {:?}", e);
            return;
        }
    };
    let ecdh_shared_secret = calculate_shared_secret(tweak_data, *secret_scan_key).unwrap();
    let pubkeys_to_check: Vec<XOnlyPublicKey> = scan_tx_helper
        .outs
        .iter()
        .filter_map(|script_pubkey| {
            if script_pubkey.len() < 2 {
                return None;
            }
            XOnlyPublicKey::from_slice(&script_pubkey[2..]).ok()
        })
        .collect();
    if pubkeys_to_check.is_empty() {
        return;
    }
    if let Ok(res) = receiver.scan_transaction(&ecdh_shared_secret, pubkeys_to_check) {
        if !res.is_empty() {
            println!("\nres: {:?}\n", res);
        }
    }
}

struct TransactionData {
    transaction_undo_size: u64,
    transaction_input_size: u64,
    scan_tx_helper: ScanTxHelper,
}

pub fn scan_txs(chainman: &ChainstateManager, receiver: &Receiver, secret_scan_key: &SecretKey) {
    let mut block_index_res = chainman.get_block_index_tip();
    let mut block_counter = 0;

    while let Ok(ref block_index) = block_index_res {
        let undo = chainman.read_undo_data(&block_index).unwrap();
        let raw_block: Vec<u8> = chainman.read_block_data(&block_index).unwrap().into();
        let block: bitcoin::Block = deserialize(&raw_block).unwrap();
        assert_eq!(block.txdata.len() - 1, undo.n_tx_undo);

        let transactions_data: Vec<TransactionData> = (0..block.txdata.len() - 1)
            .map(|i| {
                let transaction_undo_size: u64 = undo
                    .get_get_transaction_undo_size(i.try_into().unwrap())
                    .unwrap();
                let transaction_input_size: u64 =
                    block.txdata[i + 1].input.len().try_into().unwrap();
                assert_eq!(transaction_input_size, transaction_undo_size);

                let scan_tx_helper = ScanTxHelper {
                    ins: (0..transaction_input_size)
                        .map(|j| Input {
                            prevout: undo
                                .get_prevout_by_index(i as u64, j)
                                .unwrap()
                                .script_pubkey,
                            script_sig: block.txdata[i + 1].input[j as usize].script_sig.to_bytes(),
                            witness: block.txdata[i + 1].input[j as usize].witness.to_vec(),
                            prevout_data: (
                                block.txdata[i + 1].input[j as usize]
                                    .previous_output
                                    .txid
                                    .to_byte_array()
                                    .to_vec(),
                                block.txdata[i + 1].input[j as usize].previous_output.vout,
                            ),
                        })
                        .collect(),
                    outs: block.txdata[i + 1]
                        .output
                        .iter()
                        .map(|output| output.script_pubkey.to_bytes())
                        .collect(),
                };

                TransactionData {
                    transaction_undo_size,
                    transaction_input_size,
                    scan_tx_helper,
                }
            })
            .collect();

        transactions_data.par_iter().for_each(|data| {
            assert_eq!(data.transaction_input_size, data.transaction_undo_size);
            scan_tx(&receiver, &secret_scan_key, &data.scan_tx_helper);
        });

        block_index_res = block_index_res.unwrap().prev();
        block_counter += 1;
        if block_counter % 5000 == 0 {
            info!("Processed block number: {}", block_counter);
        }
    }
    log::info!("scanned txs!");
}
