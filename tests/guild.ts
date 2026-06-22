import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { assert } from "chai";
import { GuildClient, Trust } from "../sdk/src";

describe("guild", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.guild as Program;
  const client = new GuildClient(program);

  const authority = (provider.wallet as anchor.Wallet).payer;
  const treasury = Keypair.generate().publicKey;
  const poster = Keypair.generate();
  const worker = Keypair.generate();

  const MIN_BOND = new BN(0.1 * LAMPORTS_PER_SOL);
  const FEE_BPS = 250;

  before(async () => {
    for (const kp of [poster, worker]) {
      const sig = await provider.connection.requestAirdrop(
        kp.publicKey,
        2 * LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(sig);
    }
  });

  it("initializes the protocol", async () => {
    await client.initialize(authority, treasury, MIN_BOND, FEE_BPS);
    const cfg = await program.account.config.fetch(client.config());
    assert.equal(cfg.feeBps, FEE_BPS);
    assert.ok((cfg.minBond as BN).eq(MIN_BOND));
  });

  it("registers a worker agent with a bond", async () => {
    await client.registerAgent(worker, "builder-bot", MIN_BOND);
    const agent = await client.fetchAgent(worker.publicKey);
    assert.equal(agent.handle, "builder-bot");
    assert.ok((agent.bond as BN).eq(MIN_BOND));
    assert.deepEqual(agent.trust, Trust.Probation);
  });

  it("happy path: post -> claim -> submit -> accept", async () => {
    const bounty = new BN(0.05 * LAMPORTS_PER_SOL);
    const stake = new BN(0.02 * LAMPORTS_PER_SOL);
    const { jobId } = await client.postJob(
      poster, bounty, stake, Trust.Probation, "ipfs://spec-1"
    );

    await client.claimJob(worker, jobId);
    assert.deepEqual((await client.fetchJob(jobId)).status, { claimed: {} });

    await client.submitWork(worker, jobId, "ipfs://delivery-1");
    assert.deepEqual((await client.fetchJob(jobId)).status, { submitted: {} });

    const before = await provider.connection.getBalance(worker.publicKey);
    await client.reviewJob(poster, jobId, worker.publicKey, true);
    const after = await provider.connection.getBalance(worker.publicKey);

    assert.deepEqual((await client.fetchJob(jobId)).status, { accepted: {} });
    assert.isAbove(after, before, "worker received the payout");

    const agent = await client.fetchAgent(worker.publicKey);
    assert.equal((agent.jobsCompleted as BN).toNumber(), 1);
    assert.isAbove((agent.reputation as BN).toNumber(), 0);
  });

  it("slashes on rejection", async () => {
    const bounty = new BN(0.03 * LAMPORTS_PER_SOL);
    const stake = new BN(0.02 * LAMPORTS_PER_SOL);
    const { jobId } = await client.postJob(
      poster, bounty, stake, Trust.Probation, "ipfs://spec-2"
    );
    await client.claimJob(worker, jobId);
    await client.submitWork(worker, jobId, "ipfs://bad");

    const a0 = await client.fetchAgent(worker.publicKey);
    await client.reviewJob(poster, jobId, worker.publicKey, false);
    const a1 = await client.fetchAgent(worker.publicKey);

    assert.deepEqual((await client.fetchJob(jobId)).status, { rejected: {} });
    assert.equal((a1.jobsSlashed as BN).toNumber(), 1);
    assert.ok((a1.bond as BN).lt(a0.bond as BN), "bond was slashed");
  });
});
