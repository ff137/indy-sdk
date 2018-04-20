// NOTE: this file is generated by codegen/js.js

var capi = require('bindings')('indynodejs')
var wrapIndyCallback = require('./wrapIndyCallback')

function jsonify (val) {
  if (val === null || val === void 0) {
    return null
  }
  if (typeof val === 'string') {
    return val
  }
  return JSON.stringify(val)
}

var indy = {}

indy.issuerCreateSchema = function issuerCreateSchema (issuerDid, name, version, attrNames, cb) {
  cb = wrapIndyCallback(cb)
  capi.issuerCreateSchema(issuerDid, name, version, jsonify(attrNames), cb)
  return cb.promise
}

indy.issuerCreateAndStoreCredentialDef = function issuerCreateAndStoreCredentialDef (walletHandle, issuerDid, schema, tag, signatureType, config, cb) {
  cb = wrapIndyCallback(cb)
  capi.issuerCreateAndStoreCredentialDef(walletHandle, issuerDid, jsonify(schema), tag, signatureType, jsonify(config), cb)
  return cb.promise
}

indy.issuerCreateAndStoreRevocReg = function issuerCreateAndStoreRevocReg (walletHandle, issuerDid, revocDefType, tag, credDefId, config, tailsWriterHandle, cb) {
  cb = wrapIndyCallback(cb)
  capi.issuerCreateAndStoreRevocReg(walletHandle, issuerDid, revocDefType, tag, credDefId, jsonify(config), tailsWriterHandle, cb)
  return cb.promise
}

indy.issuerCreateCredentialOffer = function issuerCreateCredentialOffer (walletHandle, credDefId, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.issuerCreateCredentialOffer(walletHandle, credDefId, cb)
  return cb.promise
}

indy.issuerCreateCredential = function issuerCreateCredential (walletHandle, credOffer, credReq, credValues, revRegId, blobStorageReaderHandle, cb) {
  cb = wrapIndyCallback(cb)
  capi.issuerCreateCredential(walletHandle, jsonify(credOffer), jsonify(credReq), jsonify(credValues), revRegId, blobStorageReaderHandle, cb)
  return cb.promise
}

indy.issuerRevokeCredential = function issuerRevokeCredential (walletHandle, blobStorageReaderHandle, revRegId, credRevocId, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.issuerRevokeCredential(walletHandle, blobStorageReaderHandle, revRegId, credRevocId, cb)
  return cb.promise
}

indy.issuerMergeRevocationRegistryDeltas = function issuerMergeRevocationRegistryDeltas (revRegDelta, otherRevRegDelta, cb) {
  cb = wrapIndyCallback(cb)
  capi.issuerMergeRevocationRegistryDeltas(jsonify(revRegDelta), jsonify(otherRevRegDelta), cb)
  return cb.promise
}

indy.proverCreateMasterSecret = function proverCreateMasterSecret (walletHandle, masterSecretId, cb) {
  cb = wrapIndyCallback(cb)
  capi.proverCreateMasterSecret(walletHandle, masterSecretId, cb)
  return cb.promise
}

indy.proverCreateCredentialReq = function proverCreateCredentialReq (walletHandle, proverDid, credOffer, credDef, masterSecretId, cb) {
  cb = wrapIndyCallback(cb)
  capi.proverCreateCredentialReq(walletHandle, proverDid, jsonify(credOffer), jsonify(credDef), masterSecretId, cb)
  return cb.promise
}

indy.proverStoreCredential = function proverStoreCredential (walletHandle, credId, credReqMetadata, cred, credDef, revRegDef, cb) {
  cb = wrapIndyCallback(cb)
  capi.proverStoreCredential(walletHandle, credId, jsonify(credReqMetadata), jsonify(cred), jsonify(credDef), jsonify(revRegDef), cb)
  return cb.promise
}

indy.proverGetCredentials = function proverGetCredentials (walletHandle, filter, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.proverGetCredentials(walletHandle, jsonify(filter), cb)
  return cb.promise
}

indy.proverGetCredentialsForProofReq = function proverGetCredentialsForProofReq (walletHandle, proofRequest, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.proverGetCredentialsForProofReq(walletHandle, jsonify(proofRequest), cb)
  return cb.promise
}

indy.proverCreateProof = function proverCreateProof (walletHandle, proofReq, requestedCredentials, masterSecretName, schemas, credentialDefs, revStates, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.proverCreateProof(walletHandle, jsonify(proofReq), jsonify(requestedCredentials), masterSecretName, jsonify(schemas), jsonify(credentialDefs), jsonify(revStates), cb)
  return cb.promise
}

indy.verifierVerifyProof = function verifierVerifyProof (proofRequest, proof, schemas, credentialDefsJsons, revRegDefs, revRegs, cb) {
  cb = wrapIndyCallback(cb)
  capi.verifierVerifyProof(jsonify(proofRequest), jsonify(proof), jsonify(schemas), jsonify(credentialDefsJsons), jsonify(revRegDefs), jsonify(revRegs), cb)
  return cb.promise
}

indy.createRevocationState = function createRevocationState (blobStorageReaderHandle, revRegDef, revRegDelta, timestamp, credRevId, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.createRevocationState(blobStorageReaderHandle, jsonify(revRegDef), jsonify(revRegDelta), timestamp, credRevId, cb)
  return cb.promise
}

indy.updateRevocationState = function updateRevocationState (blobStorageReaderHandle, revState, revRegDef, revRegDelta, timestamp, credRevId, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.updateRevocationState(blobStorageReaderHandle, jsonify(revState), jsonify(revRegDef), jsonify(revRegDelta), timestamp, credRevId, cb)
  return cb.promise
}

indy.openBlobStorageReader = function openBlobStorageReader (type, config, cb) {
  cb = wrapIndyCallback(cb)
  capi.openBlobStorageReader(type, jsonify(config), cb)
  return cb.promise
}

indy.openBlobStorageWriter = function openBlobStorageWriter (type, config, cb) {
  cb = wrapIndyCallback(cb)
  capi.openBlobStorageWriter(type, jsonify(config), cb)
  return cb.promise
}

indy.createKey = function createKey (walletHandle, key, cb) {
  cb = wrapIndyCallback(cb)
  capi.createKey(walletHandle, jsonify(key), cb)
  return cb.promise
}

indy.setKeyMetadata = function setKeyMetadata (walletHandle, verkey, metadata, cb) {
  cb = wrapIndyCallback(cb)
  capi.setKeyMetadata(walletHandle, verkey, metadata, cb)
  return cb.promise
}

indy.getKeyMetadata = function getKeyMetadata (walletHandle, verkey, cb) {
  cb = wrapIndyCallback(cb)
  capi.getKeyMetadata(walletHandle, verkey, cb)
  return cb.promise
}

indy.cryptoSign = function cryptoSign (walletHandle, signerVk, messageRaw, cb) {
  cb = wrapIndyCallback(cb)
  capi.cryptoSign(walletHandle, signerVk, messageRaw, cb)
  return cb.promise
}

indy.cryptoVerify = function cryptoVerify (signerVk, messageRaw, signatureRaw, cb) {
  cb = wrapIndyCallback(cb)
  capi.cryptoVerify(signerVk, messageRaw, signatureRaw, cb)
  return cb.promise
}

indy.cryptoAuthCrypt = function cryptoAuthCrypt (walletHandle, senderVk, recipientVk, messageRaw, cb) {
  cb = wrapIndyCallback(cb)
  capi.cryptoAuthCrypt(walletHandle, senderVk, recipientVk, messageRaw, cb)
  return cb.promise
}

indy.cryptoAuthDecrypt = function cryptoAuthDecrypt (walletHandle, recipientVk, encryptedMsgRaw, cb) {
  cb = wrapIndyCallback(cb)
  capi.cryptoAuthDecrypt(walletHandle, recipientVk, encryptedMsgRaw, cb)
  return cb.promise
}

indy.cryptoAnonCrypt = function cryptoAnonCrypt (recipientVk, messageRaw, cb) {
  cb = wrapIndyCallback(cb)
  capi.cryptoAnonCrypt(recipientVk, messageRaw, cb)
  return cb.promise
}

indy.cryptoAnonDecrypt = function cryptoAnonDecrypt (walletHandle, recipientVk, encryptedMsg, cb) {
  cb = wrapIndyCallback(cb)
  capi.cryptoAnonDecrypt(walletHandle, recipientVk, encryptedMsg, cb)
  return cb.promise
}

indy.createAndStoreMyDid = function createAndStoreMyDid (walletHandle, did, cb) {
  cb = wrapIndyCallback(cb)
  capi.createAndStoreMyDid(walletHandle, jsonify(did), cb)
  return cb.promise
}

indy.replaceKeysStart = function replaceKeysStart (walletHandle, did, identity, cb) {
  cb = wrapIndyCallback(cb)
  capi.replaceKeysStart(walletHandle, did, jsonify(identity), cb)
  return cb.promise
}

indy.replaceKeysApply = function replaceKeysApply (walletHandle, did, cb) {
  cb = wrapIndyCallback(cb)
  capi.replaceKeysApply(walletHandle, did, cb)
  return cb.promise
}

indy.storeTheirDid = function storeTheirDid (walletHandle, identity, cb) {
  cb = wrapIndyCallback(cb)
  capi.storeTheirDid(walletHandle, jsonify(identity), cb)
  return cb.promise
}

indy.keyForDid = function keyForDid (poolHandle, walletHandle, did, cb) {
  cb = wrapIndyCallback(cb)
  capi.keyForDid(poolHandle, walletHandle, did, cb)
  return cb.promise
}

indy.keyForLocalDid = function keyForLocalDid (walletHandle, did, cb) {
  cb = wrapIndyCallback(cb)
  capi.keyForLocalDid(walletHandle, did, cb)
  return cb.promise
}

indy.setEndpointForDid = function setEndpointForDid (walletHandle, did, address, transportKey, cb) {
  cb = wrapIndyCallback(cb)
  capi.setEndpointForDid(walletHandle, did, address, transportKey, cb)
  return cb.promise
}

indy.getEndpointForDid = function getEndpointForDid (walletHandle, poolHandle, did, cb) {
  cb = wrapIndyCallback(cb)
  capi.getEndpointForDid(walletHandle, poolHandle, did, cb)
  return cb.promise
}

indy.setDidMetadata = function setDidMetadata (walletHandle, did, metadata, cb) {
  cb = wrapIndyCallback(cb)
  capi.setDidMetadata(walletHandle, did, metadata, cb)
  return cb.promise
}

indy.getDidMetadata = function getDidMetadata (walletHandle, did, cb) {
  cb = wrapIndyCallback(cb)
  capi.getDidMetadata(walletHandle, did, cb)
  return cb.promise
}

indy.getMyDidWithMeta = function getMyDidWithMeta (walletHandle, myDid, cb) {
  cb = wrapIndyCallback(cb)
  capi.getMyDidWithMeta(walletHandle, myDid, cb)
  return cb.promise
}

indy.listMyDidsWithMeta = function listMyDidsWithMeta (walletHandle, cb) {
  cb = wrapIndyCallback(cb)
  capi.listMyDidsWithMeta(walletHandle, cb)
  return cb.promise
}

indy.abbreviateVerkey = function abbreviateVerkey (did, fullVerkey, cb) {
  cb = wrapIndyCallback(cb)
  capi.abbreviateVerkey(did, fullVerkey, cb)
  return cb.promise
}

indy.signAndSubmitRequest = function signAndSubmitRequest (poolHandle, walletHandle, submitterDid, request, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.signAndSubmitRequest(poolHandle, walletHandle, submitterDid, jsonify(request), cb)
  return cb.promise
}

indy.submitRequest = function submitRequest (poolHandle, request, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.submitRequest(poolHandle, jsonify(request), cb)
  return cb.promise
}

indy.signRequest = function signRequest (walletHandle, submitterDid, request, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.signRequest(walletHandle, submitterDid, jsonify(request), cb)
  return cb.promise
}

indy.buildGetDdoRequest = function buildGetDdoRequest (submitterDid, targetDid, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.buildGetDdoRequest(submitterDid, targetDid, cb)
  return cb.promise
}

indy.buildNymRequest = function buildNymRequest (submitterDid, targetDid, verkey, alias, role, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.buildNymRequest(submitterDid, targetDid, verkey, alias, role, cb)
  return cb.promise
}

indy.buildAttribRequest = function buildAttribRequest (submitterDid, targetDid, hash, raw, enc, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.buildAttribRequest(submitterDid, targetDid, hash, raw, enc, cb)
  return cb.promise
}

indy.buildGetAttribRequest = function buildGetAttribRequest (submitterDid, targetDid, hash, raw, enc, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.buildGetAttribRequest(submitterDid, targetDid, hash, raw, enc, cb)
  return cb.promise
}

indy.buildGetNymRequest = function buildGetNymRequest (submitterDid, targetDid, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.buildGetNymRequest(submitterDid, targetDid, cb)
  return cb.promise
}

indy.buildSchemaRequest = function buildSchemaRequest (submitterDid, data, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.buildSchemaRequest(submitterDid, data, cb)
  return cb.promise
}

indy.buildGetSchemaRequest = function buildGetSchemaRequest (submitterDid, id, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.buildGetSchemaRequest(submitterDid, id, cb)
  return cb.promise
}

indy.parseGetSchemaResponse = function parseGetSchemaResponse (getSchemaResponse, cb) {
  cb = wrapIndyCallback(cb)
  capi.parseGetSchemaResponse(getSchemaResponse, cb)
  return cb.promise
}

indy.buildCredDefRequest = function buildCredDefRequest (submitterDid, data, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.buildCredDefRequest(submitterDid, data, cb)
  return cb.promise
}

indy.buildGetCredDefRequest = function buildGetCredDefRequest (submitterDid, id, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.buildGetCredDefRequest(submitterDid, id, cb)
  return cb.promise
}

indy.parseGetCredDefResponse = function parseGetCredDefResponse (getCredDefResponse, cb) {
  cb = wrapIndyCallback(cb)
  capi.parseGetCredDefResponse(getCredDefResponse, cb)
  return cb.promise
}

indy.buildNodeRequest = function buildNodeRequest (submitterDid, targetDid, data, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.buildNodeRequest(submitterDid, targetDid, data, cb)
  return cb.promise
}

indy.buildGetTxnRequest = function buildGetTxnRequest (submitterDid, data, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.buildGetTxnRequest(submitterDid, data, cb)
  return cb.promise
}

indy.buildPoolConfigRequest = function buildPoolConfigRequest (submitterDid, writes, force, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.buildPoolConfigRequest(submitterDid, writes, force, cb)
  return cb.promise
}

indy.buildPoolRestartRequest = function buildPoolRestartRequest (submitterDid, action, datetime, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.buildPoolRestartRequest(submitterDid, action, datetime, cb)
  return cb.promise
}

indy.buildPoolUpgradeRequest = function buildPoolUpgradeRequest (submitterDid, name, version, action, sha256, timeout, schedule, justification, reinstall, force, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.buildPoolUpgradeRequest(submitterDid, name, version, action, sha256, timeout, schedule, justification, reinstall, force, cb)
  return cb.promise
}

indy.buildRevocRegDefRequest = function buildRevocRegDefRequest (submitterDid, data, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.buildRevocRegDefRequest(submitterDid, data, cb)
  return cb.promise
}

indy.buildGetRevocRegDefRequest = function buildGetRevocRegDefRequest (submitterDid, id, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.buildGetRevocRegDefRequest(submitterDid, id, cb)
  return cb.promise
}

indy.parseGetRevocRegDefResponse = function parseGetRevocRegDefResponse (getRevocRefDefResponse, cb) {
  cb = wrapIndyCallback(cb)
  capi.parseGetRevocRegDefResponse(getRevocRefDefResponse, cb)
  return cb.promise
}

indy.buildRevocRegEntryRequest = function buildRevocRegEntryRequest (submitterDid, revocRegDefId, revDefType, value, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.buildRevocRegEntryRequest(submitterDid, revocRegDefId, revDefType, value, cb)
  return cb.promise
}

indy.buildGetRevocRegRequest = function buildGetRevocRegRequest (submitterDid, revocRegDefId, timestamp, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.buildGetRevocRegRequest(submitterDid, revocRegDefId, timestamp, cb)
  return cb.promise
}

indy.parseGetRevocRegResponse = function parseGetRevocRegResponse (getRevocRegResponse, cb) {
  cb = wrapIndyCallback(cb)
  capi.parseGetRevocRegResponse(getRevocRegResponse, cb)
  return cb.promise
}

indy.buildGetRevocRegDeltaRequest = function buildGetRevocRegDeltaRequest (submitterDid, revocRegDefId, from, to, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.buildGetRevocRegDeltaRequest(submitterDid, revocRegDefId, from, to, cb)
  return cb.promise
}

indy.parseGetRevocRegDeltaResponse = function parseGetRevocRegDeltaResponse (getRevocRegDeltaResponse, cb) {
  cb = wrapIndyCallback(cb)
  capi.parseGetRevocRegDeltaResponse(getRevocRegDeltaResponse, cb)
  return cb.promise
}

indy.isPairwiseExists = function isPairwiseExists (walletHandle, theirDid, cb) {
  cb = wrapIndyCallback(cb)
  capi.isPairwiseExists(walletHandle, theirDid, cb)
  return cb.promise
}

indy.createPairwise = function createPairwise (walletHandle, theirDid, myDid, metadata, cb) {
  cb = wrapIndyCallback(cb)
  capi.createPairwise(walletHandle, theirDid, myDid, metadata, cb)
  return cb.promise
}

indy.listPairwise = function listPairwise (walletHandle, cb) {
  cb = wrapIndyCallback(cb)
  capi.listPairwise(walletHandle, cb)
  return cb.promise
}

indy.getPairwise = function getPairwise (walletHandle, theirDid, cb) {
  cb = wrapIndyCallback(cb, true)
  capi.getPairwise(walletHandle, theirDid, cb)
  return cb.promise
}

indy.setPairwiseMetadata = function setPairwiseMetadata (walletHandle, theirDid, metadata, cb) {
  cb = wrapIndyCallback(cb)
  capi.setPairwiseMetadata(walletHandle, theirDid, metadata, cb)
  return cb.promise
}

indy.createPoolLedgerConfig = function createPoolLedgerConfig (configName, config, cb) {
  cb = wrapIndyCallback(cb)
  capi.createPoolLedgerConfig(configName, jsonify(config), cb)
  return cb.promise
}

indy.openPoolLedger = function openPoolLedger (configName, config, cb) {
  cb = wrapIndyCallback(cb)
  capi.openPoolLedger(configName, config, cb)
  return cb.promise
}

indy.refreshPoolLedger = function refreshPoolLedger (handle, cb) {
  cb = wrapIndyCallback(cb)
  capi.refreshPoolLedger(handle, cb)
  return cb.promise
}

indy.listPools = function listPools (cb) {
  cb = wrapIndyCallback(cb, true)
  capi.listPools(cb)
  return cb.promise
}

indy.closePoolLedger = function closePoolLedger (handle, cb) {
  cb = wrapIndyCallback(cb)
  capi.closePoolLedger(handle, cb)
  return cb.promise
}

indy.deletePoolLedgerConfig = function deletePoolLedgerConfig (configName, cb) {
  cb = wrapIndyCallback(cb)
  capi.deletePoolLedgerConfig(configName, cb)
  return cb.promise
}

indy.createWallet = function createWallet (poolName, name, xtype, config, credentials, cb) {
  cb = wrapIndyCallback(cb)
  capi.createWallet(poolName, name, xtype, config, credentials, cb)
  return cb.promise
}

indy.openWallet = function openWallet (name, runtimeConfig, credentials, cb) {
  cb = wrapIndyCallback(cb)
  capi.openWallet(name, runtimeConfig, credentials, cb)
  return cb.promise
}

indy.listWallets = function listWallets (cb) {
  cb = wrapIndyCallback(cb, true)
  capi.listWallets(cb)
  return cb.promise
}

indy.closeWallet = function closeWallet (handle, cb) {
  cb = wrapIndyCallback(cb)
  capi.closeWallet(handle, cb)
  return cb.promise
}

indy.deleteWallet = function deleteWallet (name, credentials, cb) {
  cb = wrapIndyCallback(cb)
  capi.deleteWallet(name, credentials, cb)
  return cb.promise
}

module.exports = indy
