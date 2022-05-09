use codec::{Decode, Encode};
use concert_io::*;
use gtest::{Program, System, WasmProgram};

#[derive(Debug)]
struct Multitoken;

impl WasmProgram for Multitoken {
    fn init(&mut self, _: Vec<u8>) -> Result<Option<Vec<u8>>, &'static str> {
        Ok(Some(b"INITIALIZED".to_vec()))
    }

    fn handle(&mut self, payload: Vec<u8>) -> Result<Option<Vec<u8>>, &'static str> {
        let res = MTKAction::decode(&mut payload[..]).map_err(|_| "Can not decode")?;
        match res {
            MTKAction::Mint {
                account,
                id,
                amount,
                meta,
            } => {
                return Ok(Some(
                    MTKEvent::TransferSingle(TransferSingleReply {
                        operator: msg::source(),
                        from: ZERO_ID,
                        to: *account,
                        id: *id,
                        amount,
                    })
                    .encode(),
                ));
            }
            MTKAction::MintBatch {
                account,
                ids,
                amounts,
                meta,
            } => {
                return Ok(Some(MTKEvent::TransferBatch {
                    operator: msg::source(),
                    from: ZERO_ID,
                    to: *account,
                    ids: ids.to_vec(),
                    values: amounts.to_vec(),
                }));
            }
            MTKAction::Burn { id, amount } => {
                return Ok(Some(
                    MTKEvent::TransferSingle(TransferSingleReply {
                        operator: msg::source(),
                        from: msg::source(),
                        to: ZERO_ID,
                        id: *id,
                        amount,
                    })
                    .encode(),
                ));
            }
            MTKAction::BalanceOfBatch { accounts, ids } => {
                return Ok(Some(MTKEvent::BalanceOfBatch(res).encode()))
            }
            _ => return Ok(None),
        }
    }

    fn handle_reply(&mut self, _: Vec<u8>) -> Result<Option<Vec<u8>>, &'static str> {
        Ok(None)
    }
}
