from indy_sdk import IndyError
from indy_sdk import wallet
from indy_sdk.error import ErrorCode

import pytest


@pytest.mark.asyncio
async def test_create_wallet_works(cleanup_storage):
    await wallet.create_wallet('pool1', 'wallet1', 'default', None, None)


@pytest.mark.asyncio
async def test_create_wallet_works_for_empty_type(cleanup_storage):
    await wallet.create_wallet('pool1', 'wallet1', None, None, None)


@pytest.mark.asyncio
async def test_create_wallet_works_for_config_json(cleanup_storage):
    await wallet.create_wallet('pool1', 'wallet3', 'default', '{"freshness_time":1000}', None)


@pytest.mark.asyncio
async def test_create_wallet_works_for_unknown_type(cleanup_storage):
    with pytest.raises(IndyError) as e:
        await wallet.create_wallet('pool1', 'wallet3', 'unknown_type', None, None)
    assert ErrorCode.WalletUnknownTypeError == e.value.error_code


@pytest.mark.asyncio
async def test_create_wallet_works_for_empty_name(cleanup_storage):
    with pytest.raises(IndyError) as e:
        await wallet.create_wallet('pool1', '', 'default', None, None)
    assert ErrorCode.CommonInvalidParam3 == e.value.error_code


@pytest.mark.asyncio
async def test_create_wallet_works_for_duplicate_name(cleanup_storage):
    with pytest.raises(IndyError) as e:
        await wallet.create_wallet('pool1', 'wallet4', 'default', None, None)
        await wallet.create_wallet('pool1', 'wallet4', 'default', None, None)
    assert ErrorCode.WalletAlreadyExistsError == e.value.error_code
