//
//  Indy_demoTests.m
//  Indy-demoTests
//

#import <Foundation/Foundation.h>
#import <XCTest/XCTest.h>
#import <Indy/Indy.h>
#import "TestUtils.h"
#import "WalletUtils.h"
#import "NSDictionary+JSON.h"
#import "AnoncredsUtils.h"
#import "BlobStorageUtils.h"

@interface AnoncredsDemo : XCTestCase

@end

@implementation AnoncredsDemo

- (void)setUp {
    [super setUp];
    // Put setup code here. This method is called before the invocation of each test method in the class.
}

- (void)tearDown {
    // Put teardown code here. This method is called after the invocation of each test method in the class.
    [super tearDown];
}

- (void)testAnoncredsDemo {
    [TestUtils cleanupStorage];

    XCTestExpectation *completionExpectation = [[XCTestExpectation alloc] initWithDescription:@"completion finished"];

    NSError *ret;

    // 1. Create wallet
    NSString *walletName = @"issuer_wallet";
    IndyHandle walletHandle = 0;
    ret = [[WalletUtils sharedInstance] createWalletWithPoolName:[TestUtils pool]
                                                      walletName:walletName
                                                           xtype:[TestUtils defaultType]
                                                          config:nil];
    XCTAssertEqual(ret.code, Success, @"WalletUtils::createWalletWithPoolName() failed!");

    // 2. Open wallet
    ret = [[WalletUtils sharedInstance] openWalletWithName:walletName
                                                    config:nil
                                                 outHandle:&walletHandle];
    XCTAssertEqual(ret.code, Success, @"WalletUtils::openWalletWithName() failed!");

    //3. Issuer create Schema
    __block NSString *schemaId;
    __block NSString *schemaJson;
    ret = [[AnoncredsUtils sharedInstance] issuerCreateSchemaWithName:[TestUtils gvtSchemaName]
                                                              version:[TestUtils schemaVersion]
                                                                attrs:[TestUtils gvtSchemaAttrs]
                                                            issuerDID:[TestUtils issuerDid]
                                                             schemaId:&schemaId
                                                           schemaJson:&schemaJson];
    XCTAssertEqual(ret.code, Success, @"issuerCreateSchemaForIssuerDID failed");

    XCTAssertTrue([schemaId isValid], @"invalid schemaId: %@", schemaId);
    XCTAssertTrue([schemaJson isValid], @"invalid schemaJson: %@", schemaJson);

    // 4. Issuer create Credential Definition for Schema
    __block NSString *credentialDefId;
    __block NSString *credentialDefJSON;
    ret = [[AnoncredsUtils sharedInstance] issuerCreateAndStoreCredentialDefForSchema:schemaJson
                                                                            issuerDID:[TestUtils issuerDid]
                                                                                  tag:[TestUtils tag]
                                                                                 type:nil
                                                                           configJSON:[[AnoncredsUtils sharedInstance] defaultCredentialDefConfig]
                                                                         walletHandle:walletHandle
                                                                            credDefId:&credentialDefId
                                                                          credDefJson:&credentialDefJSON];
    XCTAssertEqual(ret.code, Success, @"issuerCreateCredentialDefinitionWithWalletHandle failed");

    XCTAssertTrue([credentialDefId isValid], @"invalid credentialDefId: %@", credentialDefId);
    XCTAssertTrue([credentialDefJSON isValid], @"invalid credentialDefJSON: %@", credentialDefJSON);

    // 5. Prover create Master Secret
    ret = [[AnoncredsUtils sharedInstance] proverCreateMasterSecret:[TestUtils commonMasterSecretName]
                                                       walletHandle:walletHandle
                                                  outMasterSecretId:nil];

    XCTAssertEqual(ret.code, Success, @"proverCreateMasterSecret() failed!");

    // 6. Issuer create Credential Offer
    completionExpectation = [[XCTestExpectation alloc] initWithDescription:@"completion finished"];

    __block NSString *credentialOfferJSON = nil;
    ret = [[AnoncredsUtils sharedInstance] issuerCreateCredentialOfferForCredDefId:credentialDefId
                                                                      walletHandle:walletHandle
                                                                     credOfferJson:&credentialOfferJSON];
    XCTAssertEqual(ret.code, Success, @"AnoncredsUtils::issuerCreateAndStoreCredentialDef() failed!");

    // 7. Prover create Credential Request
    completionExpectation = [[XCTestExpectation alloc] initWithDescription:@"completion finished"];

    __block NSString *credentialReqJSON = nil;
    __block NSString *credentialReqMetadataJSON = nil;
    ret = [[AnoncredsUtils sharedInstance] proverCreateCredentialReqForCredentialOffer:credentialOfferJSON
                                                                     credentialDefJSON:credentialDefJSON
                                                                             proverDID:[TestUtils proverDid]
                                                                        masterSecretID:[TestUtils commonMasterSecretName]
                                                                          walletHandle:walletHandle
                                                                           credReqJson:&credentialReqJSON
                                                                   credReqMetadataJson:&credentialReqMetadataJSON];
    XCTAssertEqual(ret.code, Success, @"proverCreateAndStoreCredentialReq() failed!");

    // 8. Issuer create Credential for Credential Request
    completionExpectation = [[XCTestExpectation alloc] initWithDescription:@"completion finished"];

    __block NSString *credentialJSON = nil;
    ret = [[AnoncredsUtils sharedInstance] issuerCreateCredentialForCredentialRequest:credentialReqJSON
                                                                        credOfferJSON:credentialOfferJSON
                                                                       credValuesJSON:[[AnoncredsUtils sharedInstance] getGvtCredentialValuesJson]
                                                                             revRegId:nil
                                                              blobStorageReaderHandle:nil
                                                                         walletHandle:walletHandle
                                                                             credJson:&credentialJSON
                                                                          credRevocId:nil
                                                                    revocRegDeltaJSON:nil];
    XCTAssertEqual(ret.code, Success, @"issuerCreateCredential() failed!");

    // 9. Prover process and store Credential
    completionExpectation = [[XCTestExpectation alloc] initWithDescription:@"completion finished"];
    ret = [[AnoncredsUtils sharedInstance] proverStoreCredential:credentialJSON
                                                          credID:[[AnoncredsUtils sharedInstance] credentialId1]
                                                     credReqJSON:credentialReqJSON
                                             credReqMetadataJSON:credentialReqMetadataJSON
                                                     credDefJSON:credentialDefJSON
                                                   revRegDefJSON:nil
                                                    walletHandle:walletHandle
                                                       outCredId:nil];
    XCTAssertEqual(ret.code, Success, @"proverStoreCredential() failed!");

    // 10. Prover gets Credentials for Proof Request
    completionExpectation = [[XCTestExpectation alloc] initWithDescription:@"completion finished"];

    NSString *proofReqJSON = [NSString stringWithFormat:@"\
                              {"
            "\"nonce\":\"123432421212\","
            "\"name\":\"proof_req_1\","
            "\"version\":\"0.1\","
            "\"requested_attributes\":{\
                                    \"attr1_referent\":{\
                                        \"name\":\"name\"\
                                    }\
                              },\
                              \"requested_predicates\":{\
                                    \"predicate1_referent\":{\
                                        \"name\":\"age\",\
                                        \"p_type\":\">=\",\
                                        \"p_value\":18\
                                    }\
                              }\
                            }"];

    __block NSString *credentialsJson = nil;

    ret = [[AnoncredsUtils sharedInstance] proverGetCredentialsForProofReq:proofReqJSON
                                                              walletHandle:walletHandle
                                                           credentialsJson:&credentialsJson];
    XCTAssertEqual(ret.code, Success, @"proverGetCredentialsForProofReq() failed!");

    NSDictionary *credentials = [NSDictionary fromString:credentialsJson];
    XCTAssertTrue(credentials, @"serialization failed");

    NSDictionary *credentials_for_attr_1 = credentials[@"attrs"][@"attr1_referent"][0];
    XCTAssertTrue(credentials_for_attr_1, @"no object for key \"attr1_referent\"");
    NSString *credentialReferent = credentials_for_attr_1[@"cred_info"][@"referent"];

    // 11. Prover create Proof for Proof Request
    completionExpectation = [[XCTestExpectation alloc] initWithDescription:@"completion finished"];

    NSString *requestedCredentialsJSON = [NSString stringWithFormat:@"{\
                                     \"self_attested_attributes\":{},\
                                     \"requested_attributes\":{\"attr1_referent\":{\"cred_id\":\"%@\",\"revealed\":true}},\
                                     \"requested_predicates\":{\"predicate1_referent\":{\"cred_id\":\"%@\"}}\
                                     }", credentialReferent, credentialReferent];

    NSString *schemasJson = [[AnoncredsUtils sharedInstance] toJson:@{schemaId: [NSDictionary fromString:schemaJson]}];
    NSString *credentialDefsJson = [[AnoncredsUtils sharedInstance] toJson:@{credentialDefId: [NSDictionary fromString:credentialDefJSON]}];
    NSString *revocStatesJson = @"{}";

    NSString *proofJSON = nil;
    ret = [[AnoncredsUtils sharedInstance] proverCreateProofForRequest:proofReqJSON
                                              requestedCredentialsJSON:requestedCredentialsJSON
                                                        masterSecretID:[TestUtils commonMasterSecretName]
                                                           schemasJSON:schemasJson
                                                    credentialDefsJSON:credentialDefsJson
                                                       revocStatesJSON:revocStatesJson
                                                          walletHandle:walletHandle
                                                             proofJson:&proofJSON];
    XCTAssertEqual(ret.code, Success, @"proverCreateProof() failed!");

    // 12. Verifier verify proof
    NSDictionary *proof = [NSDictionary fromString:proofJSON];
    XCTAssertTrue(proof, @"serialization failed");

    NSDictionary *revealedAttr1 = proof[@"requested_proof"][@"revealed_attrs"][@"attr1_referent"];

    NSString *revocRegDefsJson = @"{}";
    NSString *revocRegsJson = @"{}";


    BOOL valid = false;
    ret = [[AnoncredsUtils sharedInstance] verifierVerifyProofRequest:proofReqJSON
                                                            proofJSON:proofJSON
                                                          schemasJSON:schemasJson
                                                   credentialDefsJSON:credentialDefsJson
                                                     revocRegDefsJSON:revocRegsJson
                                                        revocRegsJSON:revocRegsJson
                                                              isValid:&valid];
    XCTAssertEqual(ret.code, Success, @"verifierVerifyProof() failed!");
    XCTAssertTrue(valid);

    // 12. close wallet
    ret = [[WalletUtils sharedInstance] closeWalletWithHandle:walletHandle];
    XCTAssertEqual(ret.code, Success, @"closeWallet() failed!");

    [TestUtils cleanupStorage];
}

- (void)testAnoncredsDemoForKeychainWallet {
    [TestUtils cleanupStorage];
    [[IndyWallet sharedInstance] cleanupIndyKeychainWallet];

    XCTestExpectation *completionExpectation = [[XCTestExpectation alloc] initWithDescription:@"completion finished"];

    NSError *ret;

    // 0. register wallet type
    NSString *xType = @"keychain";
    ret = [[WalletUtils sharedInstance] registerWalletType:xType];

    // 1. Create wallet
    NSString *walletName = @"issuer_wallet";
    IndyHandle walletHandle = 0;
    ret = [[WalletUtils sharedInstance] createWalletWithPoolName:[TestUtils pool]
                                                      walletName:walletName
                                                           xtype:xType
                                                          config:nil];
    XCTAssertEqual(ret.code, Success, @"WalletUtils::createWalletWithPoolName() failed!");

    // 2. Open wallet
    ret = [[WalletUtils sharedInstance] openWalletWithName:walletName
                                                    config:nil
                                                 outHandle:&walletHandle];
    XCTAssertEqual(ret.code, Success, @"WalletUtils::openWalletWithName() failed!");

    //3. Issuer create Schema
    //3. Issuer create Schema
    __block NSString *schemaId;
    __block NSString *schemaJson;
    ret = [[AnoncredsUtils sharedInstance] issuerCreateSchemaWithName:[TestUtils gvtSchemaName]
                                                              version:[TestUtils schemaVersion]
                                                                attrs:[TestUtils gvtSchemaAttrs]
                                                            issuerDID:[TestUtils issuerDid]
                                                             schemaId:&schemaId
                                                           schemaJson:&schemaJson];
    XCTAssertEqual(ret.code, Success, @"issuerCreateSchemaForIssuerDID failed");

    XCTAssertTrue([schemaId isValid], @"invalid schemaId: %@", schemaId);
    XCTAssertTrue([schemaJson isValid], @"invalid schemaJson: %@", schemaJson);

    // 4. Issuer create Credential Definition for Schema
    __block NSString *credentialDefId;
    __block NSString *credentialDefJSON;
    ret = [[AnoncredsUtils sharedInstance] issuerCreateAndStoreCredentialDefForSchema:schemaJson
                                                                            issuerDID:[TestUtils issuerDid]
                                                                                  tag:[TestUtils tag]
                                                                                 type:nil
                                                                           configJSON:[[AnoncredsUtils sharedInstance] defaultCredentialDefConfig]
                                                                         walletHandle:walletHandle
                                                                            credDefId:&credentialDefId
                                                                          credDefJson:&credentialDefJSON];
    XCTAssertEqual(ret.code, Success, @"issuerCreateCredentialDefinitionWithWalletHandle failed");

    XCTAssertTrue([credentialDefId isValid], @"invalid credentialDefId: %@", credentialDefId);
    XCTAssertTrue([credentialDefJSON isValid], @"invalid credentialDefJSON: %@", credentialDefJSON);

    // 5. Prover create Master Secret
    ret = [[AnoncredsUtils sharedInstance] proverCreateMasterSecret:[TestUtils commonMasterSecretName]
                                                       walletHandle:walletHandle
                                                               outMasterSecretId:nil];
    XCTAssertEqual(ret.code, Success, @"proverCreateMasterSecret() failed!");

    // 6. Issuer create Credential Offer
    completionExpectation = [[XCTestExpectation alloc] initWithDescription:@"completion finished"];

    __block NSString *credentialOfferJSON = nil;
    ret = [[AnoncredsUtils sharedInstance] issuerCreateCredentialOfferForCredDefId:credentialDefId
                                                                      walletHandle:walletHandle
                                                                     credOfferJson:&credentialOfferJSON];
    XCTAssertEqual(ret.code, Success, @"AnoncredsUtils::issuerCreateAndStoreCredentialDef() failed!");

    // 7. Prover create Credential Request
    completionExpectation = [[XCTestExpectation alloc] initWithDescription:@"completion finished"];

    __block NSString *credentialReqJSON = nil;
    __block NSString *credentialReqMetadataJSON = nil;
    ret = [[AnoncredsUtils sharedInstance] proverCreateCredentialReqForCredentialOffer:credentialOfferJSON
                                                                     credentialDefJSON:credentialDefJSON
                                                                             proverDID:[TestUtils proverDid]
                                                                        masterSecretID:[TestUtils commonMasterSecretName]
                                                                          walletHandle:walletHandle
                                                                           credReqJson:&credentialReqJSON
                                                                   credReqMetadataJson:&credentialReqMetadataJSON];
    XCTAssertEqual(ret.code, Success, @"proverCreateAndStoreCredentialReq() failed!");

    // 8. Issuer create Credential for Credential Request
    completionExpectation = [[XCTestExpectation alloc] initWithDescription:@"completion finished"];

    __block NSString *credentialJSON = nil;
    ret = [[AnoncredsUtils sharedInstance] issuerCreateCredentialForCredentialRequest:credentialReqJSON
                                                                        credOfferJSON:credentialOfferJSON
                                                                       credValuesJSON:[[AnoncredsUtils sharedInstance] getGvtCredentialValuesJson]
                                                                             revRegId:nil
                                                              blobStorageReaderHandle:nil
                                                                         walletHandle:walletHandle
                                                                             credJson:&credentialJSON
                                                                          credRevocId:nil
                                                                    revocRegDeltaJSON:nil];
    XCTAssertEqual(ret.code, Success, @"issuerCreateCredential() failed!");

    // 9. Prover process and store Credential
    completionExpectation = [[XCTestExpectation alloc] initWithDescription:@"completion finished"];
    ret = [[AnoncredsUtils sharedInstance] proverStoreCredential:credentialJSON
                                                          credID:[[AnoncredsUtils sharedInstance] credentialId1]
                                                     credReqJSON:credentialReqJSON
                                             credReqMetadataJSON:credentialReqMetadataJSON
                                                     credDefJSON:credentialDefJSON
                                                   revRegDefJSON:nil
                                                    walletHandle:walletHandle
                                                       outCredId:nil];
    XCTAssertEqual(ret.code, Success, @"proverStoreCredential() failed!");

    // 10. Prover gets Credentials for Proof Request
    completionExpectation = [[XCTestExpectation alloc] initWithDescription:@"completion finished"];

    NSString *proofReqJSON = [NSString stringWithFormat:@"\
                              {"
            "\"nonce\":\"123432421212\","
            "\"name\":\"proof_req_1\","
            "\"version\":\"0.1\","
            "\"requested_attributes\":{\
                                    \"attr1_referent\":{\
                                        \"name\":\"name\"\
                                    }\
                              },\
                              \"requested_predicates\":{\
                                    \"predicate1_referent\":{\
                                        \"name\":\"age\",\
                                        \"p_type\":\">=\",\
                                        \"p_value\":18\
                                    }\
                              }\
                            }"];

    __block NSString *credentialsJson = nil;

    ret = [[AnoncredsUtils sharedInstance] proverGetCredentialsForProofReq:proofReqJSON
                                                              walletHandle:walletHandle
                                                           credentialsJson:&credentialsJson];
    XCTAssertEqual(ret.code, Success, @"proverGetCredentialsForProofReq() failed!");

    NSDictionary *credentials = [NSDictionary fromString:credentialsJson];
    XCTAssertTrue(credentials, @"serialization failed");

    NSDictionary *credentials_for_attr_1 = credentials[@"attrs"][@"attr1_referent"][0];
    XCTAssertTrue(credentials_for_attr_1, @"no object for key \"attr1_referent\"");
    NSString *credentialReferent = credentials_for_attr_1[@"cred_info"][@"referent"];

    // 11. Prover create Proof for Proof Request
    completionExpectation = [[XCTestExpectation alloc] initWithDescription:@"completion finished"];

    NSString *requestedCredentialsJSON = [NSString stringWithFormat:@"{\
                                     \"self_attested_attributes\":{},\
                                     \"requested_attributes\":{\"attr1_referent\":{\"cred_id\":\"%@\",\"revealed\":true}},\
                                     \"requested_predicates\":{\"predicate1_referent\":{\"cred_id\":\"%@\"}}\
                                     }", credentialReferent, credentialReferent];

    NSString *schemasJson = [[AnoncredsUtils sharedInstance] toJson:@{schemaId: [NSDictionary fromString:schemaJson]}];
    NSString *credentialDefsJson = [[AnoncredsUtils sharedInstance] toJson:@{credentialDefId: [NSDictionary fromString:credentialDefJSON]}];
    NSString *revocStatesJson = @"{}";

    NSString *proofJSON = nil;
    ret = [[AnoncredsUtils sharedInstance] proverCreateProofForRequest:proofReqJSON
                                              requestedCredentialsJSON:requestedCredentialsJSON
                                                        masterSecretID:[TestUtils commonMasterSecretName]
                                                           schemasJSON:schemasJson
                                                    credentialDefsJSON:credentialDefsJson
                                                       revocStatesJSON:revocStatesJson
                                                          walletHandle:walletHandle
                                                             proofJson:&proofJSON];
    XCTAssertEqual(ret.code, Success, @"proverCreateProof() failed!");

    // 12. Verifier verify proof
    NSDictionary *proof = [NSDictionary fromString:proofJSON];
    XCTAssertTrue(proof, @"serialization failed");

    NSDictionary *revealedAttr1 = proof[@"requested_proof"][@"revealed_attrs"][@"attr1_referent"];

    NSString *revocRegDefsJson = @"{}";
    NSString *revocRegsJson = @"{}";

    BOOL valid = false;

    ret = [[AnoncredsUtils sharedInstance] verifierVerifyProofRequest:proofReqJSON
                                                            proofJSON:proofJSON
                                                          schemasJSON:schemasJson
                                                   credentialDefsJSON:credentialDefsJson
                                                     revocRegDefsJSON:revocRegsJson
                                                        revocRegsJSON:revocRegsJson
                                                              isValid:&valid];
    XCTAssertEqual(ret.code, Success, @"verifierVerifyProof() failed!");
    XCTAssertTrue(valid);

    // 13. close wallet

    ret = [[WalletUtils sharedInstance] closeWalletWithHandle:walletHandle];
    XCTAssertEqual(ret.code, Success, @"closeWallet() failed!");

    [[IndyWallet sharedInstance] cleanupIndyKeychainWallet];
    [TestUtils cleanupStorage];
}


- (void)testAnoncredsDemoForRevocationProof {
    [TestUtils cleanupStorage];

    IndyHandle issuerWalletHandle = 0;
    IndyHandle proverWalletHandle = 0;
    NSError *ret = nil;

    //1. Create Issuer wallet, get wallet handle

    ret = [[WalletUtils sharedInstance] createAndOpenWalletWithPoolName:[TestUtils pool]
                                                                  xtype:nil
                                                                 handle:&issuerWalletHandle];
    XCTAssertEqual(ret.code, Success, @"WalletUtils::createAndOpenWallet() failed");

    //2. Create Prover wallet, get wallet handle
    ret = [[WalletUtils sharedInstance] createAndOpenWalletWithPoolName:[TestUtils pool]
                                                                  xtype:nil
                                                                 handle:&proverWalletHandle];
    XCTAssertEqual(ret.code, Success, @"WalletUtils::createAndOpenWallet() failed");

    // 3. Issuer create Schema
    NSString *schemaId;
    NSString *schemaJson;
    ret = [[AnoncredsUtils sharedInstance] issuerCreateSchemaWithName:[TestUtils gvtSchemaName]
                                                              version:[TestUtils schemaVersion]
                                                                attrs:[TestUtils gvtSchemaAttrs]
                                                            issuerDID:[TestUtils issuerDid]
                                                             schemaId:&schemaId
                                                           schemaJson:&schemaJson];
    XCTAssertEqual(ret.code, Success, @"issuerCreateSchemaForIssuerDID failed");

    XCTAssertTrue([schemaId isValid], @"invalid schemaId: %@", schemaId);
    XCTAssertTrue([schemaJson isValid], @"invalid schemaJson: %@", schemaJson);

    //4. Issuer create credential definition
    NSString *credentialDefId;
    NSString *credentialDefJSON;
    ret = [[AnoncredsUtils sharedInstance] issuerCreateAndStoreCredentialDefForSchema:schemaJson
                                                                            issuerDID:[TestUtils issuerDid]
                                                                                  tag:[TestUtils tag]
                                                                                 type:nil
                                                                           configJSON:@"{\"support_revocation\": true}"
                                                                         walletHandle:issuerWalletHandle
                                                                            credDefId:&credentialDefId
                                                                          credDefJson:&credentialDefJSON];
    XCTAssertEqual(ret.code, Success, @"issuerCreateCredentialDefinifionWithWalletHandle failed");

    //4. Issuer create revocation registry
    NSString *configJson = @"{\"max_cred_num\":5, \"issuance_type\":\"ISSUANCE_ON_DEMAND\"}";
    NSString *tailsWriterConfig = [NSString stringWithFormat:@"{\"base_dir\":\"%@\", \"uri_pattern\":\"\"}", [TestUtils tmpFilePathAppending:@"tails"]];
    NSNumber *tailsWriterHandle = nil;
    ret = [[BlobStorageUtils sharedInstance] openWriterWithType:[TestUtils defaultType]
                                                         config:tailsWriterConfig
                                                         handle:&tailsWriterHandle];
    XCTAssertEqual(ret.code, Success, @"AnoncredsUtils::openWriterWithType() failed");

    NSString *revocRefId;
    NSString *revocRegDefJson;
    ret = [[AnoncredsUtils sharedInstance] issuerCreateAndStoreRevocRegForCredentialDefId:credentialDefId
                                                                                issuerDID:[TestUtils issuerDid]
                                                                                     type:nil
                                                                                      tag:[TestUtils tag]
                                                                               configJSON:configJson
                                                                        tailsWriterHandle:[tailsWriterHandle intValue]
                                                                             walletHandle:issuerWalletHandle
                                                                               revocRegId:&revocRefId
                                                                          revocRegDefJson:&revocRegDefJson
                                                                        revocRegEntryJson:nil];
    XCTAssertEqual(ret.code, Success, @"issuerCreateAndStoreRevocRegForWithWalletHandle failed");

    //4. Prover create Master Secret
    ret = [[AnoncredsUtils sharedInstance] proverCreateMasterSecret:[TestUtils commonMasterSecretName]
                                                       walletHandle:proverWalletHandle
                                                  outMasterSecretId:nil];
    XCTAssertEqual(ret.code, Success, @"AnoncredsUtils::proverCreateMasterSecret() failed");

    // 5. Issuer create Credential Offer
    NSString *credentialOfferJson = nil;
    ret = [[AnoncredsUtils sharedInstance] issuerCreateCredentialOfferForCredDefId:credentialDefId
                                                                      walletHandle:issuerWalletHandle
                                                                     credOfferJson:&credentialOfferJson];
    XCTAssertEqual(ret.code, Success, @"AnoncredsUtils::issuerCreateAndStoreCredentialDef() failed!");

    //6. Prover create Credential Request
    NSString *credentialReq = nil;
    NSString *credentialReqMetadata = nil;
    ret = [[AnoncredsUtils sharedInstance] proverCreateCredentialReqForCredentialOffer:credentialOfferJson
                                                                     credentialDefJSON:credentialDefJSON
                                                                             proverDID:[TestUtils proverDid]
                                                                        masterSecretID:[TestUtils commonMasterSecretName]
                                                                          walletHandle:proverWalletHandle
                                                                           credReqJson:&credentialReq
                                                                   credReqMetadataJson:&credentialReqMetadata];
    XCTAssertEqual(ret.code, Success, @"AnoncredsUtils::proverCreateAndStoreCredentialReq() failed");
    XCTAssertTrue([credentialReq isValid], @"invalid credentialRequest: %@", credentialReq);
    XCTAssertTrue([credentialReqMetadata isValid], @"invalid credentialReqMetadata: %@", credentialReqMetadata);
    NSLog(@"credentialReqJson: %@", credentialReq);

    //7. Issuer create Tails reader
    NSDictionary *revRegDef = [NSDictionary fromString:revocRegDefJson];
    NSString *location = revRegDef[@"value"][@"tails_location"];
    NSString *hash = revRegDef[@"value"][@"tails_hash"];

    NSNumber *blobStorageReaderHandle = nil;
    ret = [[BlobStorageUtils sharedInstance] openReaderWithType:[TestUtils defaultType]
                                                         config:tailsWriterConfig
                                                         handle:&blobStorageReaderHandle];
    
    XCTAssertEqual(ret.code, Success, @"AnoncredsUtils::openReaderWithType() failed");

    //8. Issuer create Credential
    NSString *credentialJson = nil;
    NSString *credentialRevId = nil;
    NSString *revocRegDeltaJson = nil;
    ret = [[AnoncredsUtils sharedInstance] issuerCreateCredentialForCredentialRequest:credentialReq
                                                                        credOfferJSON:credentialOfferJson
                                                                       credValuesJSON:[[AnoncredsUtils sharedInstance] getGvtCredentialValuesJson]
                                                                             revRegId:revocRefId
                                                              blobStorageReaderHandle:blobStorageReaderHandle
                                                                         walletHandle:issuerWalletHandle
                                                                             credJson:&credentialJson
                                                                          credRevocId:&credentialRevId
                                                                    revocRegDeltaJSON:&revocRegDeltaJson];
    XCTAssertTrue([credentialJson isValid], @"invalid credentialJson: %@", credentialJson);
    XCTAssertTrue([credentialRevId isValid], @"invalid credentialRevId: %@", credentialRevId);
    XCTAssertTrue([revocRegDeltaJson isValid], @"invalid revocRegDeltaJson: %@", revocRegDeltaJson);

    // 9. Prover store received Credential
    ret = [[AnoncredsUtils sharedInstance] proverStoreCredential:credentialJson
                                                          credID:[[AnoncredsUtils sharedInstance] credentialId1]
                                                     credReqJSON:credentialReq
                                             credReqMetadataJSON:credentialReqMetadata
                                                     credDefJSON:credentialDefJSON
                                                   revRegDefJSON:revocRegDefJson
                                                    walletHandle:proverWalletHandle
                                                       outCredId:nil];
    XCTAssertEqual(ret.code, Success, @"AnoncredsUtils::proverStoreCredential() failed");


    // 10. Prover gets Credentials for Proof Request
    NSString *proofReqJson = @"{"\
                             " \"nonce\":\"123432421212\","\
                             " \"name\":\"proof_req_1\","\
                             " \"version\":\"0.1\","\
                             " \"requested_attributes\":"\
                             "             {\"attr1_referent\":"\
                             "                        {"\
                             "                          \"name\":\"name\""\
                             "                        },"
            "              \"attr2_referent\":"
            "                        {"
            "                          \"name\":\"phone\""
            "                        }"
            "             },"\
                             " \"requested_predicates\":"\
                             "             {"\
                             "              \"predicate1_referent\":"\
                             "                      {\"name\":\"age\",\"p_type\":\">=\",\"p_value\":18}"\
                             "             }"\
                             "}";

    NSString *credentialsJson = nil;

    ret = [[AnoncredsUtils sharedInstance] proverGetCredentialsForProofReq:proofReqJson
                                                              walletHandle:proverWalletHandle
                                                           credentialsJson:&credentialsJson];
    XCTAssertEqual(ret.code, Success, @"proverGetCredentialsForProofReq() failed!");

    NSDictionary *credentials = [NSDictionary fromString:credentialsJson];
    XCTAssertTrue(credentials, @"serialization failed");

    NSDictionary *credentials_for_attr_1 = credentials[@"attrs"][@"attr1_referent"][0];
    XCTAssertTrue(credentials_for_attr_1, @"no object for key \"attr1_referent\"");
    NSString *credentialReferent = credentials_for_attr_1[@"cred_info"][@"referent"];

    //11. Prover create Revocation State
    NSString *revocStateJson = nil;
    NSNumber *timestamp = @100;

    ret = [[AnoncredsUtils sharedInstance] createRevocationStateForCredRevID:credentialRevId
                                                                   timestamp:timestamp
                                                               revRegDefJSON:revocRegDefJson
                                                             revRegDeltaJSON:revocRegDeltaJson
                                                     blobStorageReaderHandle:blobStorageReaderHandle
                                                                revStateJson:&revocStateJson];

    XCTAssertEqual(ret.code, Success, @"AnoncredsUtils::createRevocationInfoForTimestamp() failed");

    // 12. Prover create Proof
    NSString *requestedCredentialsJson = [NSString stringWithFormat:@"{\
                                     \"self_attested_attributes\":{\"attr2_referent\":\"value\"},\
                                     \"requested_attributes\":{\"attr1_referent\":{\"cred_id\":\"%@\",\"revealed\":true, \"timestamp\":%@}},\
                                     \"requested_predicates\":{\"predicate1_referent\":{\"cred_id\":\"%@\", \"timestamp\":%@}}\
                                     }", credentialReferent, timestamp, credentialReferent, timestamp];


    NSString *schemasJson = [[AnoncredsUtils sharedInstance] toJson:@{schemaId: [NSDictionary fromString:schemaJson]}];

    NSString *credentialDefsJson = [[AnoncredsUtils sharedInstance] toJson:@{credentialDefId: [NSDictionary fromString:credentialDefJSON]}];

    NSString *revocStatesJson = [[AnoncredsUtils sharedInstance] toJson:@{revocRefId: @{[timestamp stringValue]: [NSDictionary fromString:revocStateJson]}}];

    NSString *proofJson;
    ret = [[AnoncredsUtils sharedInstance] proverCreateProofForRequest:proofReqJson
                                              requestedCredentialsJSON:requestedCredentialsJson
                                                        masterSecretID:[TestUtils commonMasterSecretName]
                                                           schemasJSON:schemasJson
                                                    credentialDefsJSON:credentialDefsJson
                                                       revocStatesJSON:revocStatesJson
                                                          walletHandle:proverWalletHandle
                                                             proofJson:&proofJson];
    XCTAssertEqual(ret.code, Success, @"AnoncredsUtils::proverCreateProof() failed");
    XCTAssertTrue([proofJson isValid], @"invalid proofJson: %@", proofJson);

    NSDictionary *proof = [NSDictionary fromString:proofJson];
    NSDictionary *revealedAttr1 = proof[@"requested_proof"][@"revealed_attrs"][@"attr1_referent"];
    NSString *raw = revealedAttr1[@"raw"];

    XCTAssertTrue([raw isEqualToString:@"Alex"]);

    NSString *attestedAttrUUID = proof[@"requested_proof"][@"self_attested_attrs"][@"attr2_referent"];
    XCTAssertTrue([attestedAttrUUID isEqualToString:@"value"]);

    NSString *revocRegDefsJson = [[AnoncredsUtils sharedInstance] toJson:@{revocRefId: [NSDictionary fromString:revocRegDefJson]}];

    NSString *revocRegsJson = [[AnoncredsUtils sharedInstance] toJson:@{revocRefId: @{[timestamp stringValue]: [NSDictionary fromString:revocRegDeltaJson]}}];

    // 13. Verifier verify proof
    BOOL isValid = NO;

    ret = [[AnoncredsUtils sharedInstance] verifierVerifyProofRequest:proofReqJson
                                                            proofJSON:proofJson
                                                          schemasJSON:schemasJson
                                                   credentialDefsJSON:credentialDefsJson
                                                     revocRegDefsJSON:revocRegDefsJson
                                                        revocRegsJSON:revocRegsJson
                                                              isValid:&isValid];

    XCTAssertEqual(ret.code, Success, @"AnoncredsUtils::verifierVerifyProof() failed");
    XCTAssertTrue(isValid, @"isValid == NO");
    [TestUtils cleanupStorage];
}

@end
