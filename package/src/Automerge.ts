import * as ffi from "./codegen";
import type * as AM from "@automerge/automerge/dist/wasm_types";
import { OBJECT_ID, STATE } from "@automerge/automerge/dist/mjs/constants";
import { pointerLiteralSymbol } from "uniffi-bindgen-react-native"

function transformValueRecord(value: ffi.ValueRecord): AM.PatchValue {
  if (value.valueType === "str") {
    return value.valueStr!;
  } else {
    throw new Error(`Value records of type '${value.valueType}' is not yet implemented`);
  }
}

function transformPatchAction({ path, action }: ffi.Patch): AM.Patch {
  if (action.tag === ffi.PatchAction_Tags.PutMap) {
    const { key, value, conflict } = action.inner;
    return {
      action: "put",
      path: [...path.map(segment => segment.prop.inner[0]), key],
      value: transformValueRecord(value),
      conflict,
    }
  } else {
    throw new Error(`Patching action '${action.tag}' is not yet implemented`);
  }
}

export class Automerge implements AM.Automerge {
  /** @hidden */
  constructor(private internal: ffi.AutomergeInterface) {}

  get __wbg_ptr(): bigint {
    return (this.internal as ffi.Automerge)[pointerLiteralSymbol];
  }

  put(obj: AM.ObjID, prop: AM.Prop, value: AM.Value, datatype?: AM.Datatype): void {
    if (typeof value !== "string" || datatype !== "str") {
      throw new Error("Putting anything other than a string is not yet implemented");
    }
    if (typeof prop !== "string") {
      throw new Error("Putting into sequences is not yet implemented");
    }
    this.internal.putStringToMap(obj, prop, value);
  }
  putObject(obj: AM.ObjID, prop: AM.Prop, value: AM.ObjType): AM.ObjID {
    throw new Error("Method not implemented.");
  }
  insert(obj: AM.ObjID, index: number, value: AM.Value, datatype?: AM.Datatype): void {
    throw new Error("Method not implemented.");
  }
  insertObject(obj: AM.ObjID, index: number, value: AM.ObjType): AM.ObjID {
    throw new Error("Method not implemented.");
  }
  push(obj: AM.ObjID, value: AM.Value, datatype?: AM.Datatype): void {
    throw new Error("Method not implemented.");
  }
  pushObject(obj: AM.ObjID, value: AM.ObjType): AM.ObjID {
    throw new Error("Method not implemented.");
  }
  splice(obj: AM.ObjID, start: number, delete_count: number, text?: string | Array<AM.Value>): void {
    throw new Error("Method not implemented.");
  }
  increment(obj: AM.ObjID, prop: AM.Prop, value: number): void {
    throw new Error("Method not implemented.");
  }
  delete(obj: AM.ObjID, prop: AM.Prop): void {
    throw new Error("Method not implemented.");
  }
  updateText(obj: AM.ObjID, newText: string): void {
    throw new Error("Method not implemented.");
  }
  updateSpans(obj: AM.ObjID, newSpans: AM.Span[]): void {
    throw new Error("Method not implemented.");
  }
  mark(obj: AM.ObjID, range: AM.MarkRange, name: string, value: AM.Value, datatype?: AM.Datatype): void {
    throw new Error("Method not implemented.");
  }
  unmark(obj: AM.ObjID, range: AM.MarkRange, name: string): void {
    throw new Error("Method not implemented.");
  }
  marks(obj: AM.ObjID, heads?: AM.Heads): AM.Mark[] {
    throw new Error("Method not implemented.");
  }
  marksAt(obj: AM.ObjID, index: number, heads?: AM.Heads): AM.MarkSet {
    throw new Error("Method not implemented.");
  }
  splitBlock(obj: AM.ObjID, index: number, block: { [key: string]: AM.MaterializeValue; }): void {
    throw new Error("Method not implemented.");
  }
  joinBlock(obj: AM.ObjID, index: number): void {
    throw new Error("Method not implemented.");
  }
  updateBlock(obj: AM.ObjID, index: number, block: { [key: string]: AM.MaterializeValue; }): void {
    throw new Error("Method not implemented.");
  }
  getBlock(obj: AM.ObjID, index: number): { [key: string]: AM.MaterializeValue; } | null {
    throw new Error("Method not implemented.");
  }
  diff(before: AM.Heads, after: AM.Heads): AM.Patch[] {
    throw new Error("Method not implemented.");
  }
  getCursor(obj: AM.ObjID, index: number, heads?: AM.Heads): AM.Cursor {
    throw new Error("Method not implemented.");
  }
  getCursorPosition(obj: AM.ObjID, cursor: AM.Cursor, heads?: AM.Heads): number {
    throw new Error("Method not implemented.");
  }
  isolate(heads: AM.Heads): void {
    this.internal.isolate(heads);
  }
  integrate(): void {
    this.internal.integrate();
  }
  get(obj: AM.ObjID, prop: AM.Prop, heads?: AM.Heads): AM.Value | undefined {
    throw new Error("Method not implemented.");
  }
  getWithType(obj: AM.ObjID, prop: AM.Prop, heads?: AM.Heads): AM.FullValue | null {
    // TODO: Support more return types than string
    if (typeof prop === "string") {
      const value = this.internal.getFromMapWithType(obj, prop, heads);
      if (value.getType() === "str") {
        return value ? ["str", value.getValueString()] : null;
      } else {
        throw new Error(`Unwrapping FullValue of type `)
      }
    } else if (typeof prop === "number") {
      const value = this.internal.getFromSeqWithType(obj, prop, heads) ?? null;
      if (value.getType() === "str") {
        return value ? ["str", value.getValueString()] : null;
      } else {
        throw new Error(`Unwrapping FullValue of type `)
      }
    } else {
      throw new Error(`Expected either a string or a number, got ${typeof prop}`);
    }
  }
  getAll(obj: AM.ObjID, arg: AM.Prop, heads?: AM.Heads): AM.FullValueWithId[] {
    throw new Error("Method not implemented.");
  }
  objInfo(obj: AM.ObjID, heads?: AM.Heads): AM.ObjInfo {
    throw new Error("Method not implemented.");
  }
  keys(obj: AM.ObjID, heads?: AM.Heads): string[] {
    return this.internal.keys(obj, heads);
  }
  text(obj: AM.ObjID, heads?: AM.Heads): string {
    throw new Error("Method not implemented.");
  }
  spans(obj: AM.ObjID, heads?: AM.Heads): AM.Span[] {
    throw new Error("Method not implemented.");
  }
  length(obj: AM.ObjID, heads?: AM.Heads): number {
    throw new Error("Method not implemented.");
  }
  materialize(obj?: AM.ObjID, heads?: AM.Heads, metadata?: unknown): AM.MaterializeValue {
    const values = this.internal.materialize(obj, heads);
    const result = Object.fromEntries(values.entries());
    // TODO: The following is handled in Rust code in the WASM binding
    Object.assign(result, {
      [STATE]: {
        handle: this,
      },
      [OBJECT_ID]: "_root",
    });
    return result;
  }
  toJS(): AM.MaterializeValue {
    throw new Error("Method not implemented.");
  }
  commit(message?: string, time?: number): AM.Hash | null {
    return this.internal.commit(message, time) ?? null;
  }
  emptyChange(message?: string, time?: number): AM.Hash {
    throw new Error("Method not implemented.");
  }
  merge(other: AM.Automerge): AM.Heads {
    throw new Error("Method not implemented.");
  }
  getActorId(): AM.Actor {
    throw new Error("Method not implemented.");
  }
  pendingOps(): number {
    return this.internal.pendingOps();
  }
  rollback(): number {
    return this.internal.rollback();
  }
  enableFreeze(enable: boolean): boolean {
    return this.internal.enableFreeze(enable);
  }
  registerDatatype(datatype: string, construct: Function, deconstruct: (arg: any) => any | undefined): void {
    // TODO: Implement or throw
    // throw new Error("Method not implemented.");
  }
  diffIncremental(): AM.Patch[] {
    throw new Error("Method not implemented.");
  }
  updateDiffCursor(): void {
    throw new Error("Method not implemented.");
  }
  resetDiffCursor(): void {
    throw new Error("Method not implemented.");
  }
  save(): Uint8Array {
    throw new Error("Method not implemented.");
  }
  saveNoCompress(): Uint8Array {
    throw new Error("Method not implemented.");
  }
  saveAndVerify(): Uint8Array {
    throw new Error("Method not implemented.");
  }
  saveIncremental(): Uint8Array {
    throw new Error("Method not implemented.");
  }
  saveSince(heads: AM.Heads): Uint8Array {
    throw new Error("Method not implemented.");
  }
  loadIncremental(data: Uint8Array): number {
    throw new Error("Method not implemented.");
  }
  receiveSyncMessage(state: AM.SyncState, message: AM.SyncMessage): void {
    throw new Error("Method not implemented.");
  }
  generateSyncMessage(state: AM.SyncState): AM.SyncMessage | null {
    throw new Error("Method not implemented.");
  }
  hasOurChanges(state: AM.SyncState): boolean {
    throw new Error("Method not implemented.");
  }
  applyChanges(changes: AM.Change[]): void {
    throw new Error("Method not implemented.");
  }
  getChanges(have_deps: AM.Heads): AM.Change[] {
    throw new Error("Method not implemented.");
  }
  getChangeByHash(hash: AM.Hash): AM.Change | null {
    throw new Error("Method not implemented.");
  }
  getDecodedChangeByHash(hash: AM.Hash): AM.DecodedChange | null {
    throw new Error("Method not implemented.");
  }
  getChangesAdded(other: AM.Automerge): AM.Change[] {
    throw new Error("Method not implemented.");
  }
  getHeads(): AM.Heads {
    return this.internal.getHeads();
  }
  getLastLocalChange(): AM.Change | null {
    throw new Error("Method not implemented.");
  }
  getMissingDeps(heads?: AM.Heads): AM.Heads {
    throw new Error("Method not implemented.");
  }
  free(): void {
    throw new Error("Method not implemented.");
  }
  clone(actor?: string): AM.Automerge {
    throw new Error("Method not implemented.");
  }
  fork(actor?: string, heads?: AM.Heads): AM.Automerge {
    throw new Error("Method not implemented.");
  }
  dump(): void {
    throw new Error("Method not implemented.");
  }
  applyPatches<Doc>(obj: Doc, meta?: unknown): Doc {
    throw new Error("Method not implemented.");
  }
  applyAndReturnPatches<Doc>(obj: Doc, meta?: unknown): { value: Doc; patches: AM.Patch[]; } {
    const patches = this.internal.diffIncremental().map(transformPatchAction);
    for (const patch of patches) {
      if (patch.action === "put") {
        // TODO: The path segments should be traversed instead of simply picking the last
        const key = patch.path[patch.path.length-1];
        (obj as Record<string, unknown>)[key] = patch.value;
      }
    }
    return { value: obj, patches };
  }
  topoHistoryTraversal(): AM.Hash[] {
    throw new Error("Method not implemented.");
  }
  stats(): { numChanges: number; numOps: number; } {
    throw new Error("Method not implemented.");
  }
}