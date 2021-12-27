import { GearApi, getWasmMetadata, parseHexTypes } from '@gear-js/api';
import { readFileSync } from 'fs';

export const readState = async (
    api: GearApi,
    programId: any,
    pathToMeta?: string,
    payload?: any
) => {

    const metaFile = readFileSync(pathToMeta || "");
    const state = await api.programState.read(
        programId, 
        metaFile, 
        payload
        );
    return state.toHuman()
};