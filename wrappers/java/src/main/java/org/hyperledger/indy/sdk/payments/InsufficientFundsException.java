package org.hyperledger.indy.sdk.payments;

import org.hyperledger.indy.sdk.ErrorCode;
import org.hyperledger.indy.sdk.IndyException;

public class InsufficientFundsException extends IndyException {
    private static final long serialVersionUID = 6397499268992083528L;
    private static final String message = "Insufficient funds on inputs";

    /**
     * Initializes a new {@link InsufficientFundsException} with the specified message.
     */
    public InsufficientFundsException() {
        super(message, ErrorCode.InsufficientFundsError.value());
    }

    /**
     * Initializes a new {@link InsufficientFundsException} with the specified message.
     *
     * @param sdkMessage The SDK error message.
     * @param sdkBacktrace The SDK error backtrace.
     */
    public InsufficientFundsException(String sdkMessage, String sdkBacktrace) {
        super(sdkMessage, ErrorCode.InsufficientFundsError.value(), sdkBacktrace);
    }
}
