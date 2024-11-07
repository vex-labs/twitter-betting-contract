const { base_encode, base_decode } = require('near-api-js/lib/utils/serialize');
const EC = require('elliptic').ec;
const { sha3_256 } = require('js-sha3');

// Public key of the MPC contract
const rootPublicKey = 'secp256k1:4NfTiv3UsGahebgTaHyD9vF8KYKMBnfd6kh94mK6xv8fGBiJB8TBtFMP5WWXz6B89Ac1fbpzPwAvoyQebemHFwx3';

async function deriveKey(predecessorId, derivationPath) {
    const childPubKey = await deriveChildPublicKey(najPublicKeyStrToUncompressedHexPoint(), predecessorId, derivationPath);
    return 'secp256k1:' + base_encode(Buffer.from(childPubKey.substring(2), 'hex'));
}

function najPublicKeyStrToUncompressedHexPoint() {
  const res = '04' + Buffer.from(base_decode(rootPublicKey.split(':')[1])).toString('hex');
  return res;
}

async function deriveChildPublicKey(
  parentUncompressedPublicKeyHex,
  predecessorId,
  path = ''
) {
  const ec = new EC("secp256k1");
  const scalarHex = sha3_256(
    `near-mpc-recovery v0.1.0 epsilon derivation:${predecessorId},${path}`
  );

  const x = parentUncompressedPublicKeyHex.substring(2, 66);
  const y = parentUncompressedPublicKeyHex.substring(66);

  // Create a point object from X and Y coordinates
  const oldPublicKeyPoint = ec.curve.point(x, y);

  // Multiply the scalar by the generator point G
  const scalarTimesG = ec.g.mul(scalarHex);

  // Add the result to the old public key point
  const newPublicKeyPoint = oldPublicKeyPoint.add(scalarTimesG);
  const newX = newPublicKeyPoint.getX().toString("hex").padStart(64, "0");
  const newY = newPublicKeyPoint.getY().toString("hex").padStart(64, "0");
  return "04" + newX + newY;
}

module.exports = { deriveKey };