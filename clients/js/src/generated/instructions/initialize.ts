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
  getU32Decoder,
  getU32Encoder,
  transformEncoder,
  type Address,
  type Codec,
  type Decoder,
  type Encoder,
  type IAccountMeta,
  type IInstruction,
  type IInstructionWithAccounts,
  type IInstructionWithData,
  type ReadonlyAccount,
  type WritableAccount,
} from '@solana/kit';
import { STAKE_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';
import {
  getAuthorizedDecoder,
  getAuthorizedEncoder,
  getLockupDecoder,
  getLockupEncoder,
  type Authorized,
  type AuthorizedArgs,
  type Lockup,
  type LockupArgs,
} from '../types';

export const INITIALIZE_DISCRIMINATOR = 0;

export function getInitializeDiscriminatorBytes() {
  return getU32Encoder().encode(INITIALIZE_DISCRIMINATOR);
}

export type InitializeInstruction<
  TProgram extends string = typeof STAKE_PROGRAM_ADDRESS,
  TAccountStake extends string | IAccountMeta<string> = string,
  TAccountRentSysvar extends
    | string
    | IAccountMeta<string> = 'SysvarRent111111111111111111111111111111111',
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountStake extends string
        ? WritableAccount<TAccountStake>
        : TAccountStake,
      TAccountRentSysvar extends string
        ? ReadonlyAccount<TAccountRentSysvar>
        : TAccountRentSysvar,
      ...TRemainingAccounts,
    ]
  >;

export type InitializeInstructionData = {
  discriminator: number;
  arg0: Authorized;
  arg1: Lockup;
};

export type InitializeInstructionDataArgs = {
  arg0: AuthorizedArgs;
  arg1: LockupArgs;
};

export function getInitializeInstructionDataEncoder(): Encoder<InitializeInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU32Encoder()],
      ['arg0', getAuthorizedEncoder()],
      ['arg1', getLockupEncoder()],
    ]),
    (value) => ({ ...value, discriminator: INITIALIZE_DISCRIMINATOR })
  );
}

export function getInitializeInstructionDataDecoder(): Decoder<InitializeInstructionData> {
  return getStructDecoder([
    ['discriminator', getU32Decoder()],
    ['arg0', getAuthorizedDecoder()],
    ['arg1', getLockupDecoder()],
  ]);
}

export function getInitializeInstructionDataCodec(): Codec<
  InitializeInstructionDataArgs,
  InitializeInstructionData
> {
  return combineCodec(
    getInitializeInstructionDataEncoder(),
    getInitializeInstructionDataDecoder()
  );
}

export type InitializeInput<
  TAccountStake extends string = string,
  TAccountRentSysvar extends string = string,
> = {
  /** Uninitialized stake account */
  stake: Address<TAccountStake>;
  /** Rent sysvar */
  rentSysvar?: Address<TAccountRentSysvar>;
  arg0: InitializeInstructionDataArgs['arg0'];
  arg1: InitializeInstructionDataArgs['arg1'];
};

export function getInitializeInstruction<
  TAccountStake extends string,
  TAccountRentSysvar extends string,
  TProgramAddress extends Address = typeof STAKE_PROGRAM_ADDRESS,
>(
  input: InitializeInput<TAccountStake, TAccountRentSysvar>,
  config?: { programAddress?: TProgramAddress }
): InitializeInstruction<TProgramAddress, TAccountStake, TAccountRentSysvar> {
  // Program address.
  const programAddress = config?.programAddress ?? STAKE_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    stake: { value: input.stake ?? null, isWritable: true },
    rentSysvar: { value: input.rentSysvar ?? null, isWritable: false },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  // Original args.
  const args = { ...input };

  // Resolve default values.
  if (!accounts.rentSysvar.value) {
    accounts.rentSysvar.value =
      'SysvarRent111111111111111111111111111111111' as Address<'SysvarRent111111111111111111111111111111111'>;
  }

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.stake),
      getAccountMeta(accounts.rentSysvar),
    ],
    programAddress,
    data: getInitializeInstructionDataEncoder().encode(
      args as InitializeInstructionDataArgs
    ),
  } as InitializeInstruction<
    TProgramAddress,
    TAccountStake,
    TAccountRentSysvar
  >;

  return instruction;
}

export type ParsedInitializeInstruction<
  TProgram extends string = typeof STAKE_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** Uninitialized stake account */
    stake: TAccountMetas[0];
    /** Rent sysvar */
    rentSysvar: TAccountMetas[1];
  };
  data: InitializeInstructionData;
};

export function parseInitializeInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedInitializeInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 2) {
    // TODO: Coded error.
    throw new Error('Not enough accounts');
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts![accountIndex]!;
    accountIndex += 1;
    return accountMeta;
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      stake: getNextAccount(),
      rentSysvar: getNextAccount(),
    },
    data: getInitializeInstructionDataDecoder().decode(instruction.data),
  };
}
