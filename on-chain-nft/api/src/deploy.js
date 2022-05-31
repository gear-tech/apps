const { GearApi, GearKeyring, getWasmMetadata } = require('@gear-js/api');
const { readFileSync } = require('fs');

require('dotenv').config();

const uploadProgram = async (
    api,
    pathToProgram,
    pathToMeta,
    account,
    value,
    initPayload) => {
    const code = readFileSync(pathToProgram);
    const metaFile = pathToMeta ? readFileSync(pathToMeta) : undefined;
    const meta = metaFile ? await getWasmMetadata(metaFile) : undefined;    const gas = await api.program.gasSpent.init(
        account.publicKey,
        code,
        initPayload,
        value,
        meta
      );
    console.log("GAS SPENT", gas.toHuman());
    const programId = api.program.submit({ code, initPayload: initPayload, gasLimit: gas }, meta);
    await api.program.signAndSend(account, (data) => {
        console.log(data.toHuman());
    });
    return programId;
}

async function main() {
    const gearApi = await GearApi.create();
    const jsonKeyring = readFileSync('./account.json').toString();;
    const account = GearKeyring.fromJson(jsonKeyring, 'Google06!!');
    console.log("start deploying program");

    console.log(process.env.OPT_WASM)
    const payload = {
            name: "Cryptopunk OnChain",
            symbol: "COC",
            base_uri: "http://cryptopunk",
            base_image: '<svg version="1.0" xmlns="http://www.w3.org/2000/svg" width="24.000000pt" height="24.000000pt" viewBox="0 0 24.000000 24.000000" preserveAspectRatio="xMidYMid meet"><g transform="translate(0.000000,24.000000) scale(0.100000,-0.100000)" fill="#000000" stroke="none"><path d="M0 120 l0 -120 120 0 120 0 0 120 0 120 -120 0 -120 0 0 -120z"/></g></svg>',
            layers: {
                1: {
                    1: '<svg version="1.1" id="Layer_1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" x="0px" y="0px" width="24px" height="24px" viewBox="0 0 24 24" enable-background="new 0 0 24 24" xml:space="preserve">  <image id="image0" width="24" height="24" x="0" y="0"    href="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAYBAMAAAASWSDLAAAABGdBTUEAALGPC/xhBQAAACBjSFJNAAB6JQAAgIMAAPn/AACA6QAAdTAAAOpgAAA6mAAAF2+SX8VGAAAAElBMVEX///8AAADI+/t1vb2b4OD///9Pq9TLAAAAAXRSTlMAQObYZgAAAAFiS0dEAIgFHUgAAAAJcEhZcwAADsQAAA7EAZUrDhsAAAAHdElNRQfmBR8PFTdIB56gAAAAS0lEQVQY02NgoAYQFBQUgLEZlZSUFGEcISBHSYAkDqOQspCSIZQj5KTopCQigDAZSZmSkhOyAbg5yPYICgrhdAGSq5H8IwRXg8wBANtDC4kXpgUTAAAAJXRFWHRkYXRlOmNyZWF0ZQAyMDIyLTA1LTMxVDEyOjIxOjU1KzAzOjAwoyTgMgAAACV0RVh0ZGF0ZTptb2RpZnkAMjAyMi0wNS0zMVQxMjoyMTo1NSswMzowMNJ5WI4AAAAASUVORK5CYII=" /></svg>',
                    2: '<svg version="1.1" id="Layer_1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" x="0px" y="0px" width="24px" height="24px" viewBox="0 0 24 24" enable-background="new 0 0 24 24" xml:space="preserve">  <image id="image0" width="24" height="24" x="0" y="0"    href="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAYBAMAAAASWSDLAAAABGdBTUEAALGPC/xhBQAAACBjSFJNAAB6JQAAgIMAAPn/AACA6QAAdTAAAOpgAAA6mAAAF2+SX8VGAAAAFVBMVEX///8AAAB9ommbvIheclP/AAD////+ZzauAAAAAXRSTlMAQObYZgAAAAFiS0dEAIgFHUgAAAAJcEhZcwAADsQAAA7EAZUrDhsAAAAHdElNRQfmBR8PFTdIB56gAAAAUElEQVQY02NgoAYQFBQUgLEZlZSUFGEcIWUgDyYlZITMUcLJUXFSchGAGaYqpBQI56goKTnBOOgGCBJjtJIiHmUIVysJwSWAHEFBBgQHKg4A7YELsrmaQBAAAAAldEVYdGRhdGU6Y3JlYXRlADIwMjItMDUtMzFUMTI6MjE6NTUrMDM6MDCjJOAyAAAAJXRFWHRkYXRlOm1vZGlmeQAyMDIyLTA1LTMxVDEyOjIxOjU1KzAzOjAw0nlYjgAAAABJRU5ErkJggg==" /></svg>',
                },
                2: {
                    1: '<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"><path style="fill:#ffffff; stroke:none;" d="M0 0L0 24L24 24L24 0L0 0z"/><path style="fill:#f1f1f1; stroke:none;" d="M6 10C8.89605 14.2371 12.1755 13.9963 17 14L17 10L6 10z"/><path style="fill:#338dfd; stroke:none;" d="M9 11L9 13L12 13L12 11L9 11z"/><path style="fill:#fd3333; stroke:none;" d="M13 11L13 13L16 13L16 11L13 11z"/></svg>',
                    2: '<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"><path style="fill:#ffffff; stroke:none;" d="M0 0L0 24L24 24L24 0L0 0z"/><path style="fill:#010101; stroke:none;" d="M6 10C7.7608 15.8003 12.7948 14.9994 18 15L18 9L6 10z"/><path style="fill:#8d8d8d; stroke:none;" d="M8 10L8 14L9 14L8 10z"/><path style="fill:#b5b5b5; stroke:none;" d="M8 11L8 13L17 13L17 11L8 11z"/><path style="fill:#8d8d8d; stroke:none;" d="M16 10L17 11L16 10z"/><path style="fill:#010101; stroke:none;" d="M9 11L9 13L16 13L16 11L9 11z"/><path style="fill:#8d8d8d; stroke:none;" d="M16 13L17 14L16 13z"/></svg>',
                },
            },
            royalties: null,
    }
    let program = await uploadProgram(
        gearApi,
        process.env.OPT_WASM || "",
        process.env.META_WASM,
        account,
        0,
        payload
    )
    console.log("Hello Program ID:", program.programId);
}

main();