import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CreatorFund } from "../target/types/creator_fund";
import crypto from "crypto";
import * as assert from "assert";
import { PublicKey } from "@solana/web3.js";

describe("creator_fund", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const COMMUNITY_SEED = "community";
  const FOLLOW_COMMUNITY_SEED = "follow_community";
  const POST_SEED = "post";
  const POST_VOTE_SEED = "post_vote";
  const COMMENT_SEED = "comment";
  const COMMENT_VOTE_SEED = "comment_vote";

  // authors/actors
  const bob = anchor.web3.Keypair.generate();

  const program = anchor.workspace.creatorFund as Program<CreatorFund>;

  let postTitle1 = "Hello World";
  const postContent1 = "This is my first post on Solana";

  // PDAs
  let postPDA1: anchor.web3.PublicKey;

  before(async () => {
    await airdrop(bob.publicKey);
    // create post
    postTitle1 = postTitle1 + Date.now().toString();
    [postPDA1] = getPostAddress(
      bob.publicKey,
      postTitle1,
      program.programId,
    );
    await program.methods
      .createPost(postTitle1, postContent1)
      .accounts({
        author: bob.publicKey,
        post: postPDA1,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([bob])
      .rpc();
  });

  describe("create post", () => {
    it("can create a post", async () => {
      let title = "Getting Started with Solana";
        let content = "This is a post about getting started with Solana.";
        // get post PDA
        [postPDA1] = getPostAddress(
          bob.publicKey,
          title,
          program.programId,
        );
        // create post
        await program.methods
          .createPost(title, content)
          .accounts({
            author: bob.publicKey,
            post: postPDA1,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([bob])
          .rpc();
  
        // fetch created post
        const newPost = await program.account.post.fetch(postPDA1);
  
        assert.equal(newPost.author.toBase58(), bob.publicKey.toBase58());
        assert.equal(newPost.title, title);
        assert.equal(newPost.content, content);
    });
  
    it("cannot create post with title longer than 100 characters", async () => {
      const longTitle = "a".repeat(101);
      const content = "This is a post content";
      try {
        let [postPDA] = getPostAddress(
          bob.publicKey,
          longTitle,
          program.programId,
        );
        await program.methods
          .createPost(longTitle, content)
          .accounts({
            author: bob.publicKey,
            post: postPDA,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([bob])
          .rpc();
      } catch (error) {
        const err = anchor.AnchorError.parse(error.logs);
        console.log("Create post error: ", err);
        assert.strictEqual(
          err.error.errorCode.code,
          "PostTitleTooLong",
          "Expected 'PostTitleTooLong' error for long post title",
        );
      }
    });
  
    it("cannot create a post with empty title", async () => {
      const content = "This is a post content";
      try {
        let [postPDA] = getPostAddress(
          bob.publicKey,
          "",
          program.programId,
        );
        await program.methods
          .createPost("", content)
          .accounts({
            author: bob.publicKey,
            post: postPDA,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([bob])
          .rpc();
      } catch (error) {
        const err = anchor.AnchorError.parse(error.logs);
        assert.strictEqual(
          err.error.errorCode.code,
          "PostTitleRequired",
          "Expected 'PostTitleRequired' error for empty post title",
        );
      }
    });
  
    it("cannot create a post with empty content", async () => {
      const title = "This is a post title";
      try {
        let [postPDA] = getPostAddress(
          bob.publicKey,
          title,
          program.programId,
        );
        await program.methods
          .createPost(title, "")
          .accounts({
            author: bob.publicKey,
            post: postPDA,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([bob])
          .rpc();
      } catch (error) {
        const err = anchor.AnchorError.parse(error.logs);
        assert.strictEqual(
          err.error.errorCode.code,
          "PostContentRequired",
          "Expected 'PostContentRequired' error for empty post content",
        );
      }
    });
  
    it("cannot create post with content longer than 280 characters", async () => {
      const title = "This is a post title";
      const longContent = "a".repeat(281);
      try {
        let [postPDA] = getPostAddress(
          bob.publicKey,
          title,
          program.programId,
        );
        await program.methods
          .createPost(title, longContent)
          .accounts({
            author: bob.publicKey,
            post: postPDA,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([bob])
          .rpc();
      } catch (error) {
        const err = anchor.AnchorError.parse(error.logs);
        assert.strictEqual(
          err.error.errorCode.code,
          "PostContentTooLong",
          "Expected 'PostContentTooLong' error for long post content",
        );
      }
    });
  });


  // Helper functions
  const airdrop = async (publicKey: anchor.web3.PublicKey) => {
    const sig = await program.provider.connection.requestAirdrop(
      publicKey,
      1_000_000_000, // 1 SOL
    );
    await program.provider.connection.confirmTransaction(sig, "confirmed");
  };

  // get the PDA for a post
  const getPostAddress = (
    author: PublicKey,
    title: string,
    programID: PublicKey,
  ) => {
    let hexString = crypto
      .createHash("sha256")
      .update(title, "utf-8")
      .digest("hex");
    let titleSeed = Uint8Array.from(Buffer.from(hexString, "hex"));

    return PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode(POST_SEED),
        titleSeed,
        author.toBuffer(),
      ],
      programID,
    );
  };
});
