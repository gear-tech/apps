# Dutch auction

## Introduction
A Dutch auction is one of several types of auctions for buying or selling goods. Most commonly, it means an auction in which the auctioneer begins with a high asking price in the case of selling, and lowers it until some participant accepts the price, or it reaches a predetermined reserve price. A Dutch auction has also been called a clock auction or open-outcry descending-price auction. This type of auction shows the advantage of speed since a sale never requires more than one bid.

## Prerequisites
[Gear ERC-721](https://wiki.gear-tech.io/developing-contracts/examples/erc-721#erc-721-interface)

## Contract description
### Actions
```rust
pub enum Action {
    Buy,
    Create(CreateConfig),
}
```
- `Buy` is an action to buy an ERC-721 token by current price
- `Create(CreateConfig)` is an action to create a new auction if the previous one is over or if it's the first auction in this contract.
<br><br>
  Note how DutchAuction is composed; that allows users to reuse its functionality over and over again.

#### Structures in actions:
```rust
pub struct CreateConfig {
    pub nft_contract_actor_id: ActorId,
    pub token_owner: ActorId,
    pub token_id: U256,
    pub starting_price: U256,
    pub discount_rate: U256,
}
```
**To create a new auction you need to have this fields:**
- `nft_contract_actor_id` is a contract address where auctioneers NFT had been minted
- `token_owner` is an address of token owner to send him a reward if someone bought his NFT
- `token_id` is an id of NFT in its contract
- `starting_price` is the price at which the auction starts and starts descending
- `discount_rate` is the amount by which the price will decrease per millisecond over time

### State
*Requests:*
```rust
pub enum State {
    TokenPrice(),
    IsActive(),
    Info(),
}
```

- `TokenPrice` is a state to determine the current price of the NFT being sold
- `IsActive` is a state to determine if the auction has been ended
- `Info` is a state which describes an auction to show the user more information 

Each state request has a corresponding reply with the same name.

*Replies:*
```rust
pub enum StateReply {
    TokenPrice(U256),
    IsActive(bool),
    Info(AuctionInfo),
}
```
- `TokenPrice` has an associated value with current value in units
- `IsActive` has an associated value which indicate that auction hasn't been ended
- `Info` has an associated value of `AuctionInfo` type

#### Structures in state replies:

```rust
pub struct AuctionInfo {
    pub nft_contract_actor_id: ActorId,
    pub token_id: U256,
    pub token_owner: ActorId,
    pub starting_price: U256,
}
```

- `nft_contract_actor_id` is a contract address where auctioneers NFT had been minted
- `token_id` is an id of NFT in its contract
- `token_owner` is an address of token owner to send him a reward if someone bought his NFT
- `starting_price` is the price at which the auction starts and starts descending