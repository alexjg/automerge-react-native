import { use, InitOptions } from "@automerge/automerge/dist/mjs/entrypoints/slim";
import * as ffi from './codegen';
import { Automerge } from "./Automerge";

use({
  create<T>({ actor, freeze, enableTextV2, unchecked, allowMissingChanges, convertRawStringsToText, patchCallback }: InitOptions<T> = {}) {
    if (patchCallback) {
      throw new Error("Passing 'patchCallback' is not yet supported");
    }
    return new Automerge(ffi.create({ actor, freeze, enableTextV2, unchecked, allowMissingChanges, convertRawStringsToText }));
  },
  load(data, options) {
    throw new Error("Not yet implemented");
  },
  encodeChange(change) {
    throw new Error("Not yet implemented");
  },
  decodeChange(change) {
    throw new Error("Not yet implemented");
  },
  initSyncState() {
    throw new Error("Not yet implemented");
  },
  encodeSyncMessage(message) {
    throw new Error("Not yet implemented");
  },
  decodeSyncMessage(msg) {
    throw new Error("Not yet implemented");
  },
  encodeSyncState(state) {
    throw new Error("Not yet implemented");
  },
  decodeSyncState(data) {
    throw new Error("Not yet implemented");
  },
  exportSyncState(state) {
    throw new Error("Not yet implemented");
  },
  importSyncState(state) {
    throw new Error("Not yet implemented");
  }
})
