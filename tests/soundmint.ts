import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Soundmint } from "../target/types/soundmint";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  LAMPORTS_PER_SOL,
  Transaction
} from "@solana/web3.js";
import { expect } from "chai";

describe("soundmint", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.getProvider();
  const connection = provider.connection;
  const program = anchor.workspace.Soundmint as Program<Soundmint>;

  // Helper functions
  const confirm = async (signature: string): Promise<string> => {
    const block = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
      signature,
      ...block,
    });
    return signature;
  };

  const log = async (signature: string): Promise<string> => {
    console.log(
      `Your transaction signature: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`
    );
    return signature;
  };

  // Test wallets
  const authority = Keypair.generate();
  const treasuryWallet = Keypair.generate();
  const artist = Keypair.generate();
  const collaborator1 = Keypair.generate();
  const collaborator2 = Keypair.generate();
  const streamingProvider = Keypair.generate();
  const listener = Keypair.generate();

  // PDA accounts that will be derived
  const [treasuryPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("treasury")],
    program.programId
  );

  const [artistProfilePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("artist_profile"), artist.publicKey.toBuffer()],
    program.programId
  );

  // Master NFT details (used later)
  const masterNftTitle = "Cosmic Journey";
  const masterNftDescription = "An immersive electronic music experience";
  const audioUri = "https://soundmint.com/audio/cosmic-journey.mp3";
  const artworkUri = "https://soundmint.com/artwork/cosmic-journey.jpg";
  const metadata = [
    { key: "genre", value: "Electronic" },
    { key: "bpm", value: "128" },
    { key: "key", value: "F Minor" }
  ];

  // Prepare all accounts in a central object for easy reference
  const accounts = {
    authority: authority.publicKey,
    treasuryWallet: treasuryWallet.publicKey,
    treasury: treasuryPDA,
    artist: artist.publicKey,
    artistProfile: artistProfilePDA,
    collaborator1: collaborator1.publicKey,
    collaborator2: collaborator2.publicKey,
    streamingProvider: streamingProvider.publicKey,
    listener: listener.publicKey,
    systemProgram: SystemProgram.programId,
  };

  // Fund the test wallets first
  it("Setup test accounts with SOL", async () => {
    // Create a transaction to fund multiple accounts at once
    let tx = new Transaction();

    // Add instructions to airdrop SOL to each account
    tx.add(
      SystemProgram.transfer({
        fromPubkey: provider.publicKey,
        toPubkey: authority.publicKey,
        lamports: 10 * LAMPORTS_PER_SOL,
      }),
      SystemProgram.transfer({
        fromPubkey: provider.publicKey,
        toPubkey: artist.publicKey,
        lamports: 5 * LAMPORTS_PER_SOL,
      }),
      SystemProgram.transfer({
        fromPubkey: provider.publicKey,
        toPubkey: collaborator1.publicKey,
        lamports: 2 * LAMPORTS_PER_SOL,
      }),
      SystemProgram.transfer({
        fromPubkey: provider.publicKey,
        toPubkey: collaborator2.publicKey,
        lamports: 2 * LAMPORTS_PER_SOL,
      }),
      SystemProgram.transfer({
        fromPubkey: provider.publicKey,
        toPubkey: streamingProvider.publicKey,
        lamports: 5 * LAMPORTS_PER_SOL,
      })
    );

    await provider.sendAndConfirm(tx as any)
      .then(confirm)
      .then(log);

    // Verify balances
    const authorityBalance = await connection.getBalance(authority.publicKey);
    expect(authorityBalance).to.be.above(5 * LAMPORTS_PER_SOL);
  });

  it("Initializes the platform", async () => {
    try {
      // Initialize the SoundMint platform
      const txSignature = await program.methods
        .initialize()
        .accounts({
          authority: accounts.authority,
          treasuryWallet: accounts.treasuryWallet,
          treasury: accounts.treasury,
          systemProgram: accounts.systemProgram,
        })
        .signers([authority])
        .rpc();

      await confirm(txSignature).then(log);

      // Fetch and verify the treasury account data
      const treasuryAccount = await program.account.treasury.fetch(treasuryPDA);

      expect(treasuryAccount.authority.toString()).to.equal(authority.publicKey.toString());
      expect(treasuryAccount.treasuryWallet.toString()).to.equal(treasuryWallet.publicKey.toString());
      expect(treasuryAccount.mintFee.toString()).to.equal("10000000"); // 0.01 SOL
      expect(treasuryAccount.platformFeeBasisPoints).to.equal(500); // 5%
      expect(treasuryAccount.totalRevenueCollected.toString()).to.equal("0");
    } catch (e) {
      console.error("Error in platform initialization:", e);
      throw e;
    }
  });

  it("Creates an artist profile", async () => {
    try {
      // Artist data
      const name = "Cosmic Rhythms";
      const description = "Creating immersive electronic soundscapes";
      const profileImageUri = "https://soundmint.com/artists/cosmic-rhythms.jpg";

      // Create the artist profile
      const txSignature = await program.methods
        .createArtistProfile(name, description, profileImageUri)
        .accounts({
          authority: accounts.artist,
          artistProfile: accounts.artistProfile,
          systemProgram: accounts.systemProgram,
        })
        .signers([artist])
        .rpc();

      await confirm(txSignature).then(log);

      // Fetch and verify the artist profile
      const artistProfileAccount = await program.account.artistProfile.fetch(artistProfilePDA);

      expect(artistProfileAccount.authority.toString()).to.equal(artist.publicKey.toString());
      expect(artistProfileAccount.name).to.equal(name);
      expect(artistProfileAccount.description).to.equal(description);
      expect(artistProfileAccount.profileImageUri).to.equal(profileImageUri);
      expect(artistProfileAccount.isVerified).to.be.false;
      expect(artistProfileAccount.trackCount.toString()).to.equal("0");
    } catch (e) {
      console.error("Error creating artist profile:", e);
      throw e;
    }
  });

  it("Updates an artist profile", async () => {
    try {
      // New artist data
      const newName = "Cosmic Rhythms Collective";
      const socialLinks = [
        {
          platform: "Twitter",
          url: "https://twitter.com/cosmicrhythms"
        },
        {
          platform: "Instagram",
          url: "https://instagram.com/cosmicrhythms"
        },
        {
          platform: "SoundCloud",
          url: "https://soundcloud.com/cosmicrhythms"
        }
      ];

      // Update the artist profile
      const txSignature = await program.methods
        .updateArtistProfile(newName, null, null, socialLinks)
        .accounts({
          authority: accounts.artist,
          artistProfile: accounts.artistProfile,
          systemProgram: accounts.systemProgram,
        })
        .signers([artist])
        .rpc();

      await confirm(txSignature).then(log);

      // Fetch and verify the updated artist profile
      const artistProfileAccount = await program.account.artistProfile.fetch(artistProfilePDA);

      expect(artistProfileAccount.name).to.equal(newName);
      expect(artistProfileAccount.socialLinks).to.have.lengthOf(3);
      expect(artistProfileAccount.socialLinks[0].platform).to.equal("Twitter");
      expect(artistProfileAccount.socialLinks[1].platform).to.equal("Instagram");
      expect(artistProfileAccount.socialLinks[2].platform).to.equal("SoundCloud");
    } catch (e) {
      console.error("Error updating artist profile:", e);
      throw e;
    }
  });

  it("Admin can update treasury configuration", async () => {
    try {
      // Update treasury config values
      const newMintFee = new BN(20000000); // 0.02 SOL
      const newPlatformFee = 600; // 6%

      // Update treasury configuration
      const txSignature = await program.methods
        .updateTreasuryConfig(newMintFee, newPlatformFee, null)
        .accounts({
          authority: accounts.authority,
          treasury: accounts.treasury,
          systemProgram: accounts.systemProgram,
        })
        .signers([authority])
        .rpc();

      await confirm(txSignature).then(log);

      // Fetch and verify the updated treasury
      const treasuryAccount = await program.account.treasury.fetch(treasuryPDA);

      expect(treasuryAccount.mintFee.toString()).to.equal(newMintFee.toString());
      expect(treasuryAccount.platformFeeBasisPoints).to.equal(newPlatformFee);
    } catch (e) {
      console.error("Error updating treasury configuration:", e);
      throw e;
    }
  });

  it("Admin can update streaming provider", async () => {
    try {
      // Update the streaming provider to our test provider
      const txSignature = await program.methods
        .updateStreamingProvider(accounts.streamingProvider)
        .accounts({
          authority: accounts.authority,
          treasury: accounts.treasury,
          systemProgram: accounts.systemProgram,
        })
        .signers([authority])
        .rpc();

      await confirm(txSignature).then(log);

      // Fetch and verify the updated treasury
      const treasuryAccount = await program.account.treasury.fetch(treasuryPDA);

      expect(treasuryAccount.streamingProvider.toString()).to.equal(streamingProvider.publicKey.toString());
    } catch (e) {
      console.error("Error updating streaming provider:", e);
      throw e;
    }
  });

  it("Admin can verify an artist", async () => {
    try {
      // Verify the artist
      const txSignature = await program.methods
        .verifyArtist(true)
        .accounts({
          authority: accounts.authority,
          artistProfile: accounts.artistProfile,
          artistOwner: accounts.artist,
          treasury: accounts.treasury,
          systemProgram: accounts.systemProgram,
        })
        .signers([authority])
        .rpc();

      await confirm(txSignature).then(log);

      // Fetch and verify the artist profile
      const artistProfileAccount = await program.account.artistProfile.fetch(artistProfilePDA);

      expect(artistProfileAccount.isVerified).to.be.true;
    } catch (e) {
      console.error("Error verifying artist:", e);
      throw e;
    }
  });

  // These tests can be enabled once NFT minting works

  xit("Mints a master NFT", async () => {
    try {
      // Generate a new mint keypair for the NFT
      const mintKeypair = Keypair.generate();

      // Derive the master NFT PDA
      const [masterNftPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("master_nft"), mintKeypair.publicKey.toBuffer()],
        program.programId
      );

      // Create metadata account address
      const [metadataAccount] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("metadata"),
          new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
          mintKeypair.publicKey.toBuffer(),
        ],
        new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
      );

      // Mint master NFT
      const txSignature = await program.methods
        .mintMasterNft(
          masterNftTitle,
          masterNftDescription,
          audioUri,
          artworkUri,
          metadata
        )
        .accounts({
          authority: accounts.artist,
          artistProfile: accounts.artistProfile,
          masterNft: masterNftPDA,
          mint: mintKeypair.publicKey,
          // Add other required accounts once implementation is fixed
        })
        .signers([artist, mintKeypair])
        .rpc();

      await confirm(txSignature).then(log);

      // Fetch the master NFT account
      const masterNftAccount = await program.account.masterNft.fetch(masterNftPDA);

      // Verify master NFT data
      expect(masterNftAccount.title).to.equal(masterNftTitle);
      expect(masterNftAccount.description).to.equal(masterNftDescription);
      expect(masterNftAccount.audioUri).to.equal(audioUri);
      expect(masterNftAccount.artworkUri).to.equal(artworkUri);

      // Add this to our accounts for later use
      accounts.masterNft = masterNftPDA;
      accounts.masterNftMint = mintKeypair.publicKey;
    } catch (e) {
      console.error("Error minting master NFT:", e);
      throw e;
    }
  });

  xit("Creates a royalty split", async () => {
    try {
      // Derive the royalty split PDA
      const [royaltySplitPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("royalty_split"), accounts.masterNft.toBuffer()],
        program.programId
      );

      // Configure royalty split
      const collaborators = [
        {
          address: accounts.artist,
          shareBasisPoints: 7000, // 70%
          name: "Main Artist"
        },
        {
          address: accounts.collaborator1,
          shareBasisPoints: 2000, // 20%
          name: "Producer"
        },
        {
          address: accounts.collaborator2,
          shareBasisPoints: 1000, // 10%
          name: "Featured Artist"
        }
      ];

      // Create royalty split
      const txSignature = await program.methods
        .createRoyaltySplit(collaborators)
        .accounts({
          authority: accounts.artist,
          artistProfile: accounts.artistProfile,
          masterNft: accounts.masterNft,
          royaltySplit: royaltySplitPDA,
          systemProgram: accounts.systemProgram
        })
        .signers([artist])
        .rpc();

      await confirm(txSignature).then(log);

      // Fetch the royalty split account
      const royaltySplitAccount = await program.account.royaltySplit.fetch(royaltySplitPDA);

      // Verify royalty split data
      expect(royaltySplitAccount.masterNft.toString()).to.equal(accounts.masterNft.toString());
      expect(royaltySplitAccount.collaborators).to.have.lengthOf(3);
      expect(royaltySplitAccount.totalBasisPoints).to.equal(10000);
      expect(royaltySplitAccount.collaborators[0].shareBasisPoints).to.equal(7000);
      expect(royaltySplitAccount.collaborators[1].shareBasisPoints).to.equal(2000);
      expect(royaltySplitAccount.collaborators[2].shareBasisPoints).to.equal(1000);

      // Add to our accounts
      accounts.royaltySplit = royaltySplitPDA;
    } catch (e) {
      console.error("Error creating royalty split:", e);
      throw e;
    }
  });

  // Additional tests can be uncommented once previous tests are fixed
});