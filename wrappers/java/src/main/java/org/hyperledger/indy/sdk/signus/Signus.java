package org.hyperledger.indy.sdk.signus;

import java.util.concurrent.CompletableFuture;

import org.hyperledger.indy.sdk.IndyException;
import org.hyperledger.indy.sdk.IndyJava;
import org.hyperledger.indy.sdk.LibIndy;
import org.hyperledger.indy.sdk.pool.Pool;
import org.hyperledger.indy.sdk.signus.SignusResults.CreateAndStoreMyDidResult;
import org.hyperledger.indy.sdk.signus.SignusResults.AuthenticatedEncryptResult;
import org.hyperledger.indy.sdk.signus.SignusResults.ReplaceKeysStartResult;
import org.hyperledger.indy.sdk.wallet.Wallet;

import com.sun.jna.Callback;
import com.sun.jna.Pointer;

/**
 * signus.rs API
 */

/**
 * High level wrapper around signus SDK functions.
 */
public class Signus extends IndyJava.API {

	private Signus() {

	}

	/*
	 * STATIC CALLBACKS
	 */

	/**
	 * Callback used when createAndStoreMyDid completes.
	 */
	private static Callback createAndStoreMyDidCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, String did, String verkey, String pk) {

			CompletableFuture<CreateAndStoreMyDidResult> future = (CompletableFuture<CreateAndStoreMyDidResult>) removeFuture(xcommand_handle);
			if (! checkCallback(future, err)) return;

			CreateAndStoreMyDidResult result = new CreateAndStoreMyDidResult(did, verkey, pk);
			future.complete(result);
		}
	};

	/**
	 * Callback used when replaceKeysStart completes.
	 */
	private static Callback replaceKeysStartCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, String verkey, String pk) {

			CompletableFuture<ReplaceKeysStartResult> future = (CompletableFuture<ReplaceKeysStartResult>) removeFuture(xcommand_handle);
			if (! checkCallback(future, err)) return;

			ReplaceKeysStartResult result = new ReplaceKeysStartResult(verkey, pk);
			future.complete(result);
		}
	};

	/**
	 * Callback used when replaceKeysApply completes.
	 */
	private static Callback replaceKeysApplyCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err) {

			CompletableFuture<Void> future = (CompletableFuture<Void>) removeFuture(xcommand_handle);
			if (! checkCallback(future, err)) return;

			Void result = null;
			future.complete(result);
		}
	};

	/**
	 * Callback used when storeTheirDid completes.
	 */
	private static Callback storeTheirDidCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err) {

			CompletableFuture<Void> future = (CompletableFuture<Void>) removeFuture(xcommand_handle);
			if (! checkCallback(future, err)) return;

			Void result = null;
			future.complete(result);
		}
	};

	/**
	 * Callback used when sign completes.
	 */
	private static Callback signCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, Pointer signature_raw, int signature_len) {

			CompletableFuture<byte[]> future = (CompletableFuture<byte[]>) removeFuture(xcommand_handle);
			if (! checkCallback(future, err)) return;

			byte[] result = new byte[signature_len];
			signature_raw.read(0, result, 0, signature_len);
			future.complete(result);
		}
	};

	/**
	 * Callback used when verifySignature completes.
	 */
	private static Callback verifySignatureCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, boolean valid) {

			CompletableFuture<Boolean> future = (CompletableFuture<Boolean>) removeFuture(xcommand_handle);
			if (! checkCallback(future, err)) return;

			Boolean result = Boolean.valueOf(valid);
			future.complete(result);
		}
	};

	/**
	 * Callback used when authenticate encrypt completes.
	 */
	private static Callback authenticatedEncryptCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, Pointer encrypted_msg_raw, int encrypted_msg_len, Pointer nonce_raw, int nonce_len) {

			CompletableFuture<AuthenticatedEncryptResult> future = (CompletableFuture<AuthenticatedEncryptResult>) removeFuture(xcommand_handle);
			if (! checkCallback(future, err)) return;

			byte[] encryptedMsg = new byte[encrypted_msg_len];
			encrypted_msg_raw.read(0, encryptedMsg, 0, encrypted_msg_len);

			byte[] nonce = new byte[nonce_len];
			nonce_raw.read(0, nonce, 0, nonce_len);

			AuthenticatedEncryptResult result = new AuthenticatedEncryptResult(encryptedMsg, nonce);
			future.complete(result);
		}
	};

	/**
	 * Callback used when authenticate decrypt completes.
	 */
	private static Callback authenticateDecryptCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, Pointer decrypted_msg_raw, int decrypted_msg_len) {

			CompletableFuture<byte[]> future = (CompletableFuture<byte[]>) removeFuture(xcommand_handle);
			if (! checkCallback(future, err)) return;

			byte[] result = new byte[decrypted_msg_len];
			decrypted_msg_raw.read(0, result, 0, decrypted_msg_len);
			future.complete(result);
		}
	};

	/**
	 * Callback used when anonymous encrypt completes.
	 */
	private static Callback anonymousEncryptCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, Pointer encrypted_msg_raw, int encrypted_msg_len) {

			CompletableFuture<byte[]> future = (CompletableFuture<byte[]>) removeFuture(xcommand_handle);
			if (! checkCallback(future, err)) return;

			byte[] encryptedMsg = new byte[encrypted_msg_len];
			encrypted_msg_raw.read(0, encryptedMsg, 0, encrypted_msg_len);

			future.complete(encryptedMsg);
		}
	};

	/**
	 * Callback used when anonymous decrypt completes.
	 */
	private static Callback anonymousDecryptCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, Pointer decrypted_msg_raw, int decrypted_msg_len) {

			CompletableFuture<byte[]> future = (CompletableFuture<byte[]>) removeFuture(xcommand_handle);
			if (! checkCallback(future, err)) return;

			byte[] result = new byte[decrypted_msg_len];
			decrypted_msg_raw.read(0, result, 0, decrypted_msg_len);
			future.complete(result);
		}
	};

	/*
	 * STATIC METHODS
	 */

	/**
	 * Creates keys (signing and encryption keys) for a new DID owned by the caller.
	 *
	 * @param wallet  The wallet.
	 * @param didJson Identity information as json.
	 * @return A future that resolves to a CreateAndStoreMyDidResult instance.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<CreateAndStoreMyDidResult> createAndStoreMyDid(
			Wallet wallet,
			String didJson) throws IndyException {

		CompletableFuture<CreateAndStoreMyDidResult> future = new CompletableFuture<CreateAndStoreMyDidResult>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_create_and_store_my_did(
				commandHandle,
				walletHandle,
				didJson,
				createAndStoreMyDidCb);

		checkResult(result);

		return future;
	}

	/**
	 * Generated new signing and encryption keys for an existing DID owned by the caller.
	 *
	 * @param wallet       The wallet.
	 * @param did          The DID
	 * @param identityJson identity information as json.
	 * @return A future that resolves to a ReplaceKeysStartResult instance.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<ReplaceKeysStartResult> replaceKeysStart(
			Wallet wallet,
			String did,
			String identityJson) throws IndyException {

		CompletableFuture<ReplaceKeysStartResult> future = new CompletableFuture<ReplaceKeysStartResult>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_replace_keys_start(
				commandHandle,
				walletHandle,
				did,
				identityJson,
				replaceKeysStartCb);

		checkResult(result);

		return future;
	}

	/**
	 * Apply temporary keys as main for an existing DID.
	 *
	 * @param wallet The wallet.
	 * @param did    The DID
	 * @return A future that resolves no value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Void> replaceKeysApply(
			Wallet wallet,
			String did) throws IndyException {

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_replace_keys_apply(
				commandHandle,
				walletHandle,
				did,
				replaceKeysApplyCb);

		checkResult(result);

		return future;
	}

	/**
	 * Saves their DID for a pairwise connection in a secured Wallet so that it can be used to verify transaction.
	 *
	 * @param wallet       The wallet.
	 * @param identityJson Identity information as json.
	 * @return A future that does not resolve any value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Void> storeTheirDid(
			Wallet wallet,
			String identityJson) throws IndyException {

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_store_their_did(
				commandHandle,
				walletHandle,
				identityJson,
				storeTheirDidCb);

		checkResult(result);

		return future;
	}

	/**
	 * Signs a message by a signing key associated with my DID. The DID with a signing key.
	 *
	 * @param wallet  The wallet.
	 * @param did     signing DID
	 * @param message a message to be signed
	 * @return A future that resolves to a a signature string.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<byte[]> sign(
			Wallet wallet,
			String did,
			byte[] message) throws IndyException {

		CompletableFuture<byte[]> future = new CompletableFuture<byte[]>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_sign(
				commandHandle,
				walletHandle,
				did,
				message,
				message.length,
				signCb);

		checkResult(result);

		return future;
	}

	/**
	 * Verify a signature created by a key associated with a DID.
	 *
	 * @param wallet    The wallet.
	 * @param pool      The pool.
	 * @param did       DID that signed the message
	 * @param message   message
	 * @param signature a signature to be verified
	 * @return A future that resolves to true if signature is valid, otherwise false.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Boolean> verifySignature(
			Wallet wallet,
			Pool pool,
			String did,
			byte[] message,
			byte[] signature) throws IndyException {

		CompletableFuture<Boolean> future = new CompletableFuture<Boolean>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();
		int poolHandle = pool.getPoolHandle();

		int result = LibIndy.api.indy_verify_signature(
				commandHandle,
				walletHandle,
				poolHandle,
				did,
				message,
				message.length,
				signature,
				signature.length,
				verifySignatureCb);

		checkResult(result);

		return future;
	}

	/**
	 * Encrypts a message by public-key (associated with their did) authenticated-encryption scheme
	 *
	 * @param wallet  The wallet.
	 * @param pool    The pool.
	 * @param myDid   encrypting DID
	 * @param did     encrypting DID
	 * @param message a message to be signed
	 * @return A future that resolves to a JSON string containing an encrypted message and nonce.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<AuthenticatedEncryptResult> authenticatedEncrypt(
			Wallet wallet,
			Pool pool,
			String myDid,
			String did,
			byte[] message) throws IndyException {

		CompletableFuture<AuthenticatedEncryptResult> future = new CompletableFuture<AuthenticatedEncryptResult>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();
		int poolHandle = pool.getPoolHandle();

		int result = LibIndy.api.indy_authenticated_encrypt(
				commandHandle,
				walletHandle,
				poolHandle,
				myDid,
				did,
				message,
				message.length,
				authenticatedEncryptCb);

		checkResult(result);

		return future;
	}

	/**
	 * Decrypts a message by public-key authenticated-encryption scheme using nonce.
	 *
	 * @param wallet       The wallet.
	 * @param myDid        DID
	 * @param did          DID that signed the message
	 * @param encryptedMsg encrypted message
	 * @param nonce        nonce that encrypted message
	 * @return A future that resolves to a JSON string containing the decrypted message.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<byte[]> authenticatedDecrypt(
			Wallet wallet,
			String myDid,
			String did,
			byte[] encryptedMsg,
			byte[] nonce) throws IndyException {

		CompletableFuture<byte[]> future = new CompletableFuture<byte[]>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_authenticated_decrypt(
				commandHandle,
				walletHandle,
				myDid,
				did,
				encryptedMsg,
				encryptedMsg.length,
				nonce,
				nonce.length,
				authenticateDecryptCb);

		checkResult(result);

		return future;
	}

	/**
	 * Encrypts a message by public-key (associated with did) anonymous-encryption scheme.
	 *
	 * @param wallet  The wallet.
	 * @param pool    The pool.
	 * @param did     encrypted DID
	 * @param message a message to be signed
	 * @return A future that resolves to a JSON string containing an encrypted message and nonce.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<byte[]> anonymousEncrypt(
			Wallet wallet,
			Pool pool,
			String did,
			byte[] message) throws IndyException {

		CompletableFuture<byte[]> future = new CompletableFuture<byte[]>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();
		int poolHandle = pool.getPoolHandle();

		int result = LibIndy.api.indy_anonymous_encrypt(
				commandHandle,
				walletHandle,
				poolHandle,
				did,
				message,
				message.length,
				anonymousEncryptCb);

		checkResult(result);

		return future;
	}

	/**
	 * Decrypts a message by public-key anonymous-encryption scheme.
	 *
	 * @param wallet       The wallet.
	 * @param did          DID that signed the message
	 * @param encryptedMsg encrypted message
	 * @return A future that resolves to a JSON string containing the decrypted message.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<byte[]> anonymousDecrypt(
			Wallet wallet,
			String did,
			byte[] encryptedMsg) throws IndyException {

		CompletableFuture<byte[]> future = new CompletableFuture<byte[]>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_anonymous_decrypt(
				commandHandle,
				walletHandle,
				did,
				encryptedMsg,
				encryptedMsg.length,
				anonymousDecryptCb);

		checkResult(result);

		return future;
	}
}
