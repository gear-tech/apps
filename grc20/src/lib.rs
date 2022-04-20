use std::collections::BTreeMap;

type Address = u128;
type Value = u128;

const ZERO: Address = 0;

pub trait GRC20 {
    fn transfer(to: Address, value: Value);
    fn transfer_from(from: Address, to: Address, value: Value);
}

#[derive(Debug, Default, Clone)]
struct GRC20State<I, S> {
    info: I,
    state: S,
    //total_supply: Value,
    //balances: BTreeMap<Address, Value>,
}



#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn mint() {
        assert!(true);
    }
}
