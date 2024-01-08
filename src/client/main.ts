/**
 * Hello world
 */

import {
  establishConnection,
  establishPayer,
  checkProgram,
  deposit,
  withdraw,
  reportGreetings,
  sendTransaction,
  createAccount,
  getAccountData,
} from './hello_world';

async function main() {
  console.log("Let's say hello to a Solana account...");

  // Establish connection to the cluster
  await establishConnection();

  // Determine who pays for the fees
  await establishPayer();

  // Check if the program has been deployed
  await checkProgram();

  // Say hello to an account
  await sendTransaction();
  //

  //
  let pubKey = ""
  if(!pubKey) {
    await createAccount()
  }
  await deposit(pubKey);
  // //
  // await withdraw()
  //

  const deser_result = await getAccountData(pubKey)
  console.log(deser_result)
  const value = deser_result.val
  const data = value["map"].get("FCSKqqjPcNRQE4QRVLrJASmhg95QMPKmD1BpcaWJvg1G")
  console.log(data.toString())


  // Find out how many times that account has been greeted
  // await reportGreetings();

  console.log('Success');
}

main().then(
  () => process.exit(),
  err => {
    console.error(err);
    process.exit(-1);
  },
);
