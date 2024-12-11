import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Pump } from "../target/types/pump";
import { Keypair, PublicKey } from "@solana/web3.js";
import keys from '../keys/users.json'
import key2 from '../keys/user2.json'
import { createMint, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress } from "@solana/spl-token";
import { expect } from "chai";
import { BN } from "bn.js";

const curveSeed = "CurveConfiguration"
const POOL_SEED_PREFIX = "liquidity_pool"
const LIQUIDITY_SEED = "LiqudityProvider"
const SOL_VAULT_PREFIX = "liquidity_sol_vault"

describe("pump", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider);
  const connection = provider.connection

  const program = anchor.workspace.Pump as Program<Pump>;

  const user1 = Keypair.fromSecretKey(new Uint8Array(keys))
  const user2 = Keypair.fromSecretKey(new Uint8Array(key2))
  const tokenDecimal = 9
  const amount = new BN(1000000000).mul(new BN(10 ** tokenDecimal))

  let tokenMint: PublicKey
  let tokenMintAta: PublicKey
  let exchangeTokenMint: PublicKey
  let exchangeTokenAta: PublicKey

  console.log("Admin's wallet address is : ", user1.publicKey.toBase58())
  console.log('progrma id =============> ', program.programId)

  it("Mint token to user wallet", async () => {
    console.log("Trying to create and mint token1 to user's wallet")

    try {
      tokenMint = await createMint(connection, user1, user1.publicKey, user1.publicKey, tokenDecimal)
      console.log('tokenMint address: ', tokenMint.toBase58());
      tokenMintAta = (await getOrCreateAssociatedTokenAccount(connection, user1, tokenMint, user1.publicKey)).address
      console.log('token1 account address: ', tokenMintAta.toBase58());

      exchangeTokenMint = await createMint(connection, user2, user2.publicKey, user2.publicKey, tokenDecimal)
      console.log('exchangeTokenMint address: ', exchangeTokenMint.toBase58());
      exchangeTokenAta = (await getOrCreateAssociatedTokenAccount(connection, user2, exchangeTokenMint, user2.publicKey)).address
      console.log('token2 account address: ', exchangeTokenAta.toBase58());

      
      
      try {
        //minting 100 new tokens to the token address we just created
        await mintTo(connection, user1, tokenMint, tokenMintAta, user1.publicKey, BigInt(amount.toString()))
        await mintTo(connection, user2, exchangeTokenMint, exchangeTokenAta, user2.publicKey, BigInt(amount.toString()))

      } catch (error) {
        console.log("🚀 ~ here:", error)
      }
      const token1Balance = await connection.getTokenAccountBalance(tokenMintAta)
      const token2Balance = await connection.getTokenAccountBalance(exchangeTokenAta)

      console.log("tokenBalance1 in user:", token1Balance.value.uiAmount)
      console.log("tokenBalance2 in user:", token2Balance.value.uiAmount)

      console.log('tokens successfully minted');
    } catch (error) {
      console.log("Token creation error \n", error)
    }

  })

  it("Create a liquidity pool", async () => {
    // const tokenMint = "CF9M1BCdQf8YKXCCKu8ix2XG8XU9VckEou1q2mjk8sYb"
    // const tokenAt = "EP9v99QiXTy32vKdh1KzDeSkDxwc6hyQhXVqyEWkPpMs"
    // const exchangeTokenMint ="9NKrYCT9utmmmvHBVLsYhHZfAm6ssLihm5KxC7QjV5se"
    // const exchangeTAC = "27XNLTPrzRG3EtYuDcFigPph7oWHuATeRF6YreoVLuRs"

    const [pool, bump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("CurveConfiguration"),  // Make sure this matches your POOL_SEED_PREFIX
        tokenMint.toBuffer(),
        exchangeTokenMint.toBuffer(),
      ],
      program.programId
    );

    const poolTokenAccount = await getAssociatedTokenAddress(
      tokenMint,
      pool,
      true // allowOwnerOffCurve
    );

    const poolExchangeTokenAccount = await getAssociatedTokenAddress(
      exchangeTokenMint,
      pool,
      true // allowOwnerOffCurve
    );

    try {
      // Create pool
      const tx = await program.methods
        .createPool()
        .accounts({
          pool,
          tokenMint,
          exchangeTokenMint,
          poolTokenAccount,
          poolExchangeTokenAccount,
          payer: provider.wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      // Fetch the created pool account
      const poolAccount = await program.account.liquidityPool.fetch(pool);

      // Verify the pool data
      expect(poolAccount.creator.toString()).to.equal(
        provider.wallet.publicKey.toString()
      );
      expect(poolAccount.token.toString()).to.equal(
        tokenMint.toString()
      );
      expect(poolAccount.exchangeToken.toString()).to.equal(
        exchangeTokenMint.toString()
      );
      expect(poolAccount.bump).to.equal(bump);

      // Verify token accounts were created
      const tokenAccount = await provider.connection.getAccountInfo(poolTokenAccount);
      const exchangeTokenAccount = await provider.connection.getAccountInfo(poolExchangeTokenAccount);
      
      expect(tokenAccount).to.not.be.null;
      expect(exchangeTokenAccount).to.not.be.null;

    } catch (err) {
      console.error("Error:", err);
      throw err;
    }
  })

});
