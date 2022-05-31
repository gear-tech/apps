const { GearApi, CreateType, getWasmMetadata } = require("@gear-js/api");
const { readFileSync } = require('fs');

require('dotenv').config();

const events = async () => {
    const gearApi = await GearApi.create();
    const metaFile = readFileSync(process.env.META_WASM);
    const meta = metaFile ? await getWasmMetadata(metaFile) : undefined;
  gearApi.gearEvents.subscribeToLogEvents(({ data: { id, source, payload, reply } }) => {
    console.log(`
    Log:
      messageId: ${id.toHex()}
      from program: ${source.toHex()}
      payload: ${payload.toJSON()}
      ppayload: ${CreateType.create(meta.handle_output, payload, meta).toHuman()}
    ${
      reply.isSome
        ? `reply to: ${reply.unwrap()[0].toHex()}
      with error: ${reply.unwrap()[1].toNumber() === 0 ? false : true}
      `
        : ''
    }
    `);
    console.log(CreateType.create(meta.handle_output, payload, meta).toHuman())
  });

  gearApi.gearEvents.subscribeToProgramEvents(({ method, data: { info, reason } }) => {
    console.log(`
      ${method}:
      programId: ${info.programId.toHex()}
      initMessageId: ${info.messageId.toHex()}
      origin: ${info.origin.toHex()}
      ${reason ? `reason: ${reason.toHuman()}` : ''}
      `);
  });

};

async function main() {
    await events();
}

main();