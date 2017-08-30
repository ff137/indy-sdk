﻿using System.Threading.Tasks;
using static Hyperledger.Indy.Sdk.IndyNativeMethods;

namespace Hyperledger.Indy.Sdk.Utils
{
    internal static class CallbackHelper
    {
        /// <summary>
        /// Gets the callback to use for functions that don't return a value.
        /// </summary>
        public static NoValueDelegate NoValueCallback = (xcommand_handle, err) =>
        {
            var taskCompletionSource = PendingCommands.Remove<bool>(xcommand_handle);

            if (!CheckCallback(taskCompletionSource, err))
                return;

            taskCompletionSource.SetResult(true);
        };

        /// <summary>
        /// Checks the result from a Sovrin function call.
        /// </summary>
        /// <exception cref="IndyException">If the result is not a success result a SovrinException will be thrown.</exception>
        /// <param name="result">The result to check.</param>
        public static void CheckResult(int result)
        {
            if (result != (int)ErrorCode.Success)
                throw IndyException.fromErrorCode(result);
        }

        /// <summary>
        /// Checks the result of a callback made by the Sovrin library.
        /// </summary>
        /// <typeparam name="T">The type the promise will return.</typeparam>
        /// <param name="taskCompletionSource">The source controlling the async result.</param>
        /// <param name="errorCode">The error code returned to the callback by the sovrin function.</param>
        /// <returns>true if the error code was success, otherwise false.</returns>
        /// <exception cref="IndyException">If the errorCode is not a success result a SovrinException will be thrown.</exception>
        public static bool CheckCallback<T>(TaskCompletionSource<T> taskCompletionSource, int errorCode)
        {
            if (errorCode != (int)ErrorCode.Success)
            {
                taskCompletionSource.SetException(IndyException.fromErrorCode(errorCode));
                return false;
            }

            return true;
        }

        /// <summary>
        /// Checks the result of a callback made by the Sovrin library.
        /// </summary>
        /// <param name="errorCode">The error code returned to the callback by the sovrin function.</param>
        /// <exception cref="IndyException">If the errorCode is not a success result a SovrinException will be thrown.</exception>
        public static void CheckCallback(int errorCode)
        {
            if (errorCode != (int)ErrorCode.Success)
                throw IndyException.fromErrorCode(errorCode);
        }
    }
}
