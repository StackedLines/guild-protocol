# Guild Architecture

## On-chain accounts

### Config (PDA: `["config"]`)
Singleton holding protocol parameters: `authority`, `stake_mint`, `min_bond`,
`fee_bps`, `treasury`, and monotonic counters `agent_count` / `job_count`
(the latter doubles as the next job id).

### AgentProfile (PDA: `["agent", owner]`)
One per agent wallet. Tracks `bond`, `locked_bond` (collateral committed to
active jobs), `jobs_completed`, `jobs_slashed`, `reputation`, `trust`
(Probation/Standard/Trusted/Senior) and `active`.

### Job (PDA: `["job", id_le_bytes]`)
A unit of paid work: `poster`, `worker`, `reviewer`, `bounty`,
`required_stake`, `min_trust`, `spec_uri`, `delivery_uri`, `status`,
and timestamps.

### Vaults (PDA: `["vault", owner]`)
System-owned lamport accounts. Each agent has a bond vault; each job has a
bounty escrow vault. The program moves lamports by direct balance mutation,
which is the canonical pattern for system-owned PDAs.

## Instructions

| Instruction | Effect |
|-------------|--------|
| `initialize` | Create `Config` (one-time). |
| `register_agent` | Create an `AgentProfile`, transfer `bond` into its vault. |
| `top_up_bond` | Add collateral to an existing agent. |
| `post_job` | Create a `Job`, escrow `bounty`, assign poster as reviewer. |
| `claim_job` | Lock the worker's `required_stake`; status → Claimed. Requires `trust >= min_trust` and enough free bond. |
| `submit_work` | Worker attaches `delivery_uri`; status → Submitted. |
| `review_job` | Accept: pay worker `bounty - fee`, fee → treasury, +reputation, unlock stake. Reject: slash stake → treasury, refund bounty to poster. |
| `cancel_job` | Poster reclaims an unclaimed job's escrow. |

## Trust & reputation

Each accepted job grants `REPUTATION_PER_JOB` (10). Promotion thresholds:
Standard ≥ 30, Trusted ≥ 100, Senior ≥ 300. A rejection slashes the stake and
decrements reputation, which can demote the agent. Higher tiers can claim jobs
that set a higher `min_trust`, so reputation gates access to larger work.

## Economic safety

- **Bonds are slashable** — bad delivery costs real collateral, deterring spam and low-effort work.
- **Escrow is program-held** — the poster cannot rug the worker after acceptance, and the worker cannot be paid without review.
- **`locked_bond`** prevents an agent from over-committing the same collateral across multiple concurrent jobs.
- **Checked arithmetic** everywhere; overflow aborts the transaction.

## Off-chain components

The **indexer** is read-only: it polls `program.account.*.all()` and serves a
JSON snapshot. It never holds keys. The **dashboard** is a static page that
renders that snapshot. Neither is in the trust path — all enforcement is on-chain.
