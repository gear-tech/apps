import { GearApi, getWasmMetadata } from '@gear-js/api';
import { readFileSync } from 'fs';

export const uploadProgram = async (
  api: GearApi, 
  pathToProgram: string, 
  pathToMeta?: string,
  account?: any,  
  initPayload?: any) => {
  const code = readFileSync(pathToProgram);
  const metaFile = pathToMeta ? readFileSync(pathToMeta) : undefined;
  const meta = metaFile ? await getWasmMetadata(metaFile) : undefined;

  const programId = api.program.submit({ code, initPayload, gasLimit: 1_000_000_000 }, meta);
  await api.program.signAndSend(account, (data) => {
    console.log(data);
  });
  return programId;
};