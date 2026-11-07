import { Buffer } from "buffer";
import { Address } from "@stellar/stellar-sdk";
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from "@stellar/stellar-sdk/contract";
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Timepoint,
  Duration,
} from "@stellar/stellar-sdk/contract";
export * from "@stellar/stellar-sdk";
export * as contract from "@stellar/stellar-sdk/contract";
export * as rpc from "@stellar/stellar-sdk/rpc";

if (typeof window !== "undefined") {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}


export const networks = {
  testnet: {
    networkPassphrase: "Test SDF Network ; September 2015",
    contractId: "CDOLW5XCTGHWGZWIDSBXGIUGSDKQOWILSCU7D6FNHV4NH2IGD73W5B3Q",
  }
} as const


export interface Badge {
  event_id: u32;
  issued_at: u64;
  issuer: string;
  kind: BadgeKind;
}

export const Errors = {
  1: {message:"AlreadyInitialized"},
  2: {message:"NotInitialized"},
  /**
   * Reservado (D-04): a falta de assinatura do admin falha na auth do host.
   */
  3: {message:"NotAdmin"},
  4: {message:"NotIssuer"},
  5: {message:"SelfIssue"},
  6: {message:"AlreadyHasBadge"},
  7: {message:"BadgeNotFound"}
}

export type DataKey = {tag: "Admin", values: void} | {tag: "Issuer", values: readonly [string]} | {tag: "Badge", values: readonly [string, BadgeKind, u32]} | {tag: "BadgeList", values: readonly [string]} | {tag: "Reputation", values: readonly [string]};

export type BadgeKind = {tag: "Participante", values: void} | {tag: "Mentor", values: void} | {tag: "Organizador", values: void} | {tag: "Palestrante", values: void};




export interface Client {
  /**
   * Construct and simulate a init transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Grava o admin. Só pode ser chamado uma vez (RN-01).
   */
  init: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a issue transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Emite (carimba) um badge. Exige a assinatura do emissor; o membro não assina.
   */
  issue: ({issuer, member, kind, event_id}: {issuer: string, member: string, kind: BadgeKind, event_id: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a revoke transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Revoga um badge. Só o admin. A remoção da lista é pelo VALOR (kind, event_id).
   */
  revoke: ({member, kind, event_id}: {member: string, kind: BadgeKind, event_id: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a badges_of transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Badges ativos do membro, na ordem de emissão.
   */
  badges_of: ({member}: {member: string}, options?: MethodOptions) => Promise<AssembledTransaction<Array<Badge>>>

  /**
   * Construct and simulate a reputation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Reputação on-chain do membro (0 se não tiver badges).
   */
  reputation: ({member}: {member: string}, options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a set_issuer transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Credencia (`true`) ou descredencia (`false`) um emissor. Só o admin.
   */
  set_issuer: ({issuer, enabled}: {issuer: string, enabled: boolean}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy(null, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAQAAAAAAAAAAAAAABUJhZGdlAAAAAAAABAAAAAAAAAAIZXZlbnRfaWQAAAAEAAAAAAAAAAlpc3N1ZWRfYXQAAAAAAAAGAAAAAAAAAAZpc3N1ZXIAAAAAABMAAAAAAAAABGtpbmQAAAfQAAAACUJhZGdlS2luZAAAAA==",
        "AAAABAAAAAAAAAAAAAAABUVycm9yAAAAAAAABwAAAAAAAAASQWxyZWFkeUluaXRpYWxpemVkAAAAAAABAAAAAAAAAA5Ob3RJbml0aWFsaXplZAAAAAAAAgAAAEdSZXNlcnZhZG8gKEQtMDQpOiBhIGZhbHRhIGRlIGFzc2luYXR1cmEgZG8gYWRtaW4gZmFsaGEgbmEgYXV0aCBkbyBob3N0LgAAAAAITm90QWRtaW4AAAADAAAAAAAAAAlOb3RJc3N1ZXIAAAAAAAAEAAAAAAAAAAlTZWxmSXNzdWUAAAAAAAAFAAAAAAAAAA9BbHJlYWR5SGFzQmFkZ2UAAAAABgAAAAAAAAANQmFkZ2VOb3RGb3VuZAAAAAAAAAc=",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAABQAAAAAAAAAAAAAABUFkbWluAAAAAAAAAQAAAAAAAAAGSXNzdWVyAAAAAAABAAAAEwAAAAEAAAAAAAAABUJhZGdlAAAAAAAAAwAAABMAAAfQAAAACUJhZGdlS2luZAAAAAAAAAQAAAABAAAAAAAAAAlCYWRnZUxpc3QAAAAAAAABAAAAEwAAAAEAAAAAAAAAClJlcHV0YXRpb24AAAAAAAEAAAAT",
        "AAAAAgAAAAAAAAAAAAAACUJhZGdlS2luZAAAAAAAAAQAAAAAAAAAAAAAAAxQYXJ0aWNpcGFudGUAAAAAAAAAAAAAAAZNZW50b3IAAAAAAAAAAAAAAAAAC09yZ2FuaXphZG9yAAAAAAAAAAAAAAAAC1BhbGVzdHJhbnRlAA==",
        "AAAABQAAAAAAAAAAAAAAC0lzc3VlZEV2ZW50AAAAAAEAAAAGaXNzdWVkAAAAAAAFAAAAAAAAAAZtZW1iZXIAAAAAABMAAAABAAAAAAAAAARraW5kAAAH0AAAAAlCYWRnZUtpbmQAAAAAAAAAAAAAAAAAAAhldmVudF9pZAAAAAQAAAAAAAAAAAAAAAZpc3N1ZXIAAAAAABMAAAAAAAAAAAAAAAlpc3N1ZWRfYXQAAAAAAAAGAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAAC0lzc3VlckV2ZW50AAAAAAEAAAAGaXNzdWVyAAAAAAACAAAAAAAAAAZpc3N1ZXIAAAAAABMAAAABAAAAAAAAAAdlbmFibGVkAAAAAAEAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAADFJldm9rZWRFdmVudAAAAAEAAAAHcmV2b2tlZAAAAAADAAAAAAAAAAZtZW1iZXIAAAAAABMAAAABAAAAAAAAAARraW5kAAAH0AAAAAlCYWRnZUtpbmQAAAAAAAAAAAAAAAAAAAhldmVudF9pZAAAAAQAAAAAAAAAAg==",
        "AAAAAAAAADRHcmF2YSBvIGFkbWluLiBTw7MgcG9kZSBzZXIgY2hhbWFkbyB1bWEgdmV6IChSTi0wMSkuAAAABGluaXQAAAABAAAAAAAAAAVhZG1pbgAAAAAAABMAAAABAAAD6QAAAAIAAAAD",
        "AAAAAAAAAE5FbWl0ZSAoY2FyaW1iYSkgdW0gYmFkZ2UuIEV4aWdlIGEgYXNzaW5hdHVyYSBkbyBlbWlzc29yOyBvIG1lbWJybyBuw6NvIGFzc2luYS4AAAAAAAVpc3N1ZQAAAAAAAAQAAAAAAAAABmlzc3VlcgAAAAAAEwAAAAAAAAAGbWVtYmVyAAAAAAATAAAAAAAAAARraW5kAAAH0AAAAAlCYWRnZUtpbmQAAAAAAAAAAAAACGV2ZW50X2lkAAAABAAAAAEAAAPpAAAAAgAAAAM=",
        "AAAAAAAAAFJSZXZvZ2EgdW0gYmFkZ2UuIFPDsyBvIGFkbWluLiBBIHJlbW/Dp8OjbyBkYSBsaXN0YSDDqSBwZWxvIFZBTE9SIChraW5kLCBldmVudF9pZCkuAAAAAAAGcmV2b2tlAAAAAAADAAAAAAAAAAZtZW1iZXIAAAAAABMAAAAAAAAABGtpbmQAAAfQAAAACUJhZGdlS2luZAAAAAAAAAAAAAAIZXZlbnRfaWQAAAAEAAAAAQAAA+kAAAACAAAAAw==",
        "AAAAAAAAAC5CYWRnZXMgYXRpdm9zIGRvIG1lbWJybywgbmEgb3JkZW0gZGUgZW1pc3PDo28uAAAAAAAJYmFkZ2VzX29mAAAAAAAAAQAAAAAAAAAGbWVtYmVyAAAAAAATAAAAAQAAA+oAAAfQAAAABUJhZGdlAAAA",
        "AAAAAAAAADhSZXB1dGHDp8OjbyBvbi1jaGFpbiBkbyBtZW1icm8gKDAgc2UgbsOjbyB0aXZlciBiYWRnZXMpLgAAAApyZXB1dGF0aW9uAAAAAAABAAAAAAAAAAZtZW1iZXIAAAAAABMAAAABAAAABA==",
        "AAAAAAAAAEVDcmVkZW5jaWEgKGB0cnVlYCkgb3UgZGVzY3JlZGVuY2lhIChgZmFsc2VgKSB1bSBlbWlzc29yLiBTw7MgbyBhZG1pbi4AAAAAAAAKc2V0X2lzc3VlcgAAAAAAAgAAAAAAAAAGaXNzdWVyAAAAAAATAAAAAAAAAAdlbmFibGVkAAAAAAEAAAABAAAD6QAAAAIAAAAD" ]),
      options
    )
  }
  public readonly fromJSON = {
    init: this.txFromJSON<Result<void>>,
        issue: this.txFromJSON<Result<void>>,
        revoke: this.txFromJSON<Result<void>>,
        badges_of: this.txFromJSON<Array<Badge>>,
        reputation: this.txFromJSON<u32>,
        set_issuer: this.txFromJSON<Result<void>>
  }
}