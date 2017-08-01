//
//  AgentMediumCases.m
//  libindy-demo
//
//  Created by Anastasia Tarasova on 22.06.17.
//  Copyright © 2017 Kirill Neznamov. All rights reserved.
//

#import <Foundation/Foundation.h>
#import <XCTest/XCTest.h>
#import <libindy/libindy.h>
#import "TestUtils.h"

@interface AgentMediumCases : XCTestCase

@end

@implementation AgentMediumCases

- (void)setUp
{
    [super setUp];
    // Put setup code here. This method is called before the invocation of each test method in the class.
}

- (void)tearDown
{
    // Put teardown code here. This method is called after the invocation of each test method in the class.
    [super tearDown];
}

// MARK: - Add identity
- (void)testAgentAddIdentityWorksForIncomingConnectionRequireLedgerRequestButPoolHandleIsInvalid
{
    [TestUtils cleanupStorage];
    NSError *ret;
    NSString *endpoint = @"127.0.0.1:9812";
    NSString *poolName = @"indy_agent_add_identity_works_for_incoming_connection_require_ledger_request_but_pool_handle_is_invalid";
    
    // 1. Obtain pool handle
    IndyHandle poolHandle = 0;
    ret = [[PoolUtils sharedInstance] createAndOpenPoolLedgerConfigWithName:poolName
                                                                 poolHandle:&poolHandle];
    XCTAssertEqual(ret.code, Success, @"PoolUtils::createAndOpenPoolLedgerConfigWithName() failed");
    
    // 2. Obtain listener's wallet
    IndyHandle listenerWalletHandle = 0;
    ret = [[WalletUtils sharedInstance] createAndOpenWalletWithPoolName:poolName
                                                                  xtype:nil
                                                                 handle:&listenerWalletHandle];
    XCTAssertEqual(ret.code, Success, @"PoolUtils::createAndOpenWalletWithPoolName() failed");
    
    // 3. Obtain trustee's wallet
    IndyHandle trusteeWalletHandle = 0;
    ret = [[WalletUtils sharedInstance] createAndOpenWalletWithPoolName:poolName
                                                                  xtype:nil
                                                                 handle:&trusteeWalletHandle];
    XCTAssertEqual(ret.code, Success, @"PoolUtils::createAndOpenWalletWithPoolName() failed");
    
    // 4. Create and store listener's did
    NSString *listenerDid;
    NSString *listenerVerKey;
    NSString *listenerPubKey;
    ret = [[SignusUtils sharedInstance] createAndStoreMyDidWithWalletHandle:listenerWalletHandle
                                                                       seed:nil
                                                                   outMyDid:&listenerDid
                                                                outMyVerkey:&listenerVerKey
                                                                    outMyPk:&listenerPubKey];
    XCTAssertEqual(ret.code, Success, @"SignusUtils::createAndStoreMyDidWithWalletHandle() failed for listener");
    
    // 5. create trustee did
    
    NSString *trusteeDid;
    ret =[[SignusUtils sharedInstance] createAndStoreMyDidWithWalletHandle:trusteeWalletHandle
                                                                      seed:@"000000000000000000000000Trustee1"
                                                                  outMyDid:&trusteeDid
                                                               outMyVerkey:nil
                                                                   outMyPk:nil];
    XCTAssertEqual(ret.code, Success, @"SignusUtils::createAndStoreMyDidWithWalletHandle() failed for trustee");
    
    // 6. Build nym request for listener
    NSString *listenerNymRequest;
    ret = [[LedgerUtils sharedInstance] buildNymRequestWithSubmitterDid:trusteeDid
                                                              targetDid:listenerDid
                                                                 verkey:listenerVerKey
                                                                  alias:nil
                                                                   role:nil
                                                             outRequest:&listenerNymRequest];
     XCTAssertEqual(ret.code, Success, @"LedgerUtils::buildNymRequestWithSubmitterDid() failed");
    
    // 7. Sign and submit listener's nym request
    NSString *listenerNymResponse;
    ret = [[LedgerUtils sharedInstance] signAndSubmitRequestWithPoolHandle:poolHandle
                                                              walletHandle:trusteeWalletHandle
                                                              submitterDid:trusteeDid
                                                               requestJson:listenerNymRequest
                                                           outResponseJson:&listenerNymResponse];
    XCTAssertEqual(ret.code, Success, @"LedgerUtils::signAndSubmitRequestWithPoolHandle() failed for listenerNymRequest");
    
    // 8. Build listener attribute request
    NSString *rawJson =[NSString stringWithFormat:@"{\"endpoint\":{\"ha\":\"%@\", \"verkey\":\"%@\"}}", endpoint, listenerPubKey];
    NSString *listenerAttributeRequest;
    ret = [[LedgerUtils sharedInstance] buildAttribRequestWithSubmitterDid:listenerDid
                                                                                                targetDid:listenerDid
                                                                                                     hash:nil
                                                                                                      raw:rawJson
                                                                                                      enc:nil
                                                                resultJson:&listenerAttributeRequest];
     XCTAssertEqual(ret.code, Success, @"LedgerUtils::buildAttribRequestWithSubmitterDid() failed");
    
    // 9. Sign and submit listener's attribute request
    NSString *listenerAttributeResponse;
    ret = [[LedgerUtils sharedInstance] signAndSubmitRequestWithPoolHandle:poolHandle
                                                              walletHandle:listenerWalletHandle
                                                              submitterDid:listenerDid
                                                               requestJson:listenerAttributeRequest
                                                           outResponseJson:&listenerAttributeResponse];
    XCTAssertEqual(ret.code, Success, @"LedgerUtils::buildAttribRequestWithSubmitterDid() failed for listenerAttributeResponse");
    
    // 10. listen
    IndyHandle listenerHandle = 0;
    ret = [[AgentUtils sharedInstance] listenForEndpoint:endpoint
                                       connectionCallback:nil
                                          messageCallback:nil
                                        outListenerHandle:&listenerHandle];
    XCTAssertEqual(ret.code, Success, @"LedgerUtils::listenWithEndpoint()");
    
    // 11. Add identity
    IndyHandle invalidPoolHandle = listenerHandle;
    ret = [[AgentUtils sharedInstance] addIdentityForListenerHandle:listenerHandle
                                                         poolHandle:invalidPoolHandle
                                                       walletHandle:listenerWalletHandle
                                                                did:listenerDid];
    XCTAssertEqual(ret.code, Success, @"LedgerUtils::addIdentityForListenerHandle() failed for listenerDid");
    
    /* TODO
     * Currently pool_handle and wallet_handle of add_identity will be checked only at required:
     * when listener will check incoming connection and go to ledger for info.
     * As result, add_identity will be successful but next connect will fail.
     * Possible the test should be split into two:
     * - add_identity_works_for_incompatible_pool_and_wallet
     *    with immediately check in the library
     * - connect_works_for_incorrect_connect_request
     *    actual info in ledger or listener_wallet, wrong public key in sender_wallet
     */
    
    // 12. Connect
    NSString *senderDid = [NSString stringWithString:trusteeDid];
    IndyHandle senderWalletHandle = trusteeWalletHandle;
    
    ret = [[AgentUtils sharedInstance] connectWithPoolHandle:poolHandle
                                                walletHandle:senderWalletHandle
                                                   senderDid:senderDid
                                                 receiverDid:listenerDid
                                             messageCallback:nil
                                         outConnectionHandle:nil];
    XCTAssertEqual(ret.code, CommonInvalidState, @"AgentUtils::connectWithPoolHandle() returned wrong code");
    [TestUtils cleanupStorage];
}

// MARK: - Close connection

- (void)testAgentCloseConnectionWorksForIncorrectConnectionHandle
{
    [TestUtils cleanupStorage];
    NSError *ret;
    
    // 1. create and open wallet handle
    IndyHandle walletHandle = 0;
    ret = [[WalletUtils sharedInstance] createAndOpenWalletWithPoolName:@"pool6"
                                                                  xtype:nil
                                                                 handle:&walletHandle];
    XCTAssertEqual(ret.code, Success, @"WalletUtils::createAndOpenWalletWithPoolName() failed");
    
    // 2. obtain did
    NSString *did;
    NSString *verKey;
    NSString *pubKey;
    ret = [[SignusUtils sharedInstance] createAndStoreMyDidWithWalletHandle:walletHandle
                                                                       seed:nil
                                                                   outMyDid:&did
                                                                outMyVerkey:&verKey
                                                                    outMyPk:&pubKey];
    XCTAssertEqual(ret.code, Success, @"SignusUtils::createAndStoreMyDidWithWalletHandle() failed");
    
    // 3. listen
    NSString *endpoint = @"127.0.0.1:9807";
    
    XCTestExpectation* messageExpectation = [[ XCTestExpectation alloc] initWithDescription: @"message completion finished"];
    
    IndyHandle listenerHandler = 0;
    __block NSString* messageFromClient;
    ret = [[AgentUtils sharedInstance] listenForEndpoint:endpoint
                                       connectionCallback:nil
                                          messageCallback:^(IndyHandle connectionHandle, NSString *message)
    {
        messageFromClient = message;
        [messageExpectation fulfill];
    }
                                        outListenerHandle:&listenerHandler];
    XCTAssertEqual(ret.code, Success, @"AgentUtils::listenWithEndpoint() failed");

    // 4. add identity
    ret = [[AgentUtils sharedInstance] addIdentityForListenerHandle:listenerHandler
                                                         poolHandle:-1
                                                       walletHandle:walletHandle
                                                                did:did];
    XCTAssertEqual(ret.code, Success, @"AgentUtils::addIdentityForListenerHandle() failed");
    
    // 5. store their did from parts
    ret = [[SignusUtils sharedInstance] storeTheirDidFromPartsWithWalletHandle:walletHandle
                                                                      theirDid:did
                                                                       theirPk:pubKey
                                                                   theirVerkey:verKey
                                                                      endpoint:endpoint];
    XCTAssertEqual(ret.code, Success, @"AgentUtils::storeTheirDidFromPartsWithWalletHandle() failed");
    
    // 6. Connect
    IndyHandle connectionHandle = 0;
    ret = [[AgentUtils sharedInstance] connectWithPoolHandle:0
                                                walletHandle:walletHandle
                                                   senderDid:did
                                                 receiverDid:did
                                             messageCallback:nil
                                         outConnectionHandle:&connectionHandle];
    XCTAssertEqual(ret.code, Success, @"AgentUtils::connectWithPoolHandle() failed");
    
    // 7. Close connection
    ret = [[AgentUtils sharedInstance] closeConnection:connectionHandle + 100];
    XCTAssertEqual(ret.code, CommonInvalidStructure, @"AgentUtils::closeConnection() returned wrong code");
    
    // 8. send
    NSString *clientMessage = @"msg_from_cli_to_srv";
    ret = [[AgentUtils sharedInstance] sendWithConnectionHandler:connectionHandle
                                                         message:clientMessage];
    XCTAssertEqual(ret.code, Success, @"AgentUtils::sendWithConnectionHandler() failed");
    
    // 9. wait for message callback
    [self waitForExpectations: @[messageExpectation] timeout:[TestUtils defaultTimeout]];
    
    XCTAssertTrue([messageFromClient isEqualToString:clientMessage], @"wrong message from client!");
    
    [TestUtils cleanupStorage];
}

// MARK: - Close listener

- (void)testAgentCloseListenerWorksForIncorrectHandle
{
    [TestUtils cleanupStorage];
    NSError *ret;
    
    // 1.Create and open wallet
    IndyHandle walletHandle;
    ret = [[WalletUtils sharedInstance] createAndOpenWalletWithPoolName:@"pool9"
                                                                  xtype:nil
                                                                 handle:&walletHandle];
     XCTAssertEqual(ret.code, Success, @"WalletUtils::createAndOpenWalletWithPoolName() failed");
    
    // 2. obtain did
    NSString *did;
    NSString *verKey;
    NSString *pubKey;
    ret = [[SignusUtils sharedInstance] createAndStoreMyDidWithWalletHandle:walletHandle
                                                                       seed:nil
                                                                   outMyDid:&did
                                                                outMyVerkey:&verKey
                                                                    outMyPk:&pubKey];
    XCTAssertEqual(ret.code, Success, @"SignusUtils::createAndStoreMyDidWithWalletHandle() failed");
    
    // 3. listen
    NSString *endpoint = @"127.0.0.1:9809";
    
    XCTestExpectation* messageExpectation = [[ XCTestExpectation alloc] initWithDescription: @"message completion finished"];
    
    IndyHandle listenerHandler = 0;
    __block NSString* messageFromClient;
    ret = [[AgentUtils sharedInstance] listenForEndpoint:endpoint
                                       connectionCallback:nil
                                          messageCallback:^(IndyHandle connectionHandle, NSString *message)
           {
               messageFromClient = message;
               [messageExpectation fulfill];
           }
                                        outListenerHandle:&listenerHandler];
    XCTAssertEqual(ret.code, Success, @"AgentUtils::listenWithEndpoint() failed");

    // 4. add identity
    ret = [[AgentUtils sharedInstance] addIdentityForListenerHandle:listenerHandler
                                                         poolHandle:-1
                                                       walletHandle:walletHandle
                                                                did:did];
    XCTAssertEqual(ret.code, Success, @"AgentUtils::addIdentityForListenerHandle() failed");

    // 5. store their did from parts
    ret = [[SignusUtils sharedInstance] storeTheirDidFromPartsWithWalletHandle:walletHandle
                                                                      theirDid:did
                                                                       theirPk:pubKey
                                                                   theirVerkey:verKey
                                                                      endpoint:endpoint];
    XCTAssertEqual(ret.code, Success, @"AgentUtils::storeTheirDidFromPartsWithWalletHandle() failed");
    
    // 6. Connect
    IndyHandle connectionHandle = 0;
    ret = [[AgentUtils sharedInstance] connectWithPoolHandle:0
                                                walletHandle:walletHandle
                                                   senderDid:did
                                                 receiverDid:did
                                             messageCallback:nil
                                         outConnectionHandle:&connectionHandle];
    XCTAssertEqual(ret.code, Success, @"AgentUtils::connectWithPoolHandle() failed");
    
    // 7. close listener
    IndyHandle incorrectListenerHandle = connectionHandle; // + 1;
    ret = [[AgentUtils sharedInstance] closeListener:incorrectListenerHandle];
    XCTAssertEqual(ret.code, CommonInvalidStructure, @"AgentUtils::closeListener() returned wrong code");
    
    // 8. send
    NSString *clientMessage = @"msg_from_cli_to_srv";
    ret = [[AgentUtils sharedInstance] sendWithConnectionHandler:connectionHandle
                                                         message:clientMessage];
    XCTAssertEqual(ret.code, Success, @"AgentUtils::sendWithConnectionHandler() failed");
    
    // 9. wait for message callback
    [self waitForExpectations: @[messageExpectation] timeout:[TestUtils defaultTimeout]];
    
    XCTAssertTrue([messageFromClient isEqualToString:clientMessage], @"wrong message from client!");

    [TestUtils cleanupStorage];
}


@end
