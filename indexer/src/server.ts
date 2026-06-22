/**
 * Guild Indexer + REST API
 *
 * A lightweight read-only service that mirrors on-chain Guild state into
 * a simple in-memory cache and serves it over HTTP for the dashboard.
 * It does NOT hold keys or send transactions — it only reads the cluster.
 */
import express from "express";
import cors from "cors";
import {
  Connection,
  PublicKey,
} from "@solana/web3.js";
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import fs from "fs";
import path from "path";

const RPC_URL = process.env.RPC_URL || "https://api.devnet.solana.com";
const PORT = Number(process.env.PORT || 8080);
const IDL_PATH =
  process.env.IDL_PATH ||
  path.resolve(__dirname, "../../target/idl/guild.json");

const idl = JSON.parse(fs.readFileSync(IDL_PATH, "utf-8"));
const connection = new Connection(RPC_URL, "confirmed");

// Read-only provider: a dummy wallet, never used to sign.
const dummyWallet = {
  publicKey: PublicKey.default,
  signTransaction: async (t: any) => t,
  signAllTransactions: async (t: any) => t,
};
const provider = new AnchorProvider(connection, dummyWallet as any, {
  commitment: "confirmed",
});
const program = new Program(idl, provider);

interface Snapshot {
  updatedAt: number;
  config: any;
  agents: any[];
  jobs: any[];
}

let snapshot: Snapshot = {
  updatedAt: 0,
  config: null,
  agents: [],
  jobs: [],
};

function serialize(obj: any): any {
  return JSON.parse(
    JSON.stringify(obj, (_k, v) =>
      typeof v === "bigint"
        ? v.toString()
        : v && v.toBase58
        ? v.toBase58()
        : v && v.toNumber
        ? v.toNumber()
        : v
    )
  );
}

/** Poll the cluster and rebuild the snapshot. */
async function refresh() {
  try {
    const agents = await program.account.agentProfile.all();
    const jobs = await program.account.job.all();
    let config: any = null;
    try {
      const [configPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("config")],
        program.programId
      );
      config = await program.account.config.fetch(configPda);
    } catch {
      /* not initialized yet */
    }

    snapshot = {
      updatedAt: Date.now(),
      config: config ? serialize(config) : null,
      agents: agents.map((a) => ({
        address: a.publicKey.toBase58(),
        ...serialize(a.account),
      })),
      jobs: jobs
        .map((j) => ({ address: j.publicKey.toBase58(), ...serialize(j.account) }))
        .sort((a, b) => Number(b.id) - Number(a.id)),
    };
    console.log(
      `[indexer] refreshed: ${snapshot.agents.length} agents, ${snapshot.jobs.length} jobs`
    );
  } catch (err) {
    console.error("[indexer] refresh failed:", (err as Error).message);
  }
}

const app = express();
app.use(cors());

app.get("/health", (_req, res) => res.json({ ok: true, rpc: RPC_URL }));
app.get("/api/state", (_req, res) => res.json(snapshot));
app.get("/api/agents", (_req, res) => res.json(snapshot.agents));
app.get("/api/jobs", (_req, res) => res.json(snapshot.jobs));
app.get("/api/jobs/open", (_req, res) =>
  res.json(snapshot.jobs.filter((j) => j.status && "open" in j.status))
);

app.listen(PORT, () => {
  console.log(`[indexer] API on :${PORT}, program ${program.programId.toBase58()}`);
  refresh();
  setInterval(refresh, 10_000);
});
