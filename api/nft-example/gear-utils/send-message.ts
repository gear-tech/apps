import { GearApi, getWasmMetadata } from '@gear-js/api';
import { readFileSync } from 'fs';

export const sendMessage = async (
  api: GearApi,
  destination: any, 
  pathToMeta?: string,
  account?: any,
  value?: any,  
  payload?: any) => {
      console.log(payload);
      
    try {
        const message = {
            destination: destination, // programId
            payload,
            gasLimit: 1000000000,
            value
        };
    const metaFile = pathToMeta ? readFileSync(pathToMeta) : undefined;
    const meta = metaFile ? await getWasmMetadata(metaFile) : undefined;
    await api.message.submit(message, meta);
    } catch (error: any) {
        console.error(`${error.name}: ${error.message}`);
    }
    try {
        await api.message.signAndSend(account, (data) => {
            //console.log(data);
        });
    } catch (error: any) {
        console.error(`${error.name}: ${error.message}`);
    }
};

