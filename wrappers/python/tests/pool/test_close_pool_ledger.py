from tests.utils import pool
from indy_sdk.pool import close_pool_ledger, open_pool_ledger
from indy_sdk.error import ErrorCode, IndyError

import pytest


@pytest.mark.asyncio
async def test_close_pool_ledger_works(cleanup_storage):
    handle = await pool.create_and_open_pool_ledger("pool_1")
    await close_pool_ledger(handle)


@pytest.mark.asyncio
async def test_close_pool_ledger_works_for_twice(cleanup_storage):
    handle = await pool.create_and_open_pool_ledger("pool_1")
    await close_pool_ledger(handle)

    with pytest.raises(IndyError) as e:
        await close_pool_ledger(handle)
    assert ErrorCode.PoolLedgerInvalidPoolHandle == e.value.error_code


@pytest.mark.asyncio
async def test_close_pool_ledger_works_for_reopen_after_close(cleanup_storage):
    handle = await pool.create_and_open_pool_ledger("pool_1")
    await close_pool_ledger(handle)
    handle = await open_pool_ledger("pool_1", None)
    assert handle is not None
    await close_pool_ledger(handle)


