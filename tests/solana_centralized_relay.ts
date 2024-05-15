import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaCentralizedRelay } from "../target/types/solana_centralized_relay";
import { utf8 } from "@project-serum/anchor/dist/cjs/utils/bytes";
import { BN } from "bn.js";
import { assert, expect } from "chai";

describe("solana_centralized_relay", async () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace
    .SolanaCentralizedRelay as Program<SolanaCentralizedRelay>;

  const xcall = anchor.web3.Keypair.generate().publicKey;
  const admin = anchor.web3.Keypair.generate().publicKey;
  const conn_sn = 0;

  const [centralized_connection] = await anchor.web3.PublicKey.findProgramAddressSync(
    [
      utf8.encode("centralized_state"),
    ],
    program.programId
  );
  it("Is initialized!", async () => {
    const tx = await program.methods.initialize(admin, xcall).rpc();
  });

  it("Should get the right admin and xcall address", async () => {
    const centralized_connection_account = await program.account.centralizedConnectionState.fetch(centralized_connection);
      assert.equal(centralized_connection_account.xcallAddress.toString() , xcall.toString())
      assert.equal(centralized_connection_account.adminAddress.toString() , admin.toString())
      expect(centralized_connection_account.connSn.toNumber()).to.eql(0)
  });

  it("Should set new admin")
  it("reject setting new admin by non-admin")
  it("should get admin ")
  it("should get fee")
  it("should set fees")
  it("should send message")
  it("should receive receipt")

});
