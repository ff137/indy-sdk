package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithPoolAndSingleWallet;
import org.junit.Test;

import static org.junit.Assert.assertTrue;

public class RevocRegEntryRequestTest extends IndyIntegrationTestWithPoolAndSingleWallet {

	@Test
	public void testBuildRevocRegEntryRequestWorks() throws Exception {
		String expectedResult =
				"\"operation\": {\n" +
						"            \"type\": \"114\",\n" +
						"            \"revocRegDefId\": \"RevocRegID\",\n" +
						"            \"revocDefType\": \"CL_ACCUM\",\n" +
						"            \"value\": {\n" +
						"                \"prevAccum\": \"123456789\",\n" +
						"                \"accum\": \"123456789\",\n" +
						"                \"issued\": [],\n" +
						"                \"revoked\": []\n" +
						"           }\n" +
						"        }";

		String value = "{\n" +
				"        \"accum\": \"123456789\",\n" +
				"        \"prevAccum\": \"123456789\",\n" +
				"        \"issued\": [],\n" +
				"        \"revoked\": []\n" +
				"    }";

		String request = Ledger.buildRevocRegEntryRequest(DID, "RevocRegID", "CL_ACCUM", value).get();

		assertTrue(request.replaceAll("\\s+", "").contains(expectedResult.replaceAll("\\s+", "")));
	}
}
