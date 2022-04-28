const { GearApi, GearKeyring, getWasmMetadata } = require("@gear-js/api");
const { readFileSync } = require('fs');

require('dotenv').config();

async function main() {
    const gearApi = await GearApi.create();
    const jsonKeyring = readFileSync('./account.json').toString();;
    const account = GearKeyring.fromJson(jsonKeyring, 'Google06!!');
    const metaFile = readFileSync(process.env.META_WASM);
    const meta = metaFile ? await getWasmMetadata(metaFile) : undefined;
    try {
        let somePayload = {
            Supply: {
                id: 0,
            }
        }
        const message = {
            destination: process.env.PROGRAM_ID, // programId
            payload: somePayload,
            gasLimit: 100000000,
            value: 1000,
        };
            // In that case payload will be encoded using meta.handle_input type
            await gearApi.message.submit(message, meta);
            // So if you want to use another type you can specify it
            await gearApi.message.submit(message, meta, meta.async_handle_input); // For example
        } catch (error) {
            console.error(`${error.name}: ${error.message}`);
        }
    try {
        await gearApi.message.signAndSend(account, (event) => {
            console.log("EVENT", event.toHuman());
        });
    } catch (error) {
        console.error(`${error.name}: ${error.message}`);
    }
}

main();
