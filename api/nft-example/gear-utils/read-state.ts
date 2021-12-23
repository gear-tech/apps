import { GearApi, getWasmMetadata, parseHexTypes } from '@gear-js/api';
import { readFileSync } from 'fs';

export const readState = async (
    api: GearApi,
    programId: any,
    pathToMeta?: string,
    payload?: any
) => {

    const metaFile = readFileSync(pathToMeta || "");
    const meta = await getWasmMetadata(metaFile) 
  //  console.log(parseHexTypes(meta.types as string))
    const state = await api.programState.read(
        programId, 
        metaFile, 
        payload
        );
    console.log(state.toHuman());
};