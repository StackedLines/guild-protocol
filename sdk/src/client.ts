import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider, BN } from "@coral-xyz/anchor";
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import { agentPda, configPda, jobPda, vaultPda } from "./pdas";

/** Trust tiers, mirroring the on-chain enum. */
export type TrustLevel =
  | { probation: {} }
  | { standard: {} }
  | { trusted: {} }
  | { senior: {} };

export const Trust = {
  Probation: { probation: {} } as TrustLevel,
  Standard: { standard: {} } as TrustLevel,
  Trusted: { trusted: {} } as TrustLevel,
  Senior: { senior: {} } as TrustLevel,
};

/**
 * GuildClient — a thin, typed wrapper around the Guild Anchor program.
 * It hides PDA derivation and account wiring so an agent runtime can
 * post, claim, deliver, and review jobs with one call each.
 */
export class GuildClient {
  readonly program: Program;
  readonly programId: PublicKey;

  constructor(program: Program) {
    this.program = program;
    this.programId = program.programId;
  }

  /** Build a client from a connection + wallet keypair. */
  static fromKeypair(
    connection: Connection,
    idl: anchor.Idl,
    wallet: Keypair
  ): GuildClient {
    const provider = new AnchorProvider(
      connection,
      new anchor.Wallet(wallet),
      { commitment: "confirmed" }
    );
    const program = new Program(idl, provider);
    return new GuildClient(program);
  }

  config(): PublicKey {
    return configPda(this.programId)[0];
  }

  agent(owner: PublicKey): PublicKey {
    return agentPda(this.programId, owner)[0];
  }

  job(id: number | BN): PublicKey {
    return jobPda(this.programId, id)[0];
  }

  /** One-time protocol initialization. */
  async initialize(
    authority: Keypair,
    treasury: PublicKey,
    minBond: BN,
    feeBps: number,
    stakeMint: PublicKey = PublicKey.default
  ): Promise<string> {
    return this.program.methods
      .initialize(stakeMint, minBond, feeBps)
      .accounts({
        authority: authority.publicKey,
        config: this.config(),
        treasury,
        systemProgram: SystemProgram.programId,
      })
      .signers([authority])
      .rpc();
  }

  /** Register a new agent identity with a staked bond. */
  async registerAgent(
    owner: Keypair,
    handle: string,
    bond: BN
  ): Promise<string> {
    const agent = this.agent(owner.publicKey);
    return this.program.methods
      .registerAgent(handle, bond)
      .accounts({
        owner: owner.publicKey,
        config: this.config(),
        agent,
        vault: vaultPda(this.programId, agent)[0],
        systemProgram: SystemProgram.programId,
      })
      .signers([owner])
      .rpc();
  }

  /** Post a job, escrowing the bounty. Returns the job id and tx. */
  async postJob(
    poster: Keypair,
    bounty: BN,
    requiredStake: BN,
    minTrust: TrustLevel,
    specUri: string
  ): Promise<{ jobId: BN; tx: string }> {
    const cfg = await this.program.account.config.fetch(this.config());
    const jobId = cfg.jobCount as BN;
    const job = this.job(jobId);
    const tx = await this.program.methods
      .postJob(bounty, requiredStake, minTrust, specUri)
      .accounts({
        poster: poster.publicKey,
        config: this.config(),
        job,
        vault: vaultPda(this.programId, job)[0],
        systemProgram: SystemProgram.programId,
      })
      .signers([poster])
      .rpc();
    return { jobId, tx };
  }

  /** Claim an open job, locking the worker's stake. */
  async claimJob(owner: Keypair, jobId: BN): Promise<string> {
    return this.program.methods
      .claimJob()
      .accounts({
        owner: owner.publicKey,
        agent: this.agent(owner.publicKey),
        job: this.job(jobId),
      })
      .signers([owner])
      .rpc();
  }

  /** Submit delivered work for review. */
  async submitWork(
    owner: Keypair,
    jobId: BN,
    deliveryUri: string
  ): Promise<string> {
    return this.program.methods
      .submitWork(deliveryUri)
      .accounts({
        owner: owner.publicKey,
        agent: this.agent(owner.publicKey),
        job: this.job(jobId),
      })
      .signers([owner])
      .rpc();
  }

  /** Review a submitted job: approve (release) or reject (slash). */
  async reviewJob(
    reviewer: Keypair,
    jobId: BN,
    workerOwner: PublicKey,
    approve: boolean
  ): Promise<string> {
    const job = this.job(jobId);
    const worker = this.agent(workerOwner);
    const cfg = await this.program.account.config.fetch(this.config());
    return this.program.methods
      .reviewJob(approve)
      .accounts({
        reviewer: reviewer.publicKey,
        config: this.config(),
        job,
        jobVault: vaultPda(this.programId, job)[0],
        worker,
        workerVault: vaultPda(this.programId, worker)[0],
        workerWallet: workerOwner,
        posterWallet: reviewer.publicKey,
        treasury: cfg.treasury as PublicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([reviewer])
      .rpc();
  }

  async fetchAgent(owner: PublicKey) {
    return this.program.account.agentProfile.fetch(this.agent(owner));
  }

  async fetchJob(jobId: BN) {
    return this.program.account.job.fetch(this.job(jobId));
  }
}
