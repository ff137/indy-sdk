package org.hyperledger.indy.sdk.anoncreds;

import org.hyperledger.indy.sdk.InvalidStructureException;
import org.junit.*;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;
import static org.junit.Assert.assertNotNull;

public class IssuerCreateClaimTest extends AnoncredsIntegrationTest {

	@Test
	public void testIssuerCreateClaimWorks() throws Exception {

		AnoncredsResults.IssuerCreateClaimResult createClaimResult =
				Anoncreds.issuerCreateClaim(wallet, claimRequest, gvtClaimValuesJson, null, - 1, - 1).get();
		assertNotNull(createClaimResult);
	}

	@Test
	public void testIssuerCreateClaimWorksForClaimValuesDoesNotCorrespondToClaimRequest() throws Exception {

		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(InvalidStructureException.class));

		Anoncreds.issuerCreateClaim(wallet, claimRequest, xyzClaimValuesJson, null, - 1, - 1).get();
	}

	@Test
	public void testIssuerCreateClaimWorksForInvalidClaimValues() throws Exception {

		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(InvalidStructureException.class));

		String claim = "{" +
				"        \"sex\":\"male\",\n" +
				"        \"age\":\"28\"" +
				"       }";

		Anoncreds.issuerCreateClaim(wallet, claimRequest, claim, null, - 1, - 1).get();
	}
}
