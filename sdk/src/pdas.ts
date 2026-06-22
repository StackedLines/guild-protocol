import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";

/** Seed prefixes — must match the on-chain program constants. */
export const CONFIG_SEED = Buffer.from("config");
export const AGENT_SEED = Buffer.from("agent");
export const JOB_SEED = Buffer.from("job");
export const VAULT_SEED = Buffer.from("vault");

/** Derive the singleton Config PDA. */
export function configPda(programId: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([CONFIG_SEED], programId);
}

/** Derive an agent profile PDA from its owner. */
export function agentPda(
  programId: PublicKey,
  owner: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [AGENT_SEED, owner.toBuffer()],
    programId
  );
}

/** Derive a job PDA from its sequential id. */
export function jobPda(
  programId: PublicKey,
  jobId: BN | number
): [PublicKey, number] {
  const id = BN.isBN(jobId) ? jobId : new BN(jobId);
  return PublicKey.findProgramAddressSync(
    [JOB_SEED, id.toArrayLike(Buffer, "le", 8)],
    programId
  );
}

/** Derive the lamport vault PDA owned by any account (agent or job). */
export function vaultPda(
  programId: PublicKey,
  owner: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [VAULT_SEED, owner.toBuffer()],
    programId
  );
}
