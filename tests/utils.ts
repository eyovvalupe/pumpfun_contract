import {
  Connection,
  ConfirmOptions,
  Keypair,
  Signer,
} from "@solana/web3.js";
import {
  createTokenMintAndAssociatedTokenAccount,
} from "./util";

import { configAddress } from "./config";
export async function setupInitializeTest(
    connection: Connection,
    owner: Signer,
    transferFeeConfig: { transferFeeBasisPoints: number; MaxFee: number } = {
      transferFeeBasisPoints: 0,
      MaxFee: 0,
    },
    confirmOptions?: ConfirmOptions
  ) {
    const [{ token0, token0Program }, { token1, token1Program }] =
      await createTokenMintAndAssociatedTokenAccount(
        connection,
        owner,
        new Keypair(),
        transferFeeConfig
      );
    return {
      configAddress,
      token0,
      token0Program,
      token1,
      token1Program,
    };
  }