/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */

import {
  Keypair,
  Connection,
  PublicKey,
  LAMPORTS_PER_SOL,
  SystemProgram,
  TransactionInstruction,
  Transaction,
  sendAndConfirmTransaction,
} from '@solana/web3.js';
import fs from 'mz/fs';
import path from 'path';
import * as borsh from 'borsh';
import {getPayer, getRpcUrl, createKeypairFromFile} from './utils';
import { serialize, deserialize, deserializeUnchecked } from "borsh";
import { Buffer } from "buffer";
import { Ok, Err, Result } from 'ts-results';

import bs58 from 'bs58';

/**
 * Connection to the network
 */
let connection: Connection;

/**
 * Keypair associated to the fees' payer
 */
let payer: Keypair;

/**
 * Hello world's program id
 */
let programId: PublicKey;

/**
 * The public key of the account we are saying hello to
 */
let greetedPubkey: PublicKey;


let contractPubkey: PublicKey;

/**
 * Path to program files
 */
const PROGRAM_PATH = path.resolve(__dirname, '../../dist/program');

/**
 * Path to program shared object file which should be deployed on chain.
 * This file is created when running either:
 *   - `npm run build:program-c`
 *   - `npm run build:program-rust`
 */
const PROGRAM_SO_PATH = path.join(PROGRAM_PATH, 'helloworld.so');

/**
 * Path to the keypair of the deployed program.
 * This file is created when running `solana program deploy dist/program/helloworld.so`
 */
const PROGRAM_KEYPAIR_PATH = path.join(PROGRAM_PATH, 'helloworld-keypair.json');

/**
 * The state of a greeting account managed by the hello world program
 */
class GreetingAccount {
  counter = 0;
  constructor(fields: {counter: number} | undefined = undefined) {
    if (fields) {
      this.counter = fields.counter;
    }
  }
}

/**
 * Borsh schema definition for greeting accounts
 */
const GreetingSchema = new Map([
  [GreetingAccount, {kind: 'struct', fields: [['counter', 'u32']]}],
]);

/**
 * The expected size of each greeting account.
 */
const GREETING_SIZE = borsh.serialize(
  GreetingSchema,
  new GreetingAccount(),
).length;

/**
 * Establish a connection to the cluster
 */
export async function establishConnection(): Promise<void> {
  const rpcUrl = await getRpcUrl();
  connection = new Connection(rpcUrl, 'confirmed');
  const version = await connection.getVersion();
  console.log('Connection to cluster established:', rpcUrl, version);
}

/**
 * Establish an account to pay for everything
 */
export async function establishPayer(): Promise<void> {
  let fees = 0;
  if (!payer) {
    const {feeCalculator} = await connection.getRecentBlockhash();

    // Calculate the cost to fund the greeter account
    fees += await connection.getMinimumBalanceForRentExemption(GREETING_SIZE);

    // Calculate the cost of sending transactions
    fees += feeCalculator.lamportsPerSignature * 100; // wag

    payer = await getPayer();
  }

  let lamports = await connection.getBalance(payer.publicKey);
  if (lamports < fees) {
    // If current balance is not enough to pay for fees, request an airdrop
    const sig = await connection.requestAirdrop(
      payer.publicKey,
      fees - lamports,
    );
    await connection.confirmTransaction(sig);
    lamports = await connection.getBalance(payer.publicKey);
  }

  console.log(
    'Using account',
    payer.publicKey.toBase58(),
    'containing',
    lamports / LAMPORTS_PER_SOL,
    'SOL to pay for fees',
  );
}

/**
 * Check if the hello world BPF program has been deployed
 */
export async function checkProgram(): Promise<void> {
  // Read program id from keypair file
  try {
    const programKeypair = await createKeypairFromFile(PROGRAM_KEYPAIR_PATH);
    programId = programKeypair.publicKey;
  } catch (err) {
    const errMsg = (err as Error).message;
    throw new Error(
      `Failed to read program keypair at '${PROGRAM_KEYPAIR_PATH}' due to error: ${errMsg}. Program may need to be deployed with \`solana program deploy dist/program/helloworld.so\``,
    );
  }

  // Check if the program has been deployed
  const programInfo = await connection.getAccountInfo(programId);
  if (programInfo === null) {
    if (fs.existsSync(PROGRAM_SO_PATH)) {
      throw new Error(
        'Program needs to be deployed with `solana program deploy dist/program/helloworld.so`',
      );
    } else {
      throw new Error('Program needs to be built and deployed');
    }
  } else if (!programInfo.executable) {
    throw new Error(`Program is not executable`);
  }
  console.log(`Using program ${programId.toBase58()}`);

  // Derive the address (public key) of a greeting account from the program so that it's easy to find later.
  const GREETING_SEED = 'hello';
  greetedPubkey = await PublicKey.createWithSeed(
    payer.publicKey,
    GREETING_SEED,
    programId,
  );

  console.log("greetedPubkey:", greetedPubkey.toBase58())

  // Check if the greeting account has already been created
  const greetedAccount = await connection.getAccountInfo(greetedPubkey);
  if (greetedAccount === null) {
    console.log(
      'Creating account',
      greetedPubkey.toBase58(),
      'to say hello to',
    );
    const lamports = await connection.getMinimumBalanceForRentExemption(
      GREETING_SIZE,
    );

    const transaction = new Transaction().add(
      SystemProgram.createAccountWithSeed({
        fromPubkey: payer.publicKey,
        basePubkey: payer.publicKey,
        seed: GREETING_SEED,
        newAccountPubkey: greetedPubkey,
        lamports,
        space: GREETING_SIZE,
        programId,
      }),
    );
    await sendAndConfirmTransaction(connection, transaction, [payer]);
  }
}

export async function sayHello(): Promise<void> {
  console.log('Saying hello to', greetedPubkey.toBase58());
  const receiver1 = new PublicKey('NbJvJCS7y7LnKtiJPTzfbRiuQnkxNoWpcJae99FegZW');
  const receiver2 = new PublicKey('NbJvJCS7y7LnKtiJPTzfbRiuQnkxNoWpcJae99FegZW');
  const buffer = Buffer.alloc(8); // 8 bytes for a 64-bit number
  const value = BigInt(300000000);
  buffer.writeBigUInt64LE(value,0);
  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: receiver1, isSigner: false, isWritable: true },
      { pubkey: receiver2, isSigner: false, isWritable : true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: programId,
    data: buffer, // All instructions are hellos
  });
  await sendAndConfirmTransaction(
      connection,
      new Transaction().add(instruction),
      [payer],
  );
}

export async function sendTransaction(): Promise<void> {
  console.log("sendTransaction")
  const addr = new PublicKey("FCSKqqjPcNRQE4QRVLrJASmhg95QMPKmD1BpcaWJvg1G");

  await sendAndConfirmTransaction(
      connection,
      new Transaction().add(SystemProgram.transfer({
        fromPubkey: payer.publicKey,
        toPubkey: addr,
        lamports: 10 * LAMPORTS_PER_SOL,
      })),
      [payer],
  );
}
/**
 * Say hello
 */
export async function deposit(pubKey: string): Promise<void> {
  let publicKey = contractPubkey
  if (pubKey) {
    publicKey = new PublicKey(pubKey)
  }

  const feePayer = Keypair.fromSecretKey(
      bs58.decode("2UyuFwGhV9Ts7YX5gxb6N1fppFEpGNY5gwf9szYspJLuu9VbCLhHfTo7wDdY1pFWAoUzHkyURzrE7KE5NDeQkT2S")
  );
  const lamports = await connection.getBalance(feePayer.publicKey);
  console.log("deposit: ", feePayer.publicKey.toBase58(), lamports / 10 ** 9)

  // const receiver1 = new PublicKey('B5nYWgm4SUDbLe8SvDVKEcv7VNyXVcX1z4ELUdLeNuTv');
  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: feePayer.publicKey, isSigner: true, isWritable: true },
      { pubkey: publicKey, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: programId,
    data: Buffer.from([0]), // All instructions are hellos
  });
  await sendAndConfirmTransaction(
      connection,
      new Transaction().add(instruction),
      [feePayer],
  );
}


export async function createAccount(): Promise<void> {
  const space = 205;

  // Seed the created account with lamports for rent exemption
  const rentExemptionAmount =
      await connection.getMinimumBalanceForRentExemption(space);

  const newAccountPubkey = Keypair.generate();
  const createAccountParams = {
    fromPubkey: payer.publicKey,
    newAccountPubkey: newAccountPubkey.publicKey,
    lamports: rentExemptionAmount,
    space,
    programId: programId,
  };

  const createAccountTransaction = new Transaction().add(
      SystemProgram.createAccount(createAccountParams)
  );

  contractPubkey = newAccountPubkey.publicKey
  console.log("contractPubkey: ", contractPubkey.toBase58())

  await sendAndConfirmTransaction(connection, createAccountTransaction, [
    payer,
    newAccountPubkey,
  ]);
}

export async function withdraw(): Promise<void> {

  const feePayer = Keypair.fromSecretKey(
      bs58.decode("2UyuFwGhV9Ts7YX5gxb6N1fppFEpGNY5gwf9szYspJLuu9VbCLhHfTo7wDdY1pFWAoUzHkyURzrE7KE5NDeQkT2S")
  );
  const lamports = await connection.getBalance(feePayer.publicKey);
  console.log("withdraw: ", feePayer.publicKey.toBase58(), lamports / 10 ** 9)

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: greetedPubkey, isSigner: false, isWritable: true },
      { pubkey: feePayer.publicKey, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: programId,
    data: Buffer.from([1]), // All instructions are hellos
  });

  await sendAndConfirmTransaction(
      connection,
      new Transaction().add(instruction),
      [feePayer],
  );
}

/**
 * Report the number of times the greeted account has been said hello to
 */
// Flexible class that takes properties and imbues them
// to the object instance
class Assignable {
  constructor(properties) {
    Object.keys(properties).map((key) => {
      return (this[key] = properties[key]);
    });
  }
}

export class AccoundData extends Assignable {}

const dataSchema = new Map([
  [
    AccoundData,
    {
      kind: "struct",
      fields: [
        ["initialized", "u8"],
        ["tree_length", "u32"],
        ["map", { kind: "map", key: "string", value: "u64" }],
      ],
    },
  ],
]);

export async function getAccountData(pubKey: string
): Promise<Result<AccoundData, Error>> {
  let publicKey = contractPubkey
  if (pubKey) {
    publicKey = new PublicKey(pubKey)
  }
  let nameAccount = await connection.getAccountInfo(
      publicKey,
      'processed'
  );
  return Ok(deserializeUnchecked(dataSchema, AccoundData, nameAccount.data))
}
export async function reportGreetings(): Promise<void> {
  const accountInfo = await connection.getAccountInfo(greetedPubkey);
  if (accountInfo === null) {
    throw 'Error: cannot find the greeted account';
  }
  const greeting = borsh.deserialize(
    GreetingSchema,
    GreetingAccount,
    accountInfo.data,
  );
  console.log(
    greetedPubkey.toBase58(),
    'has been greeted',
    greeting.counter,
    'time(s)',
  );
}
