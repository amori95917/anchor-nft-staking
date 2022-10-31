import * as anchor from "@project-serum/anchor";
import { Mint } from "./mint";
import { NftStaking } from "../../target/types/nft_staking";

export class TokenAccount<
  T extends anchor.web3.PublicKey | anchor.web3.Keypair
> {
  constructor(
    public program: anchor.Program<NftStaking>,
    public key: anchor.web3.PublicKey,
    public mint: Mint,
    public owner: T
  ) {}
}
