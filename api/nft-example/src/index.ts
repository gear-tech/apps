
import { GearApi, GearKeyring, getWasmMetadata, CreateType } from '@gear-js/api';
import { mint, burn, transfer } from './functions';
import { balance, tokenOwner } from './state';
import * as fs from 'fs';

require('dotenv').config();

async function  main () {
    const gearApi = await GearApi.create();
    const account_1 = GearKeyring.fromMnemonic(process.env.MNEMONIC1 as string);    
    const account_2 = GearKeyring.fromMnemonic(process.env.MNEMONIC2 as string);
    const account_3 = GearKeyring.fromMnemonic(process.env.MNEMONIC3 as string);

   
    await mint(gearApi, account_1)
    await transfer(gearApi, account_2, account_3, 1)
    await mint(gearApi, account_1)
    await mint(gearApi, account_2)     
    await burn(gearApi, account_1, 0)

    await balance(gearApi, account_1)
    await tokenOwner(gearApi, 1)
}

main()
  .catch((err) => {
    console.error(err);
  })
  .finally(() => { 
    process.exit(0)
  })