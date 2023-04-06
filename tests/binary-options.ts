import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { BinaryOptions } from "../target/types/binary_options";

describe("binary-options", () => {
  // Configure the client to use the local cluster.
  //anchor.setProvider(anchor.AnchorProvider.env());
  //let provider = anchor.AnchorProvider.local("http://127.0.0.1:8899")
  let provider = anchor.AnchorProvider.local("https://api.devnet.solana.com")

  const program = anchor.workspace.BinaryOptions as Program<BinaryOptions>;
  const admin_deposit_account = anchor.web3.Keypair.generate();
  const admin_auth = anchor.web3.Keypair.generate();
  const deposit_account = anchor.web3.Keypair.generate();
  const deposit_auth = anchor.web3.Keypair.generate(); // First participant
  const deposit_auth_2 = anchor.web3.Keypair.generate(); // Second participant
  const config = anchor.web3.Keypair.generate();
  const fs = require('fs');
  const assert = require("assert");

  let solToUSD = "J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix";
  var programKey;
  try {
      let data = fs.readFileSync(
          './tests/binary_options-keypair.json'
      );
      programKey = anchor.web3.Keypair.fromSecretKey(
          new Uint8Array(JSON.parse(data))
      );
  } catch (error) {
      //throw new Error("Please make sure the program key is binary_options-keypair.json.");
      throw error;
  }

  try {
    assert(program.programId.equals(programKey.publicKey));
  } catch (error) {
      throw new Error("Please make sure you have the same program address in Anchor.toml and binary_options-keypair.json");
  }

  // admin
  let [admin_pda_auth, admin_pda_bump] = anchor.web3.PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode("admin_auth"),
    admin_deposit_account.publicKey.toBuffer()
    ],
    program.programId);
  let [admin_sol_vault, admin_sol_bump] = anchor.web3.PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode("admin_sol_vault"),
    admin_pda_auth.toBuffer()
    ],
    program.programId);

  // depositer
  let [pda_auth, pda_bump] = anchor.web3.PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode("auth"),
    deposit_account.publicKey.toBuffer()
    ],
    program.programId);
    let [sol_vault, sol_bump] = anchor.web3.PublicKey.findProgramAddressSync(
      [anchor.utils.bytes.utf8.encode("sol_vault"),
      pda_auth.toBuffer()
      ],
      program.programId);

  before(async () => {

    let res = await provider.connection.requestAirdrop(admin_auth.publicKey, 100 * anchor.web3.LAMPORTS_PER_SOL);

    let latestBlockHash = await provider.connection.getLatestBlockhash()

    await provider.connection.confirmTransaction({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: res,
    });

  });

  before(async () => {

    let res = await provider.connection.requestAirdrop(deposit_auth.publicKey, 100 * anchor.web3.LAMPORTS_PER_SOL);

    let latestBlockHash = await provider.connection.getLatestBlockhash()

    await provider.connection.confirmTransaction({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: res,
    });

  });

  before(async () => {

    let res = await provider.connection.requestAirdrop(deposit_auth_2.publicKey, 100 * anchor.web3.LAMPORTS_PER_SOL);

    let latestBlockHash = await provider.connection.getLatestBlockhash()

    await provider.connection.confirmTransaction({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: res,
    });

  });

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize(
      {
        priceFeedId: new anchor.web3.PublicKey(solToUSD)
      }
    )
      .accounts({
        program: program.programId,
        config: config.publicKey,
        adminDepositAccount: admin_deposit_account.publicKey,
        adminPdaAuth: admin_pda_auth,
        adminSolVault: admin_sol_vault,
        adminAuth: admin_auth.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      }).signers([programKey, config, admin_deposit_account, admin_auth]).rpc();
    console.log("Your transaction signature", tx);

    let result = await program.account.depositBaseAdmin.fetch(admin_deposit_account.publicKey);
    console.log(result);
  });
  
  it("Create Binary Options", async () => {
    // Add your test here.
    /*
    A - Crypto Asset (eg SOL)
    P - Position (This can either be a long or short position)
    S - Strike Price (This is the predicted future price of Crypto Asset eg $35)
    B - Bet Amount (This is the amount that the creator(first participant) of the bet will give away if the lose eg 10 SOL)
    T - Taker amount (This is the amount that the second participant of the bet will give away if the lose eg 5 SOL)

    bet description = 'A:SOL~P:LONG~S:$35~B:10SOL~T:5SOL';
    */
    let betDescription: string = 'A:SOL~P:LONG~S:$35~B:10SOL~T:5SOL';
    let betAmount = new anchor.BN(10 * anchor.web3.LAMPORTS_PER_SOL);
    let strikePrice = new anchor.BN(35); // SOL price
    let takerAmount = new anchor.BN(5 * anchor.web3.LAMPORTS_PER_SOL);
    let participantPosition = { long: {} };

    const tx = await program.methods.createBinaryOptions(betDescription, betAmount, strikePrice, takerAmount, participantPosition)
      .accounts({
        depositAccount: deposit_account.publicKey,
        pdaAuth: pda_auth,
        solVault: sol_vault,
        depositAuth: deposit_auth.publicKey,
        adminDepositAccount: admin_deposit_account.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      }).signers([deposit_account, deposit_auth]).rpc();
    console.log("Your transaction signature", tx);

    let result = await program.account.binaryOption.fetch(deposit_account.publicKey);
    console.log(result);
  });
  
  it("Accept Binary Options", async () => {
    // Add your test here.
    let amount = new anchor.BN(5 * anchor.web3.LAMPORTS_PER_SOL);
    let participantPosition = { short: {} };

    const tx = await program.methods.acceptBinaryOptions(amount, participantPosition)
      .accounts({
        adminDepositAccount: admin_deposit_account.publicKey,
        adminPdaAuth: admin_pda_auth,
        adminSolVault: admin_sol_vault,
        depositAccount: deposit_account.publicKey,
        pdaAuth: pda_auth,
        solVault: sol_vault,
        depositAuth: deposit_auth_2.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      }).signers([deposit_auth_2]).rpc();
    console.log("Your transaction signature", tx);

    let result = await program.account.binaryOption.fetch(deposit_account.publicKey);
    console.log(result);
  });

  it("Process Prediction", async () => {
    // Add your test here.
    let betFees = new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL);

    const tx = await program.methods.processPrediction(betFees)
      .accounts({
        config: config.publicKey,
        pythPriceFeedAccount: new anchor.web3.PublicKey(solToUSD),
        depositAccount: deposit_account.publicKey,
        pdaAuth: pda_auth,
        solVault: sol_vault,
        adminDepositAccount: admin_deposit_account.publicKey,
        adminPdaAuth: admin_pda_auth,
        adminSolVault: admin_sol_vault,
        systemProgram: anchor.web3.SystemProgram.programId,
      }).signers([]).rpc();
    console.log("Your transaction signature", tx);

    let result = await program.account.binaryOption.fetch(deposit_account.publicKey);
    console.log(result);
  });

  it("Withdraw Participant Funds", async () => {
    // Add your test here.
    let amount = new anchor.BN(5 * anchor.web3.LAMPORTS_PER_SOL);

    const tx = await program.methods.withdrawParticipantFunds(amount)
      .accounts({
        depositAccount: deposit_account.publicKey,
        pdaAuth: pda_auth,
        solVault: sol_vault,
        depositAuth: deposit_auth.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      }).signers([deposit_auth]).rpc();
    console.log("Your transaction signature", tx);

    let result = await program.account.binaryOption.fetch(deposit_account.publicKey);
    console.log(result);
  });

  it("Withdraw", async () => {
    // Add your test here.
    let amount = new anchor.BN(2 * anchor.web3.LAMPORTS_PER_SOL);

    const tx = await program.methods.withdraw(amount)
      .accounts({
        adminDepositAccount: admin_deposit_account.publicKey,
        adminPdaAuth: admin_pda_auth,
        adminSolVault: admin_sol_vault,
        adminAuth: admin_auth.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      }).signers([admin_auth]).rpc();
    console.log("Your transaction signature", tx);

    let result = await program.account.binaryOption.fetch(admin_deposit_account.publicKey);
    console.log(result);
  });

});
