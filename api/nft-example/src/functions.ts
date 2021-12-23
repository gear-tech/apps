import { GearApi } from '@gear-js/api';
import { sendMessage } from '../gear-utils/send-message';

require('dotenv').config();

export const mint = async (
  api: GearApi,
  account?: any,
  ) => {

  const payload = {
     Mint: ""
  }
  
  await sendMessage(
    api,
    process.env.NFT_ID,
    process.env.META_NFT,
    account,
    0,
    payload
  )
};

export const burn = async (
  api: GearApi,
  account?: any,
  token_id?: Number
  ) => {

  const payload = {
     Burn: token_id
  }

  await sendMessage(
    api,
    process.env.NFT_ID,
    process.env.META_NFT,
    account,
    0,
    payload
  )
};

