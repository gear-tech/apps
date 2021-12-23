import { GearApi, GearKeyring, getWasmMetadata } from '@gear-js/api';
import { readState } from '../gear-utils/read-state';

require('dotenv').config();

export const balance = async (
  api: GearApi,
  account?: any,
  ) => {
    const payload = {
        balanceOfUser: GearKeyring.decodeAddress(account.address)
    }
    console.log(payload);
    
    await readState(
        api,
        process.env.NFT_ID,
        process.env.META_NFT,
        payload
      ) 
    };

export const tokenOwner = async (
    api: GearApi,
    token_id?: Number,
    ) => {
        const payload = {
            TokenOwner: token_id
          }
          
        await readState(
            api,
            process.env.NFT_ID,
            process.env.META_NFT,
            payload
        )
       
    };

export const IsTokenOwner = async (
    api: GearApi,
    account?: any,
    token_id?: Number,
    ) => {
        const payload = {
            tokenId: 17,
            user: GearKeyring.decodeAddress(account.address)
        }
            
        await readState(
            api,
            process.env.NFT_ID,
            process.env.META_NFT,
            payload
        )    
    };

export const getApproved = async (
    api: GearApi,
    token_id?: Number,
    ) => {
        const payload = {
            getApproved: token_id
        }
            
        await readState(
            api,
            process.env.NFT_ID,
            process.env.META_NFT,
            payload
        )    
    };

