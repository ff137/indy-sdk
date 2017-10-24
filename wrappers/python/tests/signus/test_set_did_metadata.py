from indy import IndyError
from indy import signus
from indy.error import ErrorCode

import pytest


@pytest.mark.asyncio
async def test_set_did_metadata_works(wallet_handle, did, metadata):
    await signus.set_did_metadata(wallet_handle, did, metadata)


@pytest.mark.asyncio
async def test_set_did_metadata_works_for_replace(wallet_handle, did, metadata):
    await signus.set_did_metadata(wallet_handle, did, metadata)
    received_metadata = await signus.get_did_metadata(wallet_handle, did)
    assert metadata == received_metadata

    new_metadata = 'new metadata'
    await signus.set_did_metadata(wallet_handle, did, new_metadata)
    updated_metadata = await signus.get_did_metadata(wallet_handle, did)
    assert new_metadata == updated_metadata


@pytest.mark.asyncio
async def test_set_did_metadata_works_for_empty_string(wallet_handle, did):
    await signus.set_did_metadata(wallet_handle, did, '')


@pytest.mark.asyncio
async def test_set_did_metadata_works_for_invalid_did(wallet_handle, metadata):
    with pytest.raises(IndyError) as e:
        await signus.set_did_metadata(wallet_handle, 'invalid_base58string', metadata)
    assert ErrorCode.WalletInvalidHandle == e.value.error_code

    with pytest.raises(IndyError) as e:
        await signus.set_did_metadata(wallet_handle, 'invalidDidLength', metadata)
    assert ErrorCode.WalletInvalidHandle == e.value.error_code


@pytest.mark.asyncio
async def test_set_did_metadata_works_for_invalid_handle(wallet_handle, did, metadata):
    with pytest.raises(IndyError) as e:
        invalid_wallet_handle = wallet_handle + 1
        await signus.set_did_metadata(invalid_wallet_handle, did, metadata)
    assert ErrorCode.WalletInvalidHandle == e.value.error_code
