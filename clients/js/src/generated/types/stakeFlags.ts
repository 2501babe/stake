/**
 * This code was AUTOGENERATED using the codama library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun codama to update it.
 *
 * @see https://github.com/codama-idl/codama
 */

import {
  combineCodec,
  getStructDecoder,
  getStructEncoder,
  getU8Decoder,
  getU8Encoder,
  type Codec,
  type Decoder,
  type Encoder,
} from '@solana/web3.js';

export type StakeFlags = { bits: number };

export type StakeFlagsArgs = StakeFlags;

export function getStakeFlagsEncoder(): Encoder<StakeFlagsArgs> {
  return getStructEncoder([['bits', getU8Encoder()]]);
}

export function getStakeFlagsDecoder(): Decoder<StakeFlags> {
  return getStructDecoder([['bits', getU8Decoder()]]);
}

export function getStakeFlagsCodec(): Codec<StakeFlagsArgs, StakeFlags> {
  return combineCodec(getStakeFlagsEncoder(), getStakeFlagsDecoder());
}