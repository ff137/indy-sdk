from indy_sdk.anoncreds import prover_get_claims_for_proof_req, prover_create_proof, prover_get_claims
from indy_sdk.error import ErrorCode, IndyError

import json
import pytest


@pytest.mark.asyncio
async def test_prover_create_proof_works(wallet_handle, prepopulated_wallet, gvt_schema, master_secret_name,
                                         schema_seq_no):
    claim_def_json, = prepopulated_wallet

    proof_req = {
        "nonce": "123432421212",
        "name": "proof_req_1",
        "version": "0.1",
        "requested_attrs": {
            "attr1_uuid": {
                "schema_seq_no": schema_seq_no,
                "name": "name"
            }
        },
        "requested_predicates": {
            "predicate1_uuid": {
                "attr_name": "age",
                "p_type": "GE",
                "value": 18
            }
        }
    }

    claims = json.loads(await prover_get_claims_for_proof_req(wallet_handle, json.dumps(proof_req)))
    claim_for_attr = claims['attrs']['attr1_uuid'][0]['claim_uuid']
    claim_for_predicate = claims['predicates']['predicate1_uuid'][0]['claim_uuid']

    requested_claims = {
        "self_attested_attributes": {},
        "requested_attrs": {
            "attr1_uuid": [claim_for_attr, True]
        },
        "requested_predicates": {
            "predicate1_uuid": claim_for_predicate
        }
    }

    schemas = {
        claim_for_attr: gvt_schema
    }

    claim_defs = {
        claim_for_attr: json.loads(claim_def_json)
    }

    await prover_create_proof(wallet_handle, json.dumps(proof_req), json.dumps(requested_claims),
                              json.dumps(schemas), master_secret_name,
                              json.dumps(claim_defs), "{}")


@pytest.mark.asyncio
async def test_prover_create_proof_works_for_using_not_satisfy_claim(wallet_handle, prepopulated_wallet, gvt_schema,
                                                                     master_secret_name,
                                                                     schema_seq_no):
    claim_def_json, = prepopulated_wallet
    claims = json.loads(await prover_get_claims(wallet_handle, "{}"))
    claim_uuid = claims[0]['claim_uuid']

    proof_req = {
        "nonce": "123432421212",
        "name": "proof_req_1",
        "version": "0.1",
        "requested_attrs": {
            "attr1_uuid": {
                "schema_seq_no": schema_seq_no,
                "name": "some_attr"
            }
        },
        "requested_predicates": {}
    }

    requested_claims = {
        "self_attested_attributes": {},
        "requested_attrs": {
            "attr1_uuid": [claim_uuid, True]
        },
        "requested_predicates": {
            "predicate1_uuid": {}
        }
    }

    schemas = {
        claim_uuid: gvt_schema
    }

    claim_defs = {
        claim_uuid: json.loads(claim_def_json)
    }

    with pytest.raises(IndyError) as e:
        await prover_create_proof(wallet_handle, json.dumps(proof_req), json.dumps(requested_claims),
                                  json.dumps(schemas), master_secret_name,
                                  json.dumps(claim_defs), "{}")

    assert ErrorCode.CommonInvalidStructure == e.value.error_code


@pytest.mark.asyncio
async def test_prover_create_proof_works_for_invalid_wallet_handle(wallet_handle, prepopulated_wallet, gvt_schema,
                                                                   master_secret_name,
                                                                   schema_seq_no):
    claim_def_json, = prepopulated_wallet

    proof_req = {
        "nonce": "123432421212",
        "name": "proof_req_1",
        "version": "0.1",
        "requested_attrs": {
            "attr1_uuid": {
                "schema_seq_no": schema_seq_no,
                "name": "name"
            }
        },
        "requested_predicates": {
            "predicate1_uuid": {
                "attr_name": "age",
                "p_type": "GE",
                "value": 18
            }
        }
    }

    claims = json.loads(await prover_get_claims_for_proof_req(wallet_handle, json.dumps(proof_req)))
    claim_for_attr = claims['attrs']['attr1_uuid'][0]['claim_uuid']
    claim_for_predicate = claims['predicates']['predicate1_uuid'][0]['claim_uuid']

    requested_claims = {
        "self_attested_attributes": {},
        "requested_attrs": {
            "attr1_uuid": [claim_for_attr, True]
        },
        "requested_predicates": {
            "predicate1_uuid": claim_for_predicate
        }
    }

    schemas = {
        claim_for_attr: gvt_schema
    }

    claim_defs = {
        claim_for_attr: json.loads(claim_def_json)
    }

    invalid_wallet_handle = wallet_handle + 100

    with pytest.raises(IndyError) as e:
        await prover_create_proof(invalid_wallet_handle, json.dumps(proof_req), json.dumps(requested_claims),
                                  json.dumps(schemas), master_secret_name,
                                  json.dumps(claim_defs), "{}")

    assert ErrorCode.WalletInvalidHandle == e.value.error_code
