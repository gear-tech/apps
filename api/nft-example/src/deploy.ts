import { GearApi, GearKeyring, getWasmMetadata } from '@gear-js/api';
import { uploadProgram } from '../gear-utils/upload-program';

require('dotenv').config();

async function deploy() {
 // const gearApi = await GearApi.create({providerAddress: "wss://rpc-node.gear-tech.io:443"});
  const gearApi = await GearApi.create();
  const account = GearKeyring.fromMnemonic(process.env.MNEMONIC1 || "");
 
  console.log("start deploying NFT");

  const initPayloadNFT= {
    name: "My NFT Token",
    symbol: "NFT",
    base_uri: "http://",
  }
  
  const nft_program_id = await uploadProgram(
    gearApi,
    process.env.PROGRAM_NFT || "",
    process.env.META_NFT,
    account,
    initPayloadNFT
  )

  console.log("NFT Program ID:", nft_program_id);

}

deploy()
  .catch((err) => {
    console.error(err);
  })
  .finally(() => { 
    process.exit(0)
  })
