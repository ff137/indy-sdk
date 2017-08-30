﻿namespace Hyperledger.Indy.Sdk.SignUsApi
{
    /// <summary>
    /// The result of encryption.
    /// </summary>
    public class EncryptResult
    {
        /// <summary>
        /// Initializes a new EncryptionResult.
        /// </summary>
        /// <param name="encryptedMsg">The encrypted message.</param>
        /// <param name="nonce">The nonce.</param>
        internal EncryptResult(byte[] encryptedMsg, byte[] nonce)
        {
            EncryptedMsg = encryptedMsg;
            Nonce = nonce;
        }

        /// <summary>
        /// Gets the encrypted message.
        /// </summary>
        public byte[] EncryptedMsg { get; }

        /// <summary>
        /// Gets the nonce.
        /// </summary>
        public byte[] Nonce { get; }

    }
}
