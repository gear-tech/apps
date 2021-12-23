
import { GearApi, GearKeyring, getWasmMetadata } from '@gear-js/api';
import { mint, burn } from './functions';
import { balance, tokenOwner } from './state';

async function  main () {
    const gearApi = await GearApi.create();
    const account = GearKeyring.fromMnemonic(process.env.MNEMONIC || "");
 
    await mint(gearApi, account)
    await mint(gearApi, account)     
    await burn(gearApi, account, 0)

    await balance(gearApi, account)
    await tokenOwner(gearApi, 2)
}

main()
  .catch((err) => {
    console.error(err);
  })
  .finally(() => { 
    process.exit(0)
  })