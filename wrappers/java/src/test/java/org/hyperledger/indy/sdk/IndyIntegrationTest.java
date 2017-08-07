package org.hyperledger.indy.sdk;

import org.hyperledger.indy.sdk.pool.Pool;
import org.hyperledger.indy.sdk.utils.InitHelper;
import org.hyperledger.indy.sdk.utils.StorageUtils;
import org.hyperledger.indy.sdk.wallet.Wallet;
import org.hyperledger.indy.sdk.wallet.WalletTypeInmem;
import org.junit.After;
import org.junit.Before;
import org.junit.BeforeClass;
import org.junit.Rule;
import org.junit.rules.ExpectedException;
import org.junit.rules.Timeout;

import java.io.IOException;
import java.util.HashSet;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;

public class IndyIntegrationTest {

	public static final String TRUSTEE_SEED = "000000000000000000000000Trustee1";

	@Rule
	public ExpectedException thrown = ExpectedException.none();

	@Rule
	public Timeout globalTimeout = new Timeout(1, TimeUnit.MINUTES);

	private static Boolean isWalletRegistered = false;

	@BeforeClass
	public static void registerWallet() throws IndyException, InterruptedException, ExecutionException {
		if (!isWalletRegistered){
			Wallet.registerWalletType("inmem", WalletTypeInmem.getInstance()).get();
		}
		isWalletRegistered = true;
	}

	@Before
	public void setUp() throws IOException {
		InitHelper.init();
		StorageUtils.cleanupStorage();
	}

	protected HashSet<Pool> openedPools = new HashSet<>();

	@After
	public void tearDown() throws IOException {
		openedPools.forEach(pool -> {
			try {
				pool.closePoolLedger();
			} catch (IndyException ignore) {
			}
		});
		openedPools.clear();
		StorageUtils.cleanupStorage();
	}
}
