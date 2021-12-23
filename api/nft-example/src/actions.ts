import { GearApi, GearKeyring, getWasmMetadata } from '@gear-js/api';
import { sendMessage } from '../gear-utils/send-message';
import { readState } from '../gear-utils/read-state';

require('dotenv').config();

export const mint = async (
  api: GearApi,
  account?: any,
  ) => {

  const mintPayload = {
     Mint: ""
  }
  await sendMessage(
    api,
    process.env.NFT_ID,
    process.env.META_NFT,
    account,
    0,
    mintPayload
  )
};

async function main() {
  //const gearApi = await GearApi.create({providerAddress: "wss://rpc-node.gear-tech.io:443"});
  const gearApi = await GearApi.create();
  const account = GearKeyring.fromMnemonic(process.env.MNEMONIC || "");
  console.log(account);
  
  const mintPayload = {
     Mint: ""
  }
  
  await sendMessage(
    gearApi,
    process.env.NFT_ID,
    process.env.META_NFT,
    account,
    0,
    mintPayload
  )

  const queryBalance = {
    balanceOfUser: GearKeyring.decodeAddress(account.address)
  }
  
  await readState(
    gearApi,
    process.env.NFT_ID,
    process.env.META_NFT,
    queryBalance
  )

  const queryTokenOwner = {
    TokenOwner: 10
  }
  
  await readState(
    gearApi,
    process.env.NFT_ID,
    process.env.META_NFT,
    queryTokenOwner
  )

  const queryIsTokenOwner = {
    isTokenOwner: {
      tokenId: 17,
      user: GearKeyring.decodeAddress(account.address)
    }
  }

  await readState(
    gearApi,
    process.env.NFT_ID,
    process.env.META_NFT,
    queryIsTokenOwner
  )

  const queryGetApproved = {
    getApproved: 1
  }

  await readState(
    gearApi,
    process.env.NFT_ID,
    process.env.META_NFT,
    queryGetApproved
  )

}




main()