import { expect, assert } from "chai";

import * as ergo from "..";
let ergo_wasm;
beforeEach(async () => {
  ergo_wasm = await ergo;
});

it("unsignedbigint tests", async () => {
  const bigint = new ergo_wasm.UnsignedBigInt(5)
  const bigint2 = new ergo_wasm.UnsignedBigInt(BigInt(5))
  assert(bigint.eq(bigint2), "should be equal")
  assert(bigint.add(new ergo_wasm.UnsignedBigInt(2)).eq(new ergo_wasm.UnsignedBigInt(7)))
  const modulus = new ergo_wasm.UnsignedBigInt(7)
  assert(bigint.mod_mul(bigint.mod_inv(modulus), modulus).eq(new ergo_wasm.UnsignedBigInt(1)))
});
