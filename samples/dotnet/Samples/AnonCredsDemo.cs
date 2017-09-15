﻿using Hyperledger.Indy.AnonCredsApi;
using Hyperledger.Indy.PoolApi;
using Hyperledger.Indy.Samples.Utils;
using Hyperledger.Indy.WalletApi;
using Newtonsoft.Json.Linq;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Text;
using System.Threading.Tasks;

namespace Hyperledger.Indy.Samples
{
    static class AnonCredsDemo
    {
        public static async Task Execute()
        {
            var issuerWalletName = "issuerWallet";
            var proverWalletName = "proverWallet";

            //1. Create and Open Pool
            await PoolUtils.CreatePoolLedgerConfig();
            var pool = await Pool.OpenPoolLedgerAsync(PoolUtils.DEFAULT_POOL_NAME, "{}");

            //2. Issuer Create and Open Wallet
            await WalletUtils.CreateWalleatAsync(PoolUtils.DEFAULT_POOL_NAME, issuerWalletName, "default", null, null);
            var issuerWallet = await Wallet.OpenWalletAsync(issuerWalletName, null, null);

            //3. Prover Create and Open Wallet
            await WalletUtils.CreateWalleatAsync(PoolUtils.DEFAULT_POOL_NAME, proverWalletName, "default", null, null);
            var proverWallet = await Wallet.OpenWalletAsync(proverWalletName, null, null);

            //4. Issuer create ClaimDef
            var schemaJson = "{\n" +
                    "   \"seqNo\":1,\n" +
                    "   \"data\": {\n" +
                    "       \"name\":\"gvt\",\n" +
                    "       \"version\":\"1.0\",\n" +
                    "       \"keys\":[\"age\",\"sex\",\"height\",\"name\"]\n" +
                    "   }\n" +
                    "}";
            var issuerDid = "NcYxiDXkpYi6ov5FcYDi1e";

            var claimDef = await AnonCreds.IssuerCreateAndStoreClaimDefAsync(issuerWallet, issuerDid, schemaJson, null, false);

            //5. Prover create Master Secret
            var masterSecret = "masterSecretName";
            await AnonCreds.ProverCreateMasterSecretAsync(proverWallet, masterSecret);

            //6. Prover store Claim Offer
            var claimOffer = string.Format("{{\"issuer_did\":\"{0}\", \"schema_seq_no\":{1}}}", issuerDid, 1);
            await AnonCreds.ProverStoreClaimOfferAsync(proverWallet, claimOffer);

            //7. Prover get Claim Offers
            var claimOfferFilter = string.Format("{{\"issuer_did\":\"{0}\"}}", issuerDid);
            var claimOffersJson = await AnonCreds.ProverGetClaimOffersAsync(proverWallet, claimOfferFilter);

            var claimOffersObject = JArray.Parse(claimOffersJson);
            Debug.Assert(claimOffersObject.Count == 1);

            var claimOfferObject = (JObject)claimOffersObject[0];
            var claimOfferJson = claimOfferObject.ToString();

            //8. Prover create ClaimReq
            var proverDid = "BzfFCYk";
            var claimReq = await AnonCreds.ProverCreateAndStoreClaimReqAsync(proverWallet, proverDid, claimOfferJson, claimDef, masterSecret);
            Debug.Assert(claimReq != null);

            //9. Issuer create Claim
            var claimAttributesJson = "{\n" +
                    "   \"sex\":[\"male\",\"5944657099558967239210949258394887428692050081607692519917050011144233115103\"],\n" +
                    "   \"name\":[\"Alex\",\"1139481716457488690172217916278103335\"],\n" +
                    "   \"height\":[\"175\",\"175\"],\n" +
                    "   \"age\":[\"28\",\"28\"]\n" +
                    "}";

            var createClaimResult = await AnonCreds.IssuerCreateClaimAsync(issuerWallet, claimReq, claimAttributesJson, -1);
            var claimJson = createClaimResult.ClaimJson;

            //10. Prover store Claim
            await AnonCreds.ProverStoreClaimAsync(proverWallet, claimJson);

            //11. Prover gets Claims for Proof Request
            var proofRequestJson = "{\n" +
                    "   \"nonce\":\"123432421212\",\n" +
                    "   \"name\":\"proof_req_1\",\n" +
                    "   \"version\":\"0.1\",\n" +
                    "   \"requested_attrs\":{\"attr1_uuid\":{\"schema_seq_no\":1,\"name\":\"name\"},\n" +
                    "       \"attr2_uuid\":{\"schema_seq_no\":1,\"name\":\"sex\"}},\n" +
                    "   \"requested_predicates\":{\"predicate1_uuid\":{\"attr_name\":\"age\",\"p_type\":\"GE\",\"value\":18}}\n" +
                    "   }";

            var claimsForProofJson = await AnonCreds.ProverGetClaimsForProofReqAsync(proverWallet, proofRequestJson);

            var claimsForProof = JObject.Parse(claimsForProofJson);
            var claimsForAttribute1 = (JArray)claimsForProof["attrs"]["attr1_uuid"];
            var claimsForAttribute2 = (JArray)claimsForProof["attrs"]["attr1_uuid"];
            var claimsForPredicate = (JArray)claimsForProof["predicates"]["predicate1_uuid"];

            Debug.Assert(claimsForAttribute1.Count == 1);
            Debug.Assert(claimsForAttribute2.Count == 1);
            Debug.Assert(claimsForPredicate.Count == 1);

            var claimUuid = claimsForAttribute1[0].Value<string>("claim_uuid");

            //12. Prover create Proof
            var selfAttestedValue = "yes";
            var requestedClaimsJson = string.Format("{{\n" +
                    "   \"self_attested_attributes\":{{\"self1\":\"{0}\"}},\n" +
                    "   \"requested_attrs\":{{\"attr1_uuid\":[\"{1}\", true],\n" +
                    "   \"attr2_uuid\":[\"{2}\", false]}},\n" +
                    "   \"requested_predicates\":{{\"predicate1_uuid\":\"{3}\"}}\n" +
                    "}}", selfAttestedValue, claimUuid, claimUuid, claimUuid);

            var schemasJson = string.Format("{{\"{0}\":{1}}}", claimUuid, schemaJson);
            var claimDefsJson = string.Format("{{\"{0}\":{1}}}", claimUuid, claimDef);
            var revocRegsJson = "{}";


            var proofJson = await AnonCreds.ProverCreateProofAsync(proverWallet, proofRequestJson, requestedClaimsJson, schemasJson,
                    masterSecret, claimDefsJson, revocRegsJson);

            var proof = JObject.Parse(proofJson);

            //13. Verifier verify Proof
            Debug.Assert(string.Equals("Alex", proof["requested_proof"]["revealed_attrs"]["attr1_uuid"][1].ToString()));
            Debug.Assert(proof["requested_proof"]["unrevealed_attrs"].Value<string>("attr2_uuid") != null);
            Debug.Assert(string.Equals(selfAttestedValue, proof["requested_proof"]["self_attested_attrs"].Value<string>("self1")));

            var valid = await AnonCreds.VerifierVerifyProofAsync(proofRequestJson, proofJson, schemasJson, claimDefsJson, revocRegsJson);
            Debug.Assert(valid == true);

            //14. Close and Delete issuer wallet
            await issuerWallet.CloseAsync();
            await Wallet.DeleteWalletAsync(issuerWalletName, null);

            //15. Close and Delete prover wallet
            await proverWallet.CloseAsync();
            await Wallet.DeleteWalletAsync(proverWalletName, null);

            //16. Close pool
            await pool.CloseAsync();

            //17. Delete Pool ledger config
            await Pool.DeletePoolLedgerConfigAsync(PoolUtils.DEFAULT_POOL_NAME);

            Console.WriteLine("Anoncreds sample -> completed");
        }
    }
}
