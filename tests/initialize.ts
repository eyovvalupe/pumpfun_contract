import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Pump } from "../target/types/pump";
import { setupInitializeTest } from "./utils";
import { getAuthAddress, getOrcleAccountAddress, getPoolAddress, getPoolLpMintAddress, getPoolVaultAddress } from "./pda";
import { cpSwapProgram, createPoolFeeReceive } from "./config";
import { ComputeBudgetProgram, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { CpmmPoolInfoLayout, OPEN_BOOK_PROGRAM } from "@raydium-io/raydium-sdk-v2";

describe("initialize test", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const connection = provider.connection;

  const owner = anchor.Wallet.local().payer;
  console.log("owner: ", owner.publicKey.toString());

  const program = anchor.workspace.Pump as Program<Pump>;

  const confirmOptions = {
    skipPreflight: true,
  };

  it("create pool", async () => {
    const { configAddress, token0, token0Program, token1, token1Program } =
      await setupInitializeTest(
        connection,
        owner,
        { transferFeeBasisPoints: 0, MaxFee: 0 },
        confirmOptions
      );

    const initAmount0 = new BN(10000000000);
    const initAmount1 = new BN(10000000000);
    // const { poolAddress, cpSwapPoolState, tx } = await initialize(
    //   program,
    //   owner,
    //   configAddress,
    //   token0,
    //   token0Program,
    //   token1,
    //   token1Program,
    //   confirmOptions,
    //   { initAmount0, initAmount1 }
    // );
    const [auth] = await getAuthAddress(cpSwapProgram)
    const [poolAddress] = await getPoolAddress(
      configAddress,
      token0,
      token1,
      cpSwapProgram
    )
    const [lpMintAddress] = await getPoolLpMintAddress(
      poolAddress,
      cpSwapProgram
    )
    const [vault0] = await getPoolVaultAddress(
      poolAddress,
      token0,
      cpSwapProgram
    )
    const [vault1] = await getPoolVaultAddress(
      poolAddress,
      token1,
      cpSwapProgram
    )
    const [creatorLpTokenAddress] = PublicKey.findProgramAddressSync(
      [
        owner.publicKey.toBuffer(),
        TOKEN_PROGRAM_ID.toBuffer(),
        lpMintAddress.toBuffer()
      ],
      ASSOCIATED_PROGRAM_ID
    )
    const [observationAddress] = await getOrcleAccountAddress(
      poolAddress,
      cpSwapProgram
    )
    const creatorToken0 = getAssociatedTokenAddressSync(
      token0,
      owner.publicKey,
      false,
      token0Program
    )
    const creatorToken1 = getAssociatedTokenAddressSync(
      token1,
      owner.publicKey,
      false,
      token1Program
    )
    const tx = await program.methods
      .raydiumInitialize(initAmount0, initAmount1, new BN(0))
      .accounts({
        cpSwapProgram: cpSwapProgram,
      creator: owner.publicKey,
      ammConfig: configAddress,
      authority: auth,
      poolState: poolAddress,
      token0Mint: token0,
      token1Mint: token1,
      lpMint: lpMintAddress,
      creatorToken0,
      creatorToken1,
      creatorLpToken: creatorLpTokenAddress,
      token0Vault: vault0,
      token1Vault: vault1,
      createPoolFee: createPoolFeeReceive,
      observationState: observationAddress,
      tokenProgram: TOKEN_PROGRAM_ID,
      token0Program: token0Program,
      token1Program: token1Program,
      associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
      })
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({units: 400000})
      ])
      .rpc(confirmOptions);

      const accountInfo = await program.provider.connection.getAccountInfo(
        poolAddress
      );
      const poolState =  CpmmPoolInfoLayout.decode(accountInfo.data);
      const cpSwapPoolState = {
        ammConfig: poolState.configId,
        token0Mint: poolState.mintA,
        token0Program: poolState.mintProgramA,
        token1Mint: poolState.mintB,
        token1Program: poolState.mintProgramB,
      };

    console.log("pool address: ", poolAddress.toString(), " tx:", tx);
  });
});
