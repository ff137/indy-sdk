package org.hyperledger.indy.sdk.non_secrets;

import org.hyperledger.indy.sdk.wallet.WalletValueNotFoundException;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;


public class DeleteRecordTest extends NonSecretsIntegrationTest {

	@Test
	public void testDeleteRecordWorks() throws Exception {
		WalletRecord.add(wallet, type, id, value, tags).get();
		WalletRecord.delete(wallet, type, id).get();
		WalletRecord.add(wallet, type, id, value, tags).get();
	}

	@Test
	public void testDeleteRecordWorksForTwice() throws Exception {
		WalletRecord.add(wallet, type, id, value, tags).get();

		WalletRecord.delete(wallet, type, id).get();

		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletValueNotFoundException.class));

		WalletRecord.delete(wallet, type, id).get();
	}

	@Test
	public void testDeleteRecordWorksForNotFoundRecord() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletValueNotFoundException.class));

		WalletRecord.deleteTags(wallet, type, id, "[\"tagName1\"]").get();
	}
}