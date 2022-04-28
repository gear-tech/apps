const { GearApi } = require("@gear-js/api");

const events = async () => {
    const gearApi = await GearApi.create();
    const unsub = await gearApi.gearEvents.subscribeToLogEvents(({ data: { id, source, dest, payload, reply } }) => {
        console.log(`
        logId: ${id.toHex()}
        source: ${source.toHex()}
        payload: ${payload.toHuman()}
        `);
      });
      // Unsubscribe
    unsub();
}

async function main() {
    await events();
}

main();