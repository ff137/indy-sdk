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
