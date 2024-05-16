import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaCentralizedRelay } from "../target/types/solana_centralized_relay";
import { utf8 } from "@project-serum/anchor/dist/cjs/utils/bytes";
import { BN } from "bn.js";
import { assert, expect } from "chai";
import { AnchorProvider } from "@project-serum/anchor";

describe("solana_centralized_relay", async () => {
anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace
    .SolanaCentralizedRelay as Program<SolanaCentralizedRelay>;
  const xcall = anchor.web3.Keypair.generate();
  const admin = anchor.web3.Keypair.generate();

const MESSAGE_FEE = 1;
const RESPONSE_FEE = 2;
  const CONN_SN = 0;
  const new_admin = anchor.web3.Keypair.generate();
  const NETWORK_ID = "0x2.icon";

  const [centralized_connection] =
     anchor.web3.PublicKey.findProgramAddressSync(
      [utf8.encode("centralized_state")],
      program.programId
    );

    const [fees] =
     anchor.web3.PublicKey.findProgramAddressSync(
      [utf8.encode("fees")],
      program.programId
    );

  it("Is initialized!", async () => {
    const tx = await program.methods
      .initialize(admin.publicKey, xcall.publicKey)
      .rpc();
  });

  it("Should get the right admin and xcall address via contract state", async () => {
    const centralized_connection_account =
      await program.account.centralizedConnectionState.fetch(
        centralized_connection
      );

      
    assert.equal(
      centralized_connection_account.xcallAddress.toString(),
      xcall.publicKey.toString()
    );
    assert.equal(
      centralized_connection_account.adminAddress.toString(),
      admin.publicKey.toString()
    );
    expect(centralized_connection_account.connSn.toNumber()).to.eql(CONN_SN);
  });

  it("Should set new admin", async () => {
    let airdropTx = await anchor.getProvider().connection.requestAirdrop(admin.publicKey, 100000000);
    await anchor.getProvider().connection.confirmTransaction(airdropTx)

    const tx = await program.methods
      .setAdmin(new_admin.publicKey)
      .accounts({
        user: admin.publicKey
      }).signers([admin])
      .rpc();

      
      const centralized_connection_account =
      await program.account.centralizedConnectionState.fetch(
        centralized_connection
        );

    assert.notEqual(centralized_connection_account.adminAddress.toString() , admin.publicKey.toString())
    assert.equal(
      centralized_connection_account.adminAddress.toString(),
      new_admin.publicKey.toString()
    );
  });

  it("reject updating new admin by non-admin", async () => {
    try {
      await program.methods
        .setAdmin(new_admin.publicKey)
        .accounts({
          user: admin.publicKey
        })
        .signers([admin])
        .rpc();

      // If the above line doesn't throw an error, fail the test
      throw new Error('Expected the function to fail, but it succeeded.');
    } catch (error) {
      // Assert that the error is the expected error
      expect(error.message).to.include('A require_keys_eq expression was violated');
    }


  });

  it("should get admin via function", async () => {
    const tx = await program.methods
      .getAdmin()
      .accounts({
        centralizedConnectionState: centralized_connection,
      })
      .rpc();

    const centralized_connection_account =
      await program.account.centralizedConnectionState.fetch(
        centralized_connection
      );
      const provider = anchor.getProvider();
  
  });



  it("should set fees", async () => {
    let airdropTx = await anchor.getProvider().connection.requestAirdrop(new_admin.publicKey, 100000000);
    await anchor.getProvider().connection.confirmTransaction(airdropTx)

    const tx = await program.methods
      .setFee(new anchor.BN(MESSAGE_FEE), new anchor.BN(RESPONSE_FEE), NETWORK_ID)
      .accounts({
        user: new_admin.publicKey, 
      })
      .signers([new_admin])
      .rpc();


  });

  it("should get fee", async () => {
    const tx = await program.methods
      .getFee(NETWORK_ID, true)
      .accounts({
      })
      .rpc();

      const fee_account_value =
      await program.account.feesState.fetch(
        fees
      );

      expect(fee_account_value.messageFees.toNumber()).to.equal(MESSAGE_FEE)
      expect(fee_account_value.responseFees.toNumber()).to.equal(RESPONSE_FEE)
  });

  
  it("should send message");
  it("should receive receipt");
});
