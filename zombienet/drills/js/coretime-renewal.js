// 15 §4.7; 09 §4/§7.1 — permissionless renewal under every freeze.
async function run(nodeName, networkInfo, args) {
  const { wsUri, userDefinedTypes } = networkInfo.nodesByName[nodeName];
  const api = await zombie.connect(wsUri, userDefinedTypes);
  await zombie.util.cryptoWaitReady();
  const keyring = new zombie.Keyring({ type: "sr25519" });
  const alice = keyring.addFromUri("//Alice");
  const periodIndex = Number(args[0]);

  if (!api.tx.futarchyTreasury?.executeCoretimeRenewal) {
    throw new Error("execute_coretime_renewal is absent from runtime metadata");
  }
  return new Promise((resolve, reject) => {
    let unsubscribe;
    let settled = false;
    const finish = (callback) => {
      settled = true;
      if (unsubscribe) unsubscribe();
      callback();
    };
    api.tx.futarchyTreasury
      .executeCoretimeRenewal(periodIndex)
      .signAndSend(alice, ({ dispatchError, events, status }) => {
        if (dispatchError) {
          finish(() => reject(new Error(`renewal dispatch failed: ${dispatchError.toString()}`)));
        } else if (status.isInBlock) {
          const success = events.some(({ event }) =>
            event.section === "system" && event.method === "ExtrinsicSuccess");
          if (success) {
            finish(() => resolve(periodIndex));
          }
        }
      })
      .then((unsub) => {
        unsubscribe = unsub;
        if (settled) unsubscribe();
      })
      .catch((error) => finish(() => reject(error)));
  });
}

module.exports = { run };
