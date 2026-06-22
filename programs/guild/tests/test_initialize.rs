use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;

/// Smoke test: the compiled program loads into LiteSVM and the
/// funded payer has the expected balance. Full end-to-end coverage
/// of the instruction set lives in the TypeScript test-suite
/// (`tests/guild.ts`), which exercises the complete job lifecycle.
#[test]
fn test_program_loads() {
    let program_id = guild::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/guild.so");
    svm.add_program(program_id, bytes);
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    let balance = svm.get_balance(&payer.pubkey()).unwrap();
    assert_eq!(balance, 1_000_000_000);
}
